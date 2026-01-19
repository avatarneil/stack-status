use crate::BranchInfo;
use anyhow::Result;
use tokio::process::Command;

/// Check if Graphite CLI (gt) is installed
pub async fn is_installed() -> bool {
    Command::new("gt")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get current git branch
pub async fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get stack information from Graphite CLI
/// Returns branches from top of stack to trunk
pub async fn get_stack() -> Result<Vec<BranchInfo>> {
    let output = Command::new("gt")
        .args(["log", "short"])
        .output()
        .await?;

    if !output.status.success() {
        // Fall back to current branch
        let current = get_current_branch().await?;
        return Ok(vec![BranchInfo {
            name: current,
            is_current: true,
            is_trunk: false,
        }]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_gt_log_short(&stdout))
}

/// Parse gt log short output into structured data
///
/// Example output:
/// ```
/// ◉ feature-c
/// │
/// ◯ feature-b
/// │
/// ◯ feature-a
/// │
/// ◯ main
/// ```
fn parse_gt_log_short(output: &str) -> Vec<BranchInfo> {
    let mut branches = Vec::new();
    let trunk_names = ["main", "master", "develop", "trunk"];

    for line in output.lines() {
        // Skip connector lines (just │)
        if line.trim() == "│" || line.trim().is_empty() {
            continue;
        }

        // Match lines with branch indicators
        // ◉ = current branch, ◯ = other branch, ● = trunk indicator
        let is_current = line.contains('◉');

        // Extract branch name (after the indicator character)
        let branch_name = line
            .trim()
            .trim_start_matches(|c| c == '◉' || c == '◯' || c == '●' || c == '│' || c == ' ')
            .trim()
            .to_string();

        if branch_name.is_empty() {
            continue;
        }

        let is_trunk = trunk_names.iter().any(|&t| branch_name == t);

        branches.push(BranchInfo {
            name: branch_name,
            is_current,
            is_trunk,
        });
    }

    branches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gt_log_short() {
        let output = r#"
◉ feature-c
│
◯ feature-b
│
◯ feature-a
│
◯ main
"#;
        let branches = parse_gt_log_short(output);
        assert_eq!(branches.len(), 4);
        assert_eq!(branches[0].name, "feature-c");
        assert!(branches[0].is_current);
        assert!(!branches[0].is_trunk);
        assert_eq!(branches[3].name, "main");
        assert!(branches[3].is_trunk);
    }
}
