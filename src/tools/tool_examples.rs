/// AI CLI 工具注册示例
/// 展示如何注册和使用各种工具

use std::sync::Arc;
use crate::tools::{
    ToolRegistry,
    // 文件工具
    FileReadTool, FileWriteTool, FileListTool,
    // 代码工具
    CodeSearchTool, FunctionFinderTool, CodeStructureTool,
    // 终端工具
    CommandExecuteTool, EnvironmentInfoTool,
    // 项目工具
    ProjectStructureTool, DependencyAnalyzerTool, BuildTool,
};

/// 创建完整的工具注册表，包含所有可用的工具
pub fn create_full_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 注册文件操作工具
    registry.register(Arc::new(FileReadTool));
    registry.register(Arc::new(FileWriteTool));
    registry.register(Arc::new(FileListTool));

    // 注册代码分析工具
    registry.register(Arc::new(CodeSearchTool));
    registry.register(Arc::new(FunctionFinderTool));
    registry.register(Arc::new(CodeStructureTool));

    // 注册终端命令工具
    registry.register(Arc::new(CommandExecuteTool));
    registry.register(Arc::new(EnvironmentInfoTool));

    // 注册项目管理工具
    registry.register(Arc::new(ProjectStructureTool));
    registry.register(Arc::new(DependencyAnalyzerTool));
    registry.register(Arc::new(BuildTool));

    registry
}

/// 获取所有可用工具的定义（用于发送给 LLM）
pub fn get_all_tool_definitions() -> Vec<crate::tools::ToolDefinition> {
    let registry = create_full_tool_registry();
    registry.list_definitions()
}

/// 示例：如何在 AI CLI 中使用工具
pub async fn example_tool_usage() {
    let registry = create_full_tool_registry();

    // 示例 1: 读取文件
    println!("=== 示例 1: 读取文件 ===");
    let read_call = crate::tools::ToolCall {
        tool_name: "read_file".to_string(),
        arguments: [
            ("path".to_string(), serde_json::json!("src/main.rs")),
            ("start_line".to_string(), serde_json::json!(1)),
            ("end_line".to_string(), serde_json::json!(20)),
        ].into(),
    };

    match registry.execute(read_call).await {
        crate::tools::ToolResult { success: true, data, .. } => {
            println!("✅ 文件读取成功");
            if let Some(content) = data["content"].as_str() {
                println!("内容预览:\n{}", content);
            }
        }
        crate::tools::ToolResult { success: false, error, .. } => {
            println!("❌ 文件读取失败: {:?}", error);
        }
    }

    // 示例 2: 搜索代码
    println!("\n=== 示例 2: 搜索代码 ===");
    let search_call = crate::tools::ToolCall {
        tool_name: "search_code".to_string(),
        arguments: [
            ("pattern".to_string(), serde_json::json!("fn main")),
            ("path".to_string(), serde_json::json!("src")),
            ("include_pattern".to_string(), serde_json::json!("*.rs")),
        ].into(),
    };

    match registry.execute(search_call).await {
        crate::tools::ToolResult { success: true, data, .. } => {
            println!("✅ 代码搜索成功");
            if let Some(matches) = data["matches"].as_array() {
                println!("找到 {} 个匹配", matches.len());
                for (i, match_) in matches.iter().enumerate().take(3) {
                    if let (Some(file), Some(line), Some(content)) = (
                        match_["file"].as_str(),
                        match_["line"].as_u64(),
                        match_["content"].as_str(),
                    ) {
                        println!("  {}. {}:{} - {}", i + 1, file, line, content.trim());
                    }
                }
            }
        }
        crate::tools::ToolResult { success: false, error, .. } => {
            println!("❌ 代码搜索失败: {:?}", error);
        }
    }

    // 示例 3: 执行安全命令
    println!("\n=== 示例 3: 执行命令 ===");
    let cmd_call = crate::tools::ToolCall {
        tool_name: "execute_command".to_string(),
        arguments: [
            ("command".to_string(), serde_json::json!("echo")),
            ("args".to_string(), serde_json::json!(["Hello from AI CLI!"])),
        ].into(),
    };

    match registry.execute(cmd_call).await {
        crate::tools::ToolResult { success: true, data, .. } => {
            println!("✅ 命令执行成功");
            if let Some(stdout) = data["stdout"].as_str() {
                println!("输出: {}", stdout.trim());
            }
        }
        crate::tools::ToolResult { success: false, error, .. } => {
            println!("❌ 命令执行失败: {:?}", error);
        }
    }

    // 示例 4: 分析项目结构
    println!("\n=== 示例 4: 项目分析 ===");
    let project_call = crate::tools::ToolCall {
        tool_name: "analyze_project".to_string(),
        arguments: [
            ("path".to_string(), serde_json::json!(".")),
        ].into(),
    };

    match registry.execute(project_call).await {
        crate::tools::ToolResult { success: true, data, .. } => {
            println!("✅ 项目分析成功");
            if let Some(analysis) = data["analysis"].as_object() {
                if let Some(languages) = analysis["languages"].as_object() {
                    println!("检测到语言:");
                    for (lang, count) in languages {
                        if let Some(count) = count.as_u64() {
                            println!("  - {}: {} 个文件", lang, count);
                        }
                    }
                }
                if let Some(package_managers) = analysis["package_managers"].as_array() {
                    println!("包管理器: {:?}", package_managers);
                }
            }
        }
        crate::tools::ToolResult { success: false, error, .. } => {
            println!("❌ 项目分析失败: {:?}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = create_full_tool_registry();
        assert!(registry.count() > 0);

        let definitions = get_all_tool_definitions();
        assert_eq!(definitions.len(), registry.count());

        // 检查是否包含所有预期的工具
        let tool_names: Vec<&str> = definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(tool_names.contains(&"read_file"));
        assert!(tool_names.contains(&"search_code"));
        assert!(tool_names.contains(&"execute_command"));
        assert!(tool_names.contains(&"analyze_project"));
    }

    #[tokio::test]
    async fn test_environment_info_tool() {
        let registry = create_full_tool_registry();

        let env_call = crate::tools::ToolCall {
            tool_name: "get_environment_info".to_string(),
            arguments: Default::default(),
        };

        let result = registry.execute(env_call).await;
        assert!(result.success);
        assert!(result.data["os"].is_string());
        assert!(result.data["arch"].is_string());
    }
}