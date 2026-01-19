use crate::github::CheckStatus;
use crate::StackStatus;
use anyhow::Result;
use std::io::{self, Write};

// ANSI escape codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GRAY: &str = "\x1b[90m";
const CYAN: &str = "\x1b[36m";

// Box drawing characters
const BOX_TL: &str = "┌";
const BOX_TR: &str = "┐";
const BOX_BL: &str = "└";
const BOX_BR: &str = "┘";
const BOX_H: &str = "─";
const BOX_V: &str = "│";

// Progress bar characters
const PROG_FULL: &str = "█";
const PROG_EMPTY: &str = "░";

// Spinner frames for animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const PROGRESS_SPINNER: &[&str] = &["◐", "◓", "◑", "◒"];

/// Get terminal size (width, height)
fn get_terminal_size() -> (usize, usize) {
    // Try to get from environment or use sensible defaults
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120);
    let height = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);
    (width.max(60), height.max(20))
}

/// Clear the terminal screen
pub fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().ok();
}

/// Hide cursor
pub fn hide_cursor() {
    print!("\x1b[?25l");
    io::stdout().flush().ok();
}

/// Show cursor
pub fn show_cursor() {
    print!("\x1b[?25h");
    io::stdout().flush().ok();
}

/// Set up terminal for watch mode
pub fn setup_terminal() -> Result<()> {
    hide_cursor();
    Ok(())
}

/// Restore terminal to normal mode
pub fn restore_terminal() -> Result<()> {
    show_cursor();
    Ok(())
}

/// Check for keypress (non-blocking)
pub fn check_keypress() -> Option<char> {
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

/// Get spinner character for current frame
fn spinner(frame: usize) -> &'static str {
    SPINNER_FRAMES[frame % SPINNER_FRAMES.len()]
}

/// Get progress spinner for current frame
fn progress_spinner(frame: usize) -> &'static str {
    PROGRESS_SPINNER[frame % PROGRESS_SPINNER.len()]
}

/// Render a progress bar
fn render_progress_bar(completed: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("{}{}{}", DIM, "░".repeat(width), RESET);
    }

    let filled = (completed * width) / total;
    let empty = width - filled;

    format!(
        "{}{}{}{}{}",
        CYAN,
        PROG_FULL.repeat(filled),
        DIM,
        PROG_EMPTY.repeat(empty),
        RESET
    )
}

/// Render simple mode (non-watch, no animation)
pub fn render(status: &StackStatus, show_details: bool) {
    render_with_frame(status, show_details, 0);
}

/// Render with animation frame for watch mode
pub fn render_with_frame(status: &StackStatus, show_details: bool, frame: usize) {
    let (term_width, _term_height) = get_terminal_size();
    let width = term_width.min(100).max(60);
    let box_width = (width - 6).min(80);
    let name_width = (width - 30).min(50).max(25);

    // Header box
    println!(
        "{}╭{}╮{}",
        DIM,
        BOX_H.repeat(width - 2),
        RESET
    );

    let title = "Stack Status";
    let time_str = format!("Updated: {}", status.timestamp);
    let padding = width - 4 - title.len() - time_str.len();
    println!(
        "{}│{} {}{}{}{}{}{} {}│{}",
        DIM, RESET,
        BOLD, title, RESET,
        " ".repeat(padding),
        CYAN, time_str,
        DIM, RESET
    );
    println!(
        "{}╰{}╯{}",
        DIM,
        BOX_H.repeat(width - 2),
        RESET
    );
    println!();

    // Render each branch
    for (i, branch) in status.branches.iter().enumerate() {
        let is_last = i == status.branches.len() - 1;

        // Branch indicator with color
        let (indicator, indicator_color) = if branch.is_trunk {
            ("●", GRAY)
        } else if branch.is_current {
            ("◉", BLUE)
        } else {
            ("◯", DIM)
        };

        // PR number and link hint
        let pr_info = branch
            .pr
            .map(|n| format!(" {}#{}{}", CYAN, n, RESET))
            .unwrap_or_default();

        // Overall status indicator (animated for running)
        let status_str = if let Some(ref summary) = branch.summary {
            match summary.overall {
                CheckStatus::Running => {
                    let spin = progress_spinner(frame);
                    format!(
                        "{}{} {} Running ({}/{}){}",
                        YELLOW, spin, spin,
                        summary.passed + summary.failed,
                        summary.total,
                        RESET
                    )
                }
                CheckStatus::Queued => {
                    format!("{}○ ○ Queued{}", GRAY, RESET)
                }
                CheckStatus::Passed => {
                    format!("{}✓ ✓ All {} passed{}", GREEN, summary.total, RESET)
                }
                CheckStatus::Failed => {
                    format!(
                        "{}✗ ✗ {} failed{}, {}{} passed{}",
                        RED, summary.failed, RESET,
                        GREEN, summary.passed, RESET
                    )
                }
                _ => {
                    format!("{}{}{}", DIM, summary.text(), RESET)
                }
            }
        } else if branch.is_trunk {
            String::new()
        } else {
            format!("{}— No PR{}", DIM, RESET)
        };

        // Full branch name (or truncate if really long)
        let branch_display = if branch.branch.len() > name_width {
            format!("{}…", &branch.branch[..name_width - 1])
        } else {
            branch.branch.clone()
        };

        // Print branch line
        println!(
            "{}{}{} {}{}{}{}",
            indicator_color,
            indicator,
            RESET,
            if branch.is_current { BOLD } else { "" },
            branch_display,
            if branch.is_current { RESET } else { "" },
            pr_info,
        );

        // Status on next line, indented
        if !status_str.is_empty() {
            println!("    {}", status_str);
        }

        // Always show checks if we have them (details mode shows more info per check)
        if !branch.is_trunk && branch.checks.is_some() {
            if let Some(ref checks) = branch.checks {
                if !checks.is_empty() {
                    println!();

                    // Top border
                    println!(
                        "    {}{}{}{}{}",
                        DIM, BOX_TL, BOX_H.repeat(box_width - 2), BOX_TR, RESET
                    );

                    for check in checks {
                        let (icon, color) = match check.status {
                            CheckStatus::Passed => ("✓", GREEN),
                            CheckStatus::Failed => ("✗", RED),
                            CheckStatus::Running => (spinner(frame), YELLOW),
                            CheckStatus::Queued => ("○", GRAY),
                            CheckStatus::Skipped => ("◌", GRAY),
                            CheckStatus::Cancelled => ("⊘", GRAY),
                            CheckStatus::Unknown => ("?", GRAY),
                        };

                        // Check name - use more space
                        let check_name_width = box_width - 25;
                        let name = if check.name.len() > check_name_width {
                            format!("{}…", &check.name[..check_name_width - 1])
                        } else {
                            check.name.clone()
                        };

                        // Duration or status indicator
                        let timing = match check.status {
                            CheckStatus::Passed | CheckStatus::Failed => {
                                check.duration_secs
                                    .map(|d| format_duration(d))
                                    .unwrap_or_else(|| "—".to_string())
                            }
                            CheckStatus::Running => {
                                "running…".to_string()
                            }
                            CheckStatus::Queued => "queued".to_string(),
                            CheckStatus::Skipped => "skipped".to_string(),
                            CheckStatus::Cancelled => "cancelled".to_string(),
                            CheckStatus::Unknown => "—".to_string(),
                        };

                        // Status label
                        let status_label = match check.status {
                            CheckStatus::Passed => format!("{}passed{}", GREEN, RESET),
                            CheckStatus::Failed => format!("{}FAILED{}", RED, RESET),
                            CheckStatus::Running => format!("{}running{}", YELLOW, RESET),
                            CheckStatus::Queued => format!("{}queued{}", GRAY, RESET),
                            CheckStatus::Skipped => format!("{}skipped{}", GRAY, RESET),
                            CheckStatus::Cancelled => format!("{}stopped{}", GRAY, RESET),
                            CheckStatus::Unknown => format!("{}unknown{}", GRAY, RESET),
                        };

                        // Show URL hint in details mode
                        let url_hint = if show_details && check.url.is_some() {
                            format!(" {}↗{}", DIM, RESET)
                        } else {
                            String::new()
                        };

                        println!(
                            "    {}{}{} {}{} {:<width$} {:>10}  {}{}  {}{}{}",
                            DIM, BOX_V, RESET,
                            color, icon,
                            name,
                            timing,
                            status_label,
                            url_hint,
                            DIM, BOX_V, RESET,
                            width = check_name_width,
                        );
                    }

                    // Progress bar for in-progress checks
                    if let Some(ref summary) = branch.summary {
                        if summary.running > 0 || summary.queued > 0 {
                            let completed = summary.passed + summary.failed + summary.skipped + summary.cancelled;
                            let total = summary.total;
                            let bar_width = (box_width - 20).min(40);

                            println!(
                                "    {}{}{}{}{}",
                                DIM, BOX_V, RESET,
                                " ".repeat(box_width - 2),
                                format!("{}{}{}", DIM, BOX_V, RESET)
                            );
                            let padding = if box_width > bar_width + 22 {
                                " ".repeat(box_width - bar_width - 22)
                            } else {
                                String::new()
                            };
                            println!(
                                "    {}{}{} {} {}/{} complete {}{}{}{}",
                                DIM, BOX_V, RESET,
                                render_progress_bar(completed, total, bar_width),
                                completed, total,
                                padding,
                                DIM, BOX_V, RESET
                            );
                        }
                    }

                    // Bottom border
                    println!(
                        "    {}{}{}{}{}",
                        DIM, BOX_BL, BOX_H.repeat(box_width - 2), BOX_BR, RESET
                    );
                }
            }
        }

        // Connector line (except for last item)
        if !is_last {
            println!("{}  │{}", DIM, RESET);
        }
    }

    println!();
}

/// Render the help bar for watch mode
pub fn render_help_bar() {
    let (width, _) = get_terminal_size();
    let bar_width = width.min(100);

    println!(
        "{}{}{}",
        DIM,
        "─".repeat(bar_width),
        RESET
    );
    println!(
        "  {}q{} quit   {}r{} refresh   {}d{} details   {}Ctrl+C{} exit",
        BOLD, RESET, BOLD, RESET, BOLD, RESET, BOLD, RESET
    );
}

/// Render completion message
pub fn render_complete_message() {
    println!();
    println!(
        "  {}✓ All checks complete!{}",
        GREEN, RESET
    );
}
