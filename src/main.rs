mod display;
mod github;
mod graphite;
mod mcp;

use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use tokio::time::interval;

#[derive(Parser, Debug)]
#[command(name = "stack-status")]
#[command(about = "Display Graphite stack status with live CI check progress")]
#[command(version)]
struct Args {
    /// Watch mode: continuously refresh status
    #[arg(short, long)]
    watch: bool,

    /// Refresh interval in seconds (default: 10)
    #[arg(short, long, default_value = "10")]
    interval: u64,

    /// Show specific branch's stack (default: current branch)
    #[arg(short, long)]
    branch: Option<String>,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Run as MCP server (stdio transport)
    #[arg(long)]
    mcp: bool,

    /// Show detailed check information
    #[arg(short, long)]
    details: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // MCP server mode
    if args.mcp {
        return mcp::run_server().await;
    }

    // Check prerequisites
    let has_gt = graphite::is_installed().await;
    let has_gh = github::is_installed().await;

    if !has_gh {
        eprintln!("Warning: GitHub CLI (gh) not found. Install from https://cli.github.com/");
        eprintln!("         CI status checks will not be available.");
    }

    if !has_gt {
        eprintln!("Warning: Graphite CLI (gt) not found. Install from https://graphite.dev/");
        eprintln!("         Showing current branch only (no stack hierarchy).");
    }

    // Single run or watch mode
    if args.watch {
        run_watch_mode(&args, has_gt, has_gh).await
    } else {
        run_once(&args, has_gt, has_gh).await
    }
}

async fn run_once(args: &Args, has_gt: bool, has_gh: bool) -> Result<()> {
    let status = fetch_status(args, has_gt, has_gh).await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        display::render(&status, args.details);
    }

    Ok(())
}

async fn run_watch_mode(args: &Args, has_gt: bool, has_gh: bool) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(args.interval));

    // Set up terminal for raw mode to capture key presses
    display::setup_terminal()?;

    loop {
        ticker.tick().await;

        let status = fetch_status(args, has_gt, has_gh).await?;

        // Clear screen and render
        display::clear_screen();

        if args.json {
            println!("{}", serde_json::to_string_pretty(&status)?);
        } else {
            display::render(&status, args.details);
            display::render_help_bar();
        }

        // Check for key press (non-blocking)
        if let Some(key) = display::check_keypress() {
            match key {
                'q' => break,
                'r' => continue, // Force refresh
                _ => {}
            }
        }

        // Check if all checks are complete (exit watch mode)
        if status.all_complete() {
            display::render_complete_message();
            break;
        }
    }

    display::restore_terminal()?;
    Ok(())
}

async fn fetch_status(_args: &Args, has_gt: bool, has_gh: bool) -> Result<StackStatus> {
    let mut status = StackStatus::new();

    // Get stack from Graphite or fall back to current branch
    let branches = if has_gt {
        graphite::get_stack().await?
    } else {
        // Fall back to current branch only
        let current = graphite::get_current_branch().await?;
        vec![BranchInfo {
            name: current,
            is_current: true,
            is_trunk: false,
        }]
    };

    // Get PR and check status for each branch
    for branch in branches {
        if branch.is_trunk {
            status.branches.push(BranchStatus {
                branch: branch.name,
                is_current: branch.is_current,
                is_trunk: true,
                pr: None,
                checks: None,
                summary: None,
            });
            continue;
        }

        let (pr, checks) = if has_gh {
            let pr = github::get_pr_for_branch(&branch.name).await;
            let checks = if pr.is_some() {
                Some(github::get_checks(&branch.name).await?)
            } else {
                None
            };
            (pr, checks)
        } else {
            (None, None)
        };

        let summary = checks.as_ref().map(|c| github::summarize_checks(c));

        status.branches.push(BranchStatus {
            branch: branch.name,
            is_current: branch.is_current,
            is_trunk: false,
            pr,
            checks,
            summary,
        });
    }

    status.timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    Ok(status)
}

#[derive(Debug, serde::Serialize)]
pub struct StackStatus {
    pub branches: Vec<BranchStatus>,
    pub timestamp: String,
}

impl StackStatus {
    fn new() -> Self {
        Self {
            branches: Vec::new(),
            timestamp: String::new(),
        }
    }

    fn all_complete(&self) -> bool {
        self.branches.iter().all(|b| {
            b.is_trunk
                || b.summary
                    .as_ref()
                    .map(|s| s.running == 0 && s.queued == 0)
                    .unwrap_or(true)
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct BranchStatus {
    pub branch: String,
    pub is_current: bool,
    pub is_trunk: bool,
    pub pr: Option<u64>,
    pub checks: Option<Vec<github::Check>>,
    pub summary: Option<github::CheckSummary>,
}

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_trunk: bool,
}
