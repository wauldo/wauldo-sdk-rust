//! Task planning methods for AgentClient

use std::sync::OnceLock;

use regex::Regex;
use serde_json::json;

use crate::error::{Error, Result};
use crate::types::*;

use super::AgentClient;

/// Compiled regex for step detection — anchored to avoid false positives
fn step_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(\d+)\.\s+(.+)$").expect("valid regex"))
}

impl AgentClient {
    /// Plan a task
    pub async fn plan_task(&mut self, task: &str) -> Result<PlanResult> {
        self.plan_task_with_options(task, PlanOptions::default())
            .await
    }

    /// Plan a task with options
    pub async fn plan_task_with_options(
        &mut self,
        task: &str,
        options: PlanOptions,
    ) -> Result<PlanResult> {
        if task.trim().is_empty() {
            return Err(Error::validation_field("Task cannot be empty", "task"));
        }

        let max_steps = options.max_steps.unwrap_or(10);
        if !(1..=20).contains(&max_steps) {
            return Err(Error::validation_field(
                "max_steps must be between 1 and 20",
                "max_steps",
            ));
        }

        let detail_level = match options.detail_level.unwrap_or_default() {
            DetailLevel::Brief => "brief",
            DetailLevel::Normal => "normal",
            DetailLevel::Detailed => "detailed",
        };

        let content = self
            .call_tool(
                "plan_task",
                json!({
                    "task": task,
                    "context": options.context.unwrap_or_default(),
                    "max_steps": max_steps,
                    "detail_level": detail_level
                }),
            )
            .await?;

        Ok(self.parse_plan_result(&content, task))
    }

    pub(crate) fn parse_plan_result(&self, content: &str, task: &str) -> PlanResult {
        // Try JSON first (structured output from server v0.2+)
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(steps_arr) = data.get("steps").and_then(|s| s.as_array()) {
                let steps = steps_arr
                    .iter()
                    .map(|s| PlanStep {
                        number: s["number"].as_u64().unwrap_or(0) as usize,
                        title: s["title"].as_str().unwrap_or("").to_string(),
                        description: s["description"].as_str().unwrap_or("").to_string(),
                        priority: s["priority"].as_str().unwrap_or("Medium").to_string(),
                        effort: s["effort"].as_str().unwrap_or("").to_string(),
                        dependencies: s["dependencies"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                    })
                    .collect();
                return PlanResult {
                    task: data["task"].as_str().unwrap_or(task).to_string(),
                    category: data["category"].as_str().unwrap_or("General").to_string(),
                    steps,
                    total_effort: data["total_effort"].as_str().unwrap_or("").to_string(),
                    raw_content: content.to_string(),
                };
            }
        }

        // Fallback: markdown heuristic parser (server v0.1.x)
        let mut steps = Vec::new();
        let mut category = "General".to_string();
        let mut total_effort = String::new();
        let mut current_step = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Extract category: "**Category**: X" → take everything after the first ':'
            if let Some(rest) = trimmed.strip_prefix("**Category**:") {
                category = rest.trim().to_string();
                if category.is_empty() {
                    category = "General".to_string();
                }
                continue;
            }

            // Match numbered steps strictly: "1. Title" via anchored regex
            if let Some(caps) = step_pattern().captures(trimmed) {
                current_step += 1;
                let title = caps[2].trim().to_string();
                if !title.is_empty() {
                    steps.push(PlanStep {
                        number: current_step,
                        title,
                        description: String::new(),
                        priority: "Medium".to_string(),
                        effort: String::new(),
                        dependencies: vec![],
                    });
                }
                continue;
            }

            // Extract effort
            if let Some(rest) = trimmed.strip_prefix("**Estimated total effort**:") {
                total_effort = rest.trim().to_string();
            }
        }

        PlanResult {
            task: task.to_string(),
            category,
            steps,
            total_effort,
            raw_content: content.to_string(),
        }
    }
}
