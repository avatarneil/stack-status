use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Check if GitHub CLI (gh) is installed
pub async fn is_installed() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get PR number for a branch
pub async fn get_pr_for_branch(branch: &str) -> Option<u64> {
    let output = Command::new("gh")
        .args(["pr", "view", branch, "--json", "number", "--jq", ".number"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()
}

/// Raw check data from gh CLI
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCheck {
    name: String,
    state: Option<String>,
    conclusion: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    details_url: Option<String>,
    bucket: Option<String>,
}

/// Normalized check information
#[derive(Debug, Serialize, Clone)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: Option<String>,
    pub duration_secs: Option<u64>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Passed,
    Failed,
    Running,
    Queued,
    Skipped,
    Cancelled,
    Unknown,
}

impl CheckStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            CheckStatus::Passed => "✓",
            CheckStatus::Failed => "✗",
            CheckStatus::Running => "◐",
            CheckStatus::Queued => "○",
            CheckStatus::Skipped => "○",
            CheckStatus::Cancelled => "⊘",
            CheckStatus::Unknown => "?",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            CheckStatus::Passed => "\x1b[32m",   // Green
            CheckStatus::Failed => "\x1b[31m",   // Red
            CheckStatus::Running => "\x1b[33m",  // Yellow
            CheckStatus::Queued => "\x1b[90m",   // Gray
            CheckStatus::Skipped => "\x1b[90m",  // Gray
            CheckStatus::Cancelled => "\x1b[90m", // Gray
            CheckStatus::Unknown => "\x1b[90m",  // Gray
        }
    }
}

/// Get CI checks for a branch
pub async fn get_checks(branch: &str) -> Result<Vec<Check>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "checks",
            branch,
            "--json",
            "name,state,conclusion,startedAt,completedAt,detailsUrl,bucket",
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let raw_checks: Vec<RawCheck> = serde_json::from_slice(&output.stdout).unwrap_or_default();

    Ok(raw_checks.into_iter().map(normalize_check).collect())
}

fn normalize_check(raw: RawCheck) -> Check {
    let status = match raw.bucket.as_deref() {
        Some("pass") => CheckStatus::Passed,
        Some("fail") => CheckStatus::Failed,
        Some("pending") => {
            if raw.state.as_deref() == Some("IN_PROGRESS") {
                CheckStatus::Running
            } else {
                CheckStatus::Queued
            }
        }
        Some("skipping") => CheckStatus::Skipped,
        Some("cancel") => CheckStatus::Cancelled,
        _ => CheckStatus::Unknown,
    };

    let duration_secs = match (&raw.started_at, &raw.completed_at) {
        (Some(start), Some(end)) => {
            let start_time = chrono::DateTime::parse_from_rfc3339(start).ok();
            let end_time = chrono::DateTime::parse_from_rfc3339(end).ok();
            match (start_time, end_time) {
                (Some(s), Some(e)) => Some((e - s).num_seconds().max(0) as u64),
                _ => None,
            }
        }
        _ => None,
    };

    Check {
        name: raw.name,
        status,
        conclusion: raw.conclusion,
        duration_secs,
        url: raw.details_url,
    }
}

/// Summary of check statuses
#[derive(Debug, Serialize, Clone)]
pub struct CheckSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub running: usize,
    pub queued: usize,
    pub skipped: usize,
    pub cancelled: usize,
    pub overall: CheckStatus,
}

impl CheckSummary {
    pub fn text(&self) -> String {
        if self.failed > 0 {
            format!("{} failed", self.failed)
        } else if self.running > 0 || self.queued > 0 {
            format!("{}/{} running", self.passed, self.total)
        } else if self.passed > 0 {
            format!("{}/{} passed", self.passed, self.total)
        } else {
            "no checks".to_string()
        }
    }
}

/// Summarize check results
pub fn summarize_checks(checks: &[Check]) -> CheckSummary {
    let mut summary = CheckSummary {
        total: checks.len(),
        passed: 0,
        failed: 0,
        running: 0,
        queued: 0,
        skipped: 0,
        cancelled: 0,
        overall: CheckStatus::Unknown,
    };

    for check in checks {
        match check.status {
            CheckStatus::Passed => summary.passed += 1,
            CheckStatus::Failed => summary.failed += 1,
            CheckStatus::Running => summary.running += 1,
            CheckStatus::Queued => summary.queued += 1,
            CheckStatus::Skipped => summary.skipped += 1,
            CheckStatus::Cancelled => summary.cancelled += 1,
            CheckStatus::Unknown => {}
        }
    }

    summary.overall = if summary.failed > 0 {
        CheckStatus::Failed
    } else if summary.running > 0 || summary.queued > 0 {
        CheckStatus::Running
    } else if summary.passed > 0 {
        CheckStatus::Passed
    } else {
        CheckStatus::Unknown
    };

    summary
}

/// Get PR URL for a branch
pub async fn get_pr_url(branch: &str) -> Option<String> {
    let output = Command::new("gh")
        .args(["pr", "view", branch, "--json", "url", "--jq", ".url"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        None
    } else {
        Some(url)
    }
}
