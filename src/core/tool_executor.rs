use std::sync::Arc;

use crate::core::conversation_engine::ProcessedResponse;
use crate::core::retry_handler::RetryableError;
use crate::tools::{ToolCall, ToolRegistry, ToolResult};

/// 工具执行错误
#[derive(Debug)]
pub enum ToolExecutionError {
    MissingRegistry,
    ToolFailure(String),
}

impl std::fmt::Display for ToolExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolExecutionError::MissingRegistry => write!(f, "tool registry not configured"),
            ToolExecutionError::ToolFailure(err) => write!(f, "tool execution failed: {}", err),
        }
    }
}

impl std::error::Error for ToolExecutionError {}

impl RetryableError for ToolExecutionError {
    fn retryable(&self) -> bool {
        matches!(self, ToolExecutionError::ToolFailure(_))
    }
}

/// 工具执行器：封装 ToolRegistry，处理递归调用
#[derive(Clone)]
pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
}

impl ToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.has_tool(name)
    }

    /// 执行一批工具调用并返回结果
    pub async fn execute_calls(&self, calls: Vec<ToolCall>) -> Result<Vec<ToolResult>, ToolExecutionError> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            let result = self.registry.execute(call).await;
            if result.success {
                results.push(result);
            } else {
                return Err(ToolExecutionError::ToolFailure(result.error.unwrap_or_default()));
            }
        }

        Ok(results)
    }

    /// 根据工具结果构造用于反馈给 LLM 的字符串
    pub fn format_tool_results(&self, results: &[ToolResult]) -> String {
        let mut formatted = String::new();
        for result in results {
            let name = result
                .data
                .get("tool")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_tool");
            formatted.push_str(&format!("\n<tool_result name=\"{}\">\n{}\n</tool_result>\n", name, result.data));
        }
        formatted
    }

    /// 递归执行工具调用：如果新的 AI 响应继续触发工具调用，则继续循环
    pub async fn execute_recursive<F, Fut>(
        &self,
        mut response: ProcessedResponse,
        mut next_round: F,
    ) -> Result<ProcessedResponse, ToolExecutionError>
    where
        F: FnMut(Vec<ToolResult>) -> Fut,
        Fut: std::future::Future<Output = Result<ProcessedResponse, ToolExecutionError>>,
    {
        let mut depth = 0;
        loop {
            depth += 1;
            if depth > 5 {
                return Err(ToolExecutionError::ToolFailure("maximum tool recursion depth reached".to_string()));
            }

            if response.modifications.is_empty() {
                return Ok(response);
            }

            let tool_calls: Vec<ToolCall> = response
                .modifications
                .iter()
                .map(|modification| ToolCall {
                    tool_name: "apply_modification".to_string(),
                    arguments: [
                        ("file_path".to_string(), serde_json::json!(modification.file_path)),
                        ("operation".to_string(), serde_json::json!(format!("{:?}", modification.operation))),
                        ("new_content".to_string(), serde_json::json!(modification.new_content)),
                    ]
                    .into_iter()
                    .collect(),
                })
                .collect();

            let results = self.execute_calls(tool_calls).await?;
            response = next_round(results).await?;
        }
    }
}
