/// 代码分析工具集
/// 提供代码搜索、语法分析、结构分析等功能

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::future::Future;

/// 代码搜索工具
pub struct CodeSearchTool;

impl Tool for CodeSearchTool {
    fn name(&self) -> &str {
        "search_code"
    }

    fn description(&self) -> &str {
        "在代码文件中搜索指定的模式或文本"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "pattern".to_string(),
                    description: "搜索模式（支持正则表达式）".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "path".to_string(),
                    description: "搜索路径（文件或目录）".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "include_pattern".to_string(),
                    description: "包含的文件模式（如*.rs, *.js）".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                },
                ToolParameter {
                    name: "case_sensitive".to_string(),
                    description: "是否区分大小写（默认false）".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let pattern = match ctx.get_string("pattern") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: pattern".to_string()),
                },
            };

            let path = match ctx.get_string("path") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: path".to_string()),
                },
            };

            let include_pattern = ctx.get_string("include_pattern");
            let case_sensitive = ctx.get_bool("case_sensitive").unwrap_or(false);

            match search_code(&pattern, &path, include_pattern.as_deref(), case_sensitive) {
                Ok(results) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "pattern": pattern,
                        "path": path,
                        "matches": results
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Search failed: {}", e)),
                },
            }
        })
    }
}

/// 函数定义查找工具
pub struct FunctionFinderTool;

impl Tool for FunctionFinderTool {
    fn name(&self) -> &str {
        "find_functions"
    }

    fn description(&self) -> &str {
        "在代码中查找函数定义"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "搜索路径".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "language".to_string(),
                    description: "编程语言（rust, python, javascript等）".to_string(),
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

            let language = match ctx.get_string("language") {
                Some(l) => l,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: language".to_string()),
                },
            };

            match find_functions(&path, &language) {
                Ok(functions) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "language": language,
                        "functions": functions
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Function search failed: {}", e)),
                },
            }
        })
    }
}

/// 代码结构分析工具
pub struct CodeStructureTool;

impl Tool for CodeStructureTool {
    fn name(&self) -> &str {
        "analyze_structure"
    }

    fn description(&self) -> &str {
        "分析代码文件的结构（类、函数、导入等）"
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
                    name: "language".to_string(),
                    description: "编程语言".to_string(),
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

            let language = match ctx.get_string("language") {
                Some(l) => l,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: language".to_string()),
                },
            };

            match analyze_code_structure(&path, &language) {
                Ok(structure) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "language": language,
                        "structure": structure
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Structure analysis failed: {}", e)),
                },
            }
        })
    }
}

fn search_code(pattern: &str, path: &str, include_pattern: Option<&str>, _case_sensitive: bool) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let regex = Regex::new(pattern)?;
    let mut results = Vec::new();

    fn search_in_path(path: &Path, regex: &Regex, include_pattern: Option<&str>, results: &mut Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        if path.is_file() {
            // 检查文件扩展名
            if let Some(pattern) = include_pattern {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if !matches_pattern(file_name, pattern) {
                    return Ok(());
                }
            }

            let content = fs::read_to_string(path)?;
            for (line_num, line) in content.lines().enumerate() {
                if regex.is_match(line) {
                    results.push(serde_json::json!({
                        "file": path.to_string_lossy(),
                        "line": line_num + 1,
                        "content": line.trim()
                    }));
                }
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                search_in_path(&entry.path(), regex, include_pattern, results)?;
            }
        }
        Ok(())
    }

    search_in_path(Path::new(path), &regex, include_pattern, &mut results)?;
    Ok(results)
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        filename.ends_with(ext)
    } else {
        filename.contains(pattern)
    }
}

fn find_functions(path: &str, language: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let patterns = match language.to_lowercase().as_str() {
        "rust" => vec![r"^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"],
        "python" => vec![r"^(?:def|class)\s+(\w+)"],
        "javascript" | "typescript" => vec![r"^(?:export\s+)?(?:async\s+)?(?:function\s+(\w+)|const\s+(\w+)\s*=|\w+\s*\([^)]*\)\s*=>)"],
        _ => return Err(format!("Unsupported language: {}", language).into()),
    };

    let mut functions = Vec::new();
    let regexes: Vec<Regex> = patterns.iter().map(|p| Regex::new(p)).collect::<Result<_, _>>()?;

    fn search_functions(path: &Path, regexes: &[Regex], functions: &mut Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        if path.is_file() {
            let content = fs::read_to_string(path)?;
            for (line_num, line) in content.lines().enumerate() {
                for regex in regexes {
                    if let Some(captures) = regex.captures(line) {
                        let name = captures.get(1)
                            .or_else(|| captures.get(2))
                            .map(|m| m.as_str())
                            .unwrap_or("unknown");
                        functions.push(serde_json::json!({
                            "file": path.to_string_lossy(),
                            "line": line_num + 1,
                            "name": name,
                            "signature": line.trim()
                        }));
                    }
                }
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                search_functions(&entry.path(), regexes, functions)?;
            }
        }
        Ok(())
    }

    search_functions(Path::new(path), &regexes, &mut functions)?;
    Ok(functions)
}

fn analyze_code_structure(path: &str, language: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut structure = serde_json::json!({
        "imports": [],
        "functions": [],
        "classes": [],
        "constants": []
    });

    match language.to_lowercase().as_str() {
        "rust" => analyze_rust_structure(&lines, &mut structure),
        "python" => analyze_python_structure(&lines, &mut structure),
        _ => return Err(format!("Unsupported language for structure analysis: {}", language).into()),
    }

    Ok(structure)
}

fn analyze_rust_structure(lines: &[&str], structure: &mut serde_json::Value) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") {
            structure["imports"].as_array_mut().unwrap().push(serde_json::json!({
                "line": i + 1,
                "statement": trimmed
            }));
        } else if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            structure["functions"].as_array_mut().unwrap().push(serde_json::json!({
                "line": i + 1,
                "signature": trimmed
            }));
        }
    }
}

fn analyze_python_structure(lines: &[&str], structure: &mut serde_json::Value) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            structure["imports"].as_array_mut().unwrap().push(serde_json::json!({
                "line": i + 1,
                "statement": trimmed
            }));
        } else if trimmed.starts_with("def ") {
            structure["functions"].as_array_mut().unwrap().push(serde_json::json!({
                "line": i + 1,
                "signature": trimmed
            }));
        } else if trimmed.starts_with("class ") {
            structure["classes"].as_array_mut().unwrap().push(serde_json::json!({
                "line": i + 1,
                "signature": trimmed
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_code_search() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#).unwrap();

        let search_tool = CodeSearchTool;
        let search_call = ToolCall {
            tool_name: "search_code".to_string(),
            arguments: [
                ("pattern".to_string(), serde_json::json!("fn ")),
                ("path".to_string(), serde_json::json!(temp_dir.path().to_str())),
                ("include_pattern".to_string(), serde_json::json!("*.rs")),
            ].into(),
        };

        let result = search_tool.execute(search_call).await;
        assert!(result.success);

        let matches = result.data["matches"].as_array().unwrap();
        assert!(matches.len() >= 2); // Should find both functions
    }

    #[tokio::test]
    async fn test_find_functions() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.py");

        fs::write(&file_path, r#"
def hello_world():
    print("Hello, world!")

class Calculator:
    def add(self, a, b):
        return a + b
"#).unwrap();

        let finder_tool = FunctionFinderTool;
        let finder_call = ToolCall {
            tool_name: "find_functions".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(temp_dir.path().to_str())),
                ("language".to_string(), serde_json::json!("python")),
            ].into(),
        };

        let result = finder_tool.execute(finder_call).await;
        assert!(result.success);

        let functions = result.data["functions"].as_array().unwrap();
        assert!(functions.len() >= 2); // Should find function and class
    }
}