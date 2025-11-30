/// 项目管理工具集
/// 提供项目结构分析、依赖管理、构建工具等功能

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::future::Future;

/// 项目结构分析工具
pub struct ProjectStructureTool;

impl Tool for ProjectStructureTool {
    fn name(&self) -> &str {
        "analyze_project"
    }

    fn description(&self) -> &str {
        "分析项目结构，识别语言、框架和配置文件"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "项目根目录路径".to_string(),
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

            match analyze_project_structure(&path) {
                Ok(analysis) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "analysis": analysis
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Project analysis failed: {}", e)),
                },
            }
        })
    }
}

/// 依赖分析工具
pub struct DependencyAnalyzerTool;

impl Tool for DependencyAnalyzerTool {
    fn name(&self) -> &str {
        "analyze_dependencies"
    }

    fn description(&self) -> &str {
        "分析项目依赖关系和版本信息"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "项目根目录路径".to_string(),
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

            match analyze_dependencies(&path) {
                Ok(deps) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "dependencies": deps
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Dependency analysis failed: {}", e)),
                },
            }
        })
    }
}

/// 构建工具
pub struct BuildTool;

impl Tool for BuildTool {
    fn name(&self) -> &str {
        "build_project"
    }

    fn description(&self) -> &str {
        "构建项目（编译、打包等）"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "项目根目录路径".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "target".to_string(),
                    description: "构建目标（debug, release, test等）".to_string(),
                    param_type: "string".to_string(),
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

            let target = ctx.get_string("target").unwrap_or_else(|| "debug".to_string());

            match build_project(&path, &target).await {
                Ok(result) => ToolResult {
                    success: result.success,
                    data: serde_json::json!({
                        "path": path,
                        "target": target,
                        "output": result.output,
                        "duration_ms": result.duration_ms
                    }),
                    error: result.error,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Build failed: {}", e)),
                },
            }
        })
    }
}

fn analyze_project_structure(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut analysis = serde_json::json!({
        "languages": {},
        "frameworks": [],
        "config_files": [],
        "build_tools": [],
        "package_managers": [],
        "structure": {
            "src_dirs": [],
            "test_dirs": [],
            "config_dirs": [],
            "docs": []
        }
    });

    fn analyze_directory(path: &Path, analysis: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // 检测语言
                if let Some(languages) = analysis["languages"].as_object_mut() {
                    detect_language(&file_name, languages);
                }

                // 检测框架和配置文件
                detect_frameworks_and_configs(&file_name, analysis);
            } else if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // 检测目录结构
                if let Some(structure) = analysis["structure"].as_object_mut() {
                    detect_structure(&dir_name, structure);
                }

                // 递归分析（限制深度）
                if path.components().count() < 5 {
                    analyze_directory(&path, analysis)?;
                }
            }
        }

        Ok(())
    }

    analyze_directory(Path::new(path), &mut analysis)?;
    Ok(analysis)
}

fn detect_language(file_name: &str, languages: &mut serde_json::Map<String, serde_json::Value>) {
    let ext_to_lang = [
        ("rs", "Rust"),
        ("py", "Python"),
        ("js", "JavaScript"),
        ("ts", "TypeScript"),
        ("java", "Java"),
        ("cpp", "C++"),
        ("c", "C"),
        ("go", "Go"),
        ("rb", "Ruby"),
        ("php", "PHP"),
        ("swift", "Swift"),
        ("kt", "Kotlin"),
        ("scala", "Scala"),
    ];

    for (ext, lang) in ext_to_lang {
        if file_name.ends_with(&format!(".{}", ext)) {
            let count = languages.get(lang).and_then(|v| v.as_u64()).unwrap_or(0);
            languages.insert(lang.to_string(), serde_json::json!(count + 1));
            break;
        }
    }
}

fn detect_frameworks_and_configs(
    file_name: &str,
    analysis: &mut serde_json::Value,
) {
    let file_name_lower = file_name.to_lowercase();
    
    match file_name_lower.as_str() {
        // Rust
        "cargo.toml" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("Cargo"));
            }
            if let Some(cf) = analysis["config_files"].as_array_mut() {
                cf.push(serde_json::json!("Cargo.toml"));
            }
        }
        "cargo.lock" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("Cargo"));
            }
        }

        // Node.js
        "package.json" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("npm/yarn"));
            }
            if let Some(cf) = analysis["config_files"].as_array_mut() {
                cf.push(serde_json::json!("package.json"));
            }
        }
        "yarn.lock" | "package-lock.json" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("npm/yarn"));
            }
        }

        // Python
        "requirements.txt" | "setup.py" | "pyproject.toml" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("pip"));
            }
            if let Some(cf) = analysis["config_files"].as_array_mut() {
                cf.push(serde_json::json!(file_name));
            }
        }
        "poetry.lock" => {
            if let Some(pm) = analysis["package_managers"].as_array_mut() {
                pm.push(serde_json::json!("Poetry"));
            }
        }

        // 构建工具
        "makefile" | "makefile.am" | "makefile.in" => {
            if let Some(bt) = analysis["build_tools"].as_array_mut() {
                bt.push(serde_json::json!("Make"));
            }
        }
        "cmake" => {
            if let Some(bt) = analysis["build_tools"].as_array_mut() {
                bt.push(serde_json::json!("CMake"));
            }
        }
        "gradle" => {
            if let Some(bt) = analysis["build_tools"].as_array_mut() {
                bt.push(serde_json::json!("Gradle"));
            }
        }
        "maven" => {
            if let Some(bt) = analysis["build_tools"].as_array_mut() {
                bt.push(serde_json::json!("Maven"));
            }
        }

        // 框架检测
        "dockerfile" => {
            if let Some(fw) = analysis["frameworks"].as_array_mut() {
                fw.push(serde_json::json!("Docker"));
            }
        }
        "docker-compose.yml" => {
            if let Some(fw) = analysis["frameworks"].as_array_mut() {
                fw.push(serde_json::json!("Docker Compose"));
            }
        }

        _ => {}
    }
}

fn detect_structure(dir_name: &str, structure: &mut serde_json::Map<String, serde_json::Value>) {
    match dir_name.to_lowercase().as_str() {
        "src" | "source" | "sources" | "lib" | "libs" => {
            if let Some(src_dirs) = structure["src_dirs"].as_array_mut() {
                src_dirs.push(serde_json::json!(dir_name));
            }
        }
        "test" | "tests" | "spec" | "specs" => {
            if let Some(test_dirs) = structure["test_dirs"].as_array_mut() {
                test_dirs.push(serde_json::json!(dir_name));
            }
        }
        "config" | "configs" | "conf" | ".config" => {
            if let Some(config_dirs) = structure["config_dirs"].as_array_mut() {
                config_dirs.push(serde_json::json!(dir_name));
            }
        }
        "doc" | "docs" | "documentation" | "wiki" => {
            if let Some(docs) = structure["docs"].as_array_mut() {
                docs.push(serde_json::json!(dir_name));
            }
        }
        _ => {}
    }
}

fn analyze_dependencies(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut deps = serde_json::json!({
        "direct": [],
        "dev": [],
        "transitive": [],
        "total_count": 0
    });

    // 检测不同类型的项目依赖
    if Path::new(path).join("Cargo.toml").exists() {
        analyze_cargo_deps(path, &mut deps)?;
    } else if Path::new(path).join("package.json").exists() {
        analyze_npm_deps(path, &mut deps)?;
    } else if Path::new(path).join("requirements.txt").exists() {
        analyze_python_deps(path, &mut deps)?;
    }

    Ok(deps)
}

fn analyze_cargo_deps(path: &str, deps: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = Path::new(path).join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path)?;

    // 简单的 TOML 解析（实际项目中应该使用 toml crate）
    let direct = deps["direct"].as_array_mut().unwrap();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('"') && line.contains(" = ") {
            if let Some(dep_name) = line.split('"').nth(1) {
                direct.push(serde_json::json!({
                    "name": dep_name,
                    "type": "cargo"
                }));
            }
        }
    }

    deps["total_count"] = serde_json::json!(direct.len());
    Ok(())
}

fn analyze_npm_deps(path: &str, deps: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let package_json_path = Path::new(path).join("package.json");
    let content = fs::read_to_string(package_json_path)?;

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(deps_obj) = json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps_obj {
                if let Some(direct) = deps["direct"].as_array_mut() {
                    direct.push(serde_json::json!({
                        "name": name,
                        "version": version,
                        "type": "npm"
                    }));
                }
            }
        }

        if let Some(dev_deps_obj) = json.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, version) in dev_deps_obj {
                if let Some(dev) = deps["dev"].as_array_mut() {
                    dev.push(serde_json::json!({
                        "name": name,
                        "version": version,
                        "type": "npm-dev"
                    }));
                }
            }
        }
    }

    if let Some(direct) = deps["direct"].as_array() {
        deps["total_count"] = serde_json::json!(direct.len());
    }
    Ok(())
}

fn analyze_python_deps(path: &str, deps: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let req_path = Path::new(path).join("requirements.txt");
    let content = fs::read_to_string(req_path)?;

    let direct = deps["direct"].as_array_mut().unwrap();

    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            let parts: Vec<&str> = line.split(&['=', '>', '<', '!', '~'][..]).collect();
            if let Some(package) = parts.first() {
                direct.push(serde_json::json!({
                    "name": package.trim(),
                    "spec": line,
                    "type": "pip"
                }));
            }
        }
    }

    deps["total_count"] = serde_json::json!(direct.len());
    Ok(())
}

async fn build_project(path: &str, target: &str) -> Result<BuildResult, Box<dyn std::error::Error + Send + Sync>> {
    use std::time::Instant;
    use tokio::process::Command;

    let start_time = Instant::now();

    let (command, args) = match target {
        "release" => ("cargo", vec!["build", "--release"]),
        "debug" => ("cargo", vec!["build"]),
        "test" => ("cargo", vec!["test"]),
        "check" => ("cargo", vec!["check"]),
        _ => ("cargo", vec!["build"]),
    };

    let output = Command::new(command)
        .args(&args)
        .current_dir(path)
        .output()
        .await?;

    let duration = start_time.elapsed().as_millis();

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(BuildResult {
        success,
        output: format!("{}\n{}", stdout, stderr),
        duration_ms: duration,
        error: if success { None } else { Some(stderr) },
    })
}

#[derive(Debug)]
struct BuildResult {
    success: bool,
    output: String,
    duration_ms: u128,
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_project_structure_analysis() {
        let temp_dir = tempdir().unwrap();

        // 创建模拟项目结构
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#).unwrap();

        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let structure_tool = ProjectStructureTool;
        let structure_call = ToolCall {
            tool_name: "analyze_project".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(temp_dir.path().to_str())),
            ].into(),
        };

        let result = structure_tool.execute(structure_call).await;
        assert!(result.success);

        let analysis = &result.data["analysis"];
        assert!(analysis["languages"]["Rust"].is_number());
        assert!(analysis["package_managers"].as_array().unwrap().contains(&serde_json::json!("Cargo")));
    }

    #[tokio::test]
    async fn test_dependency_analysis() {
        let temp_dir = tempdir().unwrap();

        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[dependencies]
serde = "1.0"
tokio = "1.0"
"#).unwrap();

        let dep_tool = DependencyAnalyzerTool;
        let dep_call = ToolCall {
            tool_name: "analyze_dependencies".to_string(),
            arguments: [
                ("path".to_string(), serde_json::json!(temp_dir.path().to_str())),
            ].into(),
        };

        let result = dep_tool.execute(dep_call).await;
        assert!(result.success);

        let deps = &result.data["dependencies"];
        assert!(deps["direct"].as_array().unwrap().len() > 0);
    }
}