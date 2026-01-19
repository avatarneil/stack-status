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
const MAGENTA: &str = "\x1b[35m";

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
    // In a full implementation, this would use termios or crossterm
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
    let width = 60;

    // Header box
    println!(
        "{}╭{}╮{}",
        DIM,
        BOX_H.repeat(width - 2),
        RESET
    );
    println!(
        "{}│{}  {}Stack Status{}{}Updated: {}{}{}│{}",
        DIM,
        RESET,
        BOLD,
        RESET,
        " ".repeat(width - 38),
        CYAN,
        status.timestamp,
        DIM,
        RESET
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

        // PR number
        let pr_str = branch
            .pr
            .map(|n| format!(" {}(#{}){}", DIM, n, RESET))
            .unwrap_or_default();

        // Overall status indicator (animated for running)
        let status_str = if let Some(ref summary) = branch.summary {
            match summary.overall {
                CheckStatus::Running => {
                    let spin = progress_spinner(frame);
                    format!("{}{}  Running{}", YELLOW, spin, RESET)
                }
                CheckStatus::Queued => {
                    format!("{}○  Queued{}", GRAY, RESET)
                }
                CheckStatus::Passed => {
                    format!("{}✓  Passed{}", GREEN, RESET)
                }
                CheckStatus::Failed => {
                    format!("{}✗  Failed{}", RED, RESET)
                }
                _ => {
                    format!("{}{}{}", DIM, summary.text(), RESET)
                }
            }
        } else if branch.is_trunk {
            String::new()
        } else {
            format!("{}—  no PR{}", DIM, RESET)
        };

        // Print branch header line
        let branch_display = if branch.branch.len() > 35 {
            format!("{}...", &branch.branch[..32])
        } else {
            branch.branch.clone()
        };

        print!(
            "{}{}{} {}{}{}{}",
            indicator_color,
            indicator,
            RESET,
            if branch.is_current { BOLD } else { "" },
            branch_display,
            if branch.is_current { RESET } else { "" },
            pr_str,
        );

        // Right-align status
        let used_width = 2 + branch_display.len() + pr_str.len() / 3; // rough estimate accounting for ANSI
        let padding = if used_width < 45 { 45 - used_width } else { 2 };
        println!("{}{}", " ".repeat(padding), status_str);

        // Show detailed checks in a nice box
        if show_details && !branch.is_trunk && branch.checks.is_some() {
            if let Some(ref checks) = branch.checks {
                if !checks.is_empty() {
                    let box_width = 54;

                    // Top border
                    println!(
                        "  {}{}{}{}{}",
                        DIM, BOX_TL, BOX_H.repeat(box_width - 2), BOX_TR, RESET
                    );

                    for check in checks {
                        let (icon, color) = match check.status {
                            CheckStatus::Passed => ("✓", GREEN),
                            CheckStatus::Failed => ("✗", RED),
                            CheckStatus::Running => (spinner(frame), YELLOW),
                            CheckStatus::Queued => ("○", GRAY),
                            CheckStatus::Skipped => ("○", GRAY),
                            CheckStatus::Cancelled => ("⊘", GRAY),
                            CheckStatus::Unknown => ("?", GRAY),
                        };

                        // Truncate name if needed
                        let name = if check.name.len() > 28 {
                            format!("{}...", &check.name[..25])
                        } else {
                            check.name.clone()
                        };

                        // Duration or progress indicator
                        let right_info = match check.status {
                            CheckStatus::Passed | CheckStatus::Failed => {
                                check.duration_secs
                                    .map(|d| format!("{:>6}", format_duration(d)))
                                    .unwrap_or_else(|| "      ".to_string())
                            }
                            CheckStatus::Running => {
                                // Show animated progress
                                format!("{}..{}", YELLOW, RESET)
                            }
                            _ => "      ".to_string()
                        };

                        // Status text
                        let status_text = match check.status {
                            CheckStatus::Passed => format!("{}done{}", GREEN, RESET),
                            CheckStatus::Failed => format!("{}fail{}", RED, RESET),
                            CheckStatus::Running => format!("{}run {}", YELLOW, RESET),
                            CheckStatus::Queued => format!("{}wait{}", GRAY, RESET),
                            CheckStatus::Skipped => format!("{}skip{}", GRAY, RESET),
                            CheckStatus::Cancelled => format!("{}stop{}", GRAY, RESET),
                            CheckStatus::Unknown => format!("{}????{}", GRAY, RESET),
                        };

                        println!(
                            "  {}{}{} {}{} {:<28}  {:>6}  {} {}{}{}",
                            DIM, BOX_V, RESET,
                            color, icon,
                            name,
                            right_info,
                            status_text,
                            DIM, BOX_V, RESET
                        );
                    }

                    // Progress summary bar
                    if let Some(ref summary) = branch.summary {
                        let completed = summary.passed + summary.failed + summary.skipped + summary.cancelled;
                        let total = summary.total;

                        if total > 0 && (summary.running > 0 || summary.queued > 0) {
                            println!(
                                "  {}{}{}{}{}",
                                DIM, BOX_V, RESET,
                                " ".repeat(box_width - 2),
                                format!("{}{}{}", DIM, BOX_V, RESET)
                            );
                            println!(
                                "  {}{}{} {}  {}/{}  {}",
                                DIM, BOX_V, RESET,
                                render_progress_bar(completed, total, 30),
                                completed,
                                total,
                                format!("{}{}{}", DIM, BOX_V, RESET)
                            );
                        }
                    }

                    // Bottom border
                    println!(
                        "  {}{}{}{}{}",
                        DIM, BOX_BL, BOX_H.repeat(box_width - 2), BOX_BR, RESET
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
        "{}{}{}",
        DIM,
        "─".repeat(60),
        RESET
    );
    println!(
        "  {}[q]{} quit   {}[r]{} refresh   {}[d]{} toggle details   {}[o]{} open PR",
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
