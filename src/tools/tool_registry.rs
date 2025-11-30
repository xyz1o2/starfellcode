/// 工具注册表和管理系统

use super::tool::{Tool, ToolCall, ToolDefinition, ToolResult};
use std::collections::HashMap;
use std::sync::Arc;

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// 列出所有可用工具定义（用于发送给 LLM）
    pub fn list_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.definition())
            .collect()
    }

    /// 执行工具调用
    pub async fn execute(&self, call: ToolCall) -> ToolResult {
        match self.get(&call.tool_name) {
            Some(tool) => tool.execute(call).await,
            None => ToolResult {
                success: false,
                data: serde_json::json!(null),
                error: Some(format!("Tool '{}' not found", call.tool_name)),
            },
        }
    }

    /// 获取工具数量
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// 检查工具是否存在
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::tool::{ToolDefinition, ToolParameter};
    use std::pin::Pin;
    use std::future::Future;

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

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool));

        assert!(registry.has_tool("mock_tool"));
        assert_eq!(registry.count(), 1);
        assert!(registry.get("mock_tool").is_some());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(MockTool));

        let call = ToolCall {
            tool_name: "mock_tool".to_string(),
            arguments: Default::default(),
        };

        let result = registry.execute(call).await;
        assert!(result.success);
    }
}
