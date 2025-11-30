/// 核心工具 Trait 定义
/// 支持 LLM 函数调用（Function Calling）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

/// 工具参数的 JSON Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: String, // "string", "number", "boolean", "object", "array"
    pub required: bool,
}

/// 工具定义（用于发送给 LLM）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// 工具调用请求（来自 LLM）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// 核心工具 Trait
pub trait Tool: Send + Sync {
    /// 工具名称（用于 LLM 识别）
    fn name(&self) -> &str;

    /// 工具描述（用于 LLM 理解）
    fn description(&self) -> &str;

    /// 工具定义（包含参数 schema）
    fn definition(&self) -> ToolDefinition;

    /// 执行工具（返回 Future 以支持 dyn trait）
    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>>;
}

/// 工具执行上下文
pub struct ToolExecutionContext {
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

impl ToolExecutionContext {
    pub fn new(tool_name: String, arguments: HashMap<String, serde_json::Value>) -> Self {
        Self {
            tool_name,
            arguments,
        }
    }

    /// 获取字符串参数
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.arguments
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// 获取数字参数
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.arguments.get(key).and_then(|v| v.as_f64())
    }

    /// 获取布尔参数
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.arguments.get(key).and_then(|v| v.as_bool())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_execution_context() {
        let mut args = HashMap::new();
        args.insert("query".to_string(), serde_json::json!("hello"));
        args.insert("count".to_string(), serde_json::json!(42));

        let ctx = ToolExecutionContext::new("test_tool".to_string(), args);
        assert_eq!(ctx.get_string("query"), Some("hello".to_string()));
        assert_eq!(ctx.get_number("count"), Some(42.0));
    }

    struct MockTool;

    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_tool"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: self.name().to_string(),
                description: self.description().to_string(),
                parameters: vec![ToolParameter {
                    name: "test".to_string(),
                    description: "Test parameter".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                }],
            }
        }

        fn execute(&self, _call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
            Box::pin(async {
                ToolResult {
                    success: true,
                    data: serde_json::json!({"result": "mock"}),
                    error: None,
                }
            })
        }
    }

    #[tokio::test]
    async fn test_mock_tool() {
        let tool = MockTool;
        let call = ToolCall {
            tool_name: "mock_tool".to_string(),
            arguments: Default::default(),
        };
        let result = tool.execute(call).await;
        assert!(result.success);
    }
}
