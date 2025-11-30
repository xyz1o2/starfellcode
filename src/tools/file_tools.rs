/// 文件操作工具集
/// 提供文件读取、写入、修改等功能

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::future::Future;

/// 文件读取工具
pub struct FileReadTool;

impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取文件内容，支持指定行范围"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "文件路径".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "start_line".to_string(),
                    description: "起始行号（可选，从1开始）".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                },
                ToolParameter {
                    name: "end_line".to_string(),
                    description: "结束行号（可选）".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let path = match ctx.get_string("path") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: path".to_string()),
                },
            };

            match fs::read_to_string(&path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    let total_lines = lines.len();

                    let start_line = ctx.get_number("start_line")
                        .map(|n| (n as usize).saturating_sub(1))
                        .unwrap_or(0);
                    let end_line = ctx.get_number("end_line")
                        .map(|n| n as usize)
                        .unwrap_or(total_lines);

                    let start_line = start_line.min(total_lines);
                    let end_line = end_line.min(total_lines).max(start_line);

                    let selected_lines: Vec<String> = lines[start_line..end_line]
                        .iter()
                        .enumerate()
                        .map(|(i, line)| format!("{:4}: {}", start_line + i + 1, line))
                        .collect();

                    ToolResult {
                        success: true,
                        data: serde_json::json!({
                            "path": path,
                            "total_lines": total_lines,
                            "selected_lines": selected_lines.len(),
                            "content": selected_lines.join("\n")
                        }),
                        error: None,
                    }
                }
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Failed to read file '{}': {}", path, e)),
                },
            }
        })
    }
}

/// 文件写入工具
pub struct FileWriteTool;

impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "写入或覆盖文件内容"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "文件路径".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "content".to_string(),
                    description: "要写入的内容".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let path = match ctx.get_string("path") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: path".to_string()),
                },
            };

            let content = match ctx.get_string("content") {
                Some(c) => c,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: content".to_string()),
                },
            };

            // 确保父目录存在
            if let Some(parent) = Path::new(&path).parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return ToolResult {
                        success: false,
                        data: serde_json::json!(null),
                        error: Some(format!("Failed to create parent directory: {}", e)),
                    };
                }
            }

            let bytes_written = content.len();
            match fs::write(&path, content) {
                Ok(_) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "bytes_written": bytes_written
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Failed to write file '{}': {}", path, e)),
                },
            }
        })
    }
}

/// 文件列表工具
pub struct FileListTool;

impl Tool for FileListTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "列出目录中的文件和子目录"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "目录路径".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "recursive".to_string(),
                    description: "是否递归列出子目录（默认false）".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let path = match ctx.get_string("path") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: path".to_string()),
                },
            };

            let recursive = ctx.get_bool("recursive").unwrap_or(false);

            match list_directory(&path, recursive) {
                Ok(entries) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "entries": entries
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Failed to list directory '{}': {}", path, e)),
                },
            }
        })
    }
}

fn list_directory(path: &str, recursive: bool) -> Result<Vec<serde_json::Value>, std::io::Error> {
    let mut entries = Vec::new();

    fn visit_dir(path: &Path, entries: &mut Vec<serde_json::Value>, recursive: bool) -> Result<(), std::io::Error> {
        let dir_entries = fs::read_dir(path)?;

        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("<invalid>")
                .to_string();

            let metadata = entry.metadata()?;
            let is_dir = metadata.is_dir();

            entries.push(serde_json::json!({
                "name": file_name,
                "path": path.to_string_lossy(),
                "is_directory": is_dir,
                "size": if is_dir { 0 } else { metadata.len() }
            }));

            if recursive && is_dir {
                visit_dir(&path, entries, recursive)?;
            }
        }

        Ok(())
    }

    visit_dir(Path::new(path), &mut entries, recursive)?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_write_and_read() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 写入文件
        let write_tool = FileWriteTool;
        let write_call = ToolCall {
            tool_name: "write_file".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(file_path.to_str())),
                ("content".to_string(), serde_json::json!("Hello, World!\nSecond line")),
            ].into(),
        };

        let write_result = write_tool.execute(write_call).await;
        assert!(write_result.success);

        // 读取文件
        let read_tool = FileReadTool;
        let read_call = ToolCall {
            tool_name: "read_file".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(file_path.to_str())),
            ].into(),
        };

        let read_result = read_tool.execute(read_call).await;
        assert!(read_result.success);

        let content = read_result.data["content"].as_str().unwrap();
        assert!(content.contains("Hello, World!"));
        assert!(content.contains("Second line"));
    }

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = tempdir().unwrap();

        // 创建测试文件
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let list_tool = FileListTool;
        let list_call = ToolCall {
            tool_name: "list_directory".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(temp_dir.path().to_str())),
            ].into(),
        };

        let list_result = list_tool.execute(list_call).await;
        assert!(list_result.success);

        let entries = list_result.data["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 3); // 2 files + 1 directory
    }
}