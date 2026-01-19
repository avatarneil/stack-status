use crate::StackStatus;
use anyhow::Result;
use std::io::{self, Write};

// ANSI escape codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
#[allow(dead_code)]
const RED: &str = "\x1b[31m";
#[allow(dead_code)]
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GRAY: &str = "\x1b[90m";
const CYAN: &str = "\x1b[36m";

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().ok();
}

/// Set up terminal for raw input (non-blocking key detection)
pub fn setup_terminal() -> Result<()> {
    // On Unix, we'd use termios to set raw mode
    // For simplicity, we'll just work in cooked mode with non-blocking reads
    Ok(())
}

/// Restore terminal to normal mode
pub fn restore_terminal() -> Result<()> {
    Ok(())
}

/// Check for keypress (non-blocking)
pub fn check_keypress() -> Option<char> {
    // In a full implementation, this would use termios or crossterm
    // For now, return None (no key detected)
    None
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Render the stack status to terminal
pub fn render(status: &StackStatus, show_details: bool) {
    // Header
    println!(
        "{}╭─────────────────────────────────────────────────────────╮{}",
        DIM, RESET
    );
    println!(
        "{}│{}  {}Stack Status{}                         Updated: {}{}  {}│{}",
        DIM, RESET, BOLD, RESET, CYAN, status.timestamp, DIM, RESET
    );
    println!(
        "{}╰─────────────────────────────────────────────────────────╯{}",
        DIM, RESET
    );
    println!();

    // Render each branch
    for (i, branch) in status.branches.iter().enumerate() {
        let is_last = i == status.branches.len() - 1;

        // Branch indicator
        let indicator = if branch.is_trunk {
            format!("{}●{}", GRAY, RESET)
        } else if branch.is_current {
            format!("{}◉{}", BLUE, RESET)
        } else {
            format!("{}◯{}", DIM, RESET)
        };

        // PR number
        let pr_str = branch
            .pr
            .map(|n| format!(" {}(#{}){}", DIM, n, RESET))
            .unwrap_or_default();

        // Status summary
        let summary_str = if let Some(ref summary) = branch.summary {
            let color = summary.overall.color_code();
            let icon = summary.overall.icon();
            format!(
                "  {}{} {}{}",
                color,
                icon,
                summary.text(),
                RESET
            )
        } else if branch.is_trunk {
            String::new()
        } else {
            format!("  {}no PR{}", DIM, RESET)
        };

        // Print branch line
        println!(
            "{} {}{}{}{}",
            indicator,
            if branch.is_current { BOLD } else { "" },
            branch.branch,
            if branch.is_current { RESET } else { "" },
            pr_str,
        );

        // Print summary on same line (right-aligned would be nice but keeping it simple)
        if !summary_str.is_empty() {
            println!("  {}", summary_str.trim());
        }

        // Show detailed checks if requested
        if show_details && !branch.is_trunk {
            if let Some(ref checks) = branch.checks {
                for check in checks {
                    let color = check.status.color_code();
                    let icon = check.status.icon();
                    let duration = check
                        .duration_secs
                        .map(|d| format!(" ({})", format_duration(d)))
                        .unwrap_or_default();

                    println!(
                        "    {}├─{} {}{} {}{}{}",
                        DIM, RESET, color, icon, check.name, duration, RESET
                    );
                }
            }
        }

        // Connector line (except for last item)
        if !is_last {
            println!("{}│{}", DIM, RESET);
        }
    }

    println!();
}

/// Render the help bar for watch mode
pub fn render_help_bar() {
    println!(
        "{}─────────────────────────────────────────────────────────{}",
        DIM, RESET
    );
    println!(
        "{}[q]{} quit  {}[r]{} refresh  {}[d]{} details",
        BOLD, RESET, BOLD, RESET, BOLD, RESET
    );
}

/// Render completion message
pub fn render_complete_message() {
    println!();
    println!(
        "{}All checks complete. Exiting watch mode.{}",
        GREEN, RESET
    );
}

/// Render a simple spinner frame
#[allow(dead_code)]
pub fn spinner_frame(frame: usize) -> char {
    const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    FRAMES[frame % FRAMES.len()]
}
