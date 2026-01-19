use crate::{github, graphite, BranchInfo, BranchStatus, StackStatus};
use anyhow::Result;
use std::future::Future;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ServerHandler, ServiceExt,
    transport::stdio,
};
use serde::Deserialize;

/// MCP Server for stack status
#[derive(Clone)]
pub struct StackStatusService {
    tool_router: ToolRouter<Self>,
}

impl StackStatusService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

/// Request for getting checks for a specific branch
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetBranchChecksRequest {
    /// The branch name to get checks for
    pub branch: String,
}

#[tool_router]
impl StackStatusService {
    /// Get the full Graphite stack status including CI check progress for all PRs
    #[tool(description = "Get the current Graphite stack status including CI check progress for all PRs in the stack")]
    async fn get_stack_status(&self) -> Result<CallToolResult, ErrorData> {
        let status = fetch_stack_status().await.map_err(|e| {
            ErrorData::new(ErrorCode(-32000), e.to_string(), None)
        })?;

        let json = serde_json::to_string_pretty(&status).map_err(|e| {
            ErrorData::new(ErrorCode(-32000), e.to_string(), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get detailed CI check status for a specific branch
    #[tool(description = "Get detailed CI check status for a specific branch or PR")]
    async fn get_pr_checks(
        &self,
        Parameters(req): Parameters<GetBranchChecksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        if !github::is_installed().await {
            return Ok(CallToolResult::success(vec![Content::text(
                r#"{"error": "GitHub CLI (gh) not installed"}"#,
            )]));
        }

        let pr = github::get_pr_for_branch(&req.branch).await;
        let pr_url = github::get_pr_url(&req.branch).await;
        let checks = github::get_checks(&req.branch).await.map_err(|e| {
            ErrorData::new(ErrorCode(-32000), e.to_string(), None)
        })?;
        let summary = github::summarize_checks(&checks);

        let result = serde_json::json!({
            "branch": req.branch,
            "pr": pr,
            "pr_url": pr_url,
            "checks": checks,
            "summary": summary
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    /// Get information about the current git branch
    #[tool(description = "Get information about the current git branch including PR status")]
    async fn get_branch_info(&self) -> Result<CallToolResult, ErrorData> {
        let current = graphite::get_current_branch().await.map_err(|e| {
            ErrorData::new(ErrorCode(-32000), e.to_string(), None)
        })?;

        let has_gt = graphite::is_installed().await;
        let has_gh = github::is_installed().await;

        let pr = if has_gh {
            github::get_pr_for_branch(&current).await
        } else {
            None
        };

        let pr_url = if has_gh {
            github::get_pr_url(&current).await
        } else {
            None
        };

        let result = serde_json::json!({
            "branch": current,
            "pr": pr,
            "pr_url": pr_url,
            "graphite_installed": has_gt,
            "github_cli_installed": has_gh
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for StackStatusService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "stack-status".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Get Graphite stack status and CI check progress. Use get_stack_status for full stack view, get_pr_checks for specific branch details.".to_string()
            ),
        }
    }
}

/// Run the MCP server using stdio transport
pub async fn run_server() -> Result<()> {
    let service = StackStatusService::new();
    let server = service.serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}

/// Fetch complete stack status (shared with CLI)
async fn fetch_stack_status() -> Result<StackStatus> {
    let has_gt = graphite::is_installed().await;
    let has_gh = github::is_installed().await;

    let mut status = StackStatus {
        branches: Vec::new(),
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
    };

    // Get stack from Graphite or fall back to current branch
    let branches = if has_gt {
        graphite::get_stack().await?
    } else {
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

    Ok(status)
}
