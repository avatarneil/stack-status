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
/// Example outputs from gt log short:
/// ```
/// ◉    branch-name
/// ◯    another-branch
/// │ ◯  side-branch (needs restack)
/// ◯─┘  main
/// ```
fn parse_gt_log_short(output: &str) -> Vec<BranchInfo> {
    let mut branches = Vec::new();
    let trunk_names = ["main", "master", "develop", "trunk"];

    // Characters used for tree drawing that should be stripped
    let tree_chars: &[char] = &['│', '─', '┘', '┐', '└', '┌', '├', '┤', '┬', '┴', '┼', ' '];

    for line in output.lines() {
        // Skip empty lines or lines that are just tree connectors
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if this line contains a branch indicator
        let has_current = line.contains('◉');
        let has_branch = line.contains('◉') || line.contains('◯');

        if !has_branch {
            continue;
        }

        // Find the position of the branch indicator and extract the name after it
        let branch_name = if let Some(pos) = line.find('◉').or_else(|| line.find('◯')) {
            // Get everything after the indicator
            let after_indicator = &line[pos + '◉'.len_utf8()..];
            // Strip tree drawing characters and whitespace
            after_indicator
                .trim_start_matches(tree_chars)
                .trim()
                .to_string()
        } else {
            continue;
        };

        if branch_name.is_empty() {
            continue;
        }

        // Check if this is a trunk branch
        let is_trunk = trunk_names.iter().any(|&t| branch_name == t);

        branches.push(BranchInfo {
            name: branch_name,
            is_current: has_current,
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
