//! Tree-of-Thought reasoning methods for AgentClient

use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

use super::AgentClient;

impl AgentClient {
    /// Perform Tree-of-Thought reasoning
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use wauldo::{AgentClient, ReasoningOptions};
    /// # async fn example() -> wauldo::Result<()> {
    /// let mut client = AgentClient::new().connect().await?;
    /// let result = client
    ///     .reason_with_options(
    ///         "What's the best sorting algorithm?",
    ///         ReasoningOptions::new().depth(4).branches(3)
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reason(&mut self, problem: &str) -> Result<ReasoningResult> {
        self.reason_with_options(problem, ReasoningOptions::default())
            .await
    }

    /// Perform reasoning with custom options
    pub async fn reason_with_options(
        &mut self,
        problem: &str,
        options: ReasoningOptions,
    ) -> Result<ReasoningResult> {
        if problem.trim().is_empty() {
            return Err(Error::validation_field(
                "Problem cannot be empty",
                "problem",
            ));
        }

        let depth = options.depth.unwrap_or(3);
        let branches = options.branches.unwrap_or(3);

        if !(1..=10).contains(&depth) {
            return Err(Error::validation_field(
                "Depth must be between 1 and 10",
                "depth",
            ));
        }
        if !(1..=10).contains(&branches) {
            return Err(Error::validation_field(
                "Branches must be between 1 and 10",
                "branches",
            ));
        }

        let content = self
            .call_tool(
                "reason_tree_of_thought",
                json!({
                    "problem": problem,
                    "depth": depth,
                    "branches": branches
                }),
            )
            .await?;

        Ok(self.parse_reasoning_result(&content, problem, depth, branches))
    }

    pub(crate) fn parse_reasoning_result(
        &self,
        content: &str,
        problem: &str,
        depth: usize,
        branches: usize,
    ) -> ReasoningResult {
        // Try JSON first (structured output from server v0.2+)
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(solution) = data.get("solution").and_then(|s| s.as_str()) {
                return ReasoningResult {
                    problem: data["problem"].as_str().unwrap_or(problem).to_string(),
                    solution: solution.to_string(),
                    thought_tree: data["thought_tree"].as_str().unwrap_or(content).to_string(),
                    depth: data["depth"].as_u64().unwrap_or(depth as u64) as usize,
                    branches: data["branches"].as_u64().unwrap_or(branches as u64) as usize,
                    raw_content: content.to_string(),
                };
            }
        }

        // Fallback: markdown heuristic parser (server v0.1.x)
        let mut solution = String::new();
        let mut in_solution = false;

        for line in content.lines() {
            if line.contains("Solution:") || line.contains("Best path:") {
                in_solution = true;
                continue;
            }
            if in_solution && !line.trim().is_empty() {
                solution = line.trim().to_string();
                break;
            }
        }

        ReasoningResult {
            problem: problem.to_string(),
            solution: if solution.is_empty() {
                "See thought tree for analysis".to_string()
            } else {
                solution
            },
            thought_tree: content.to_string(),
            depth,
            branches,
            raw_content: content.to_string(),
        }
    }
}
