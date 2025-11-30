/// 终端命令工具集
/// 提供安全的系统命令执行功能

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use std::pin::Pin;
use std::future::Future;
use tokio::process::Command as TokioCommand;
use std::env;

/// 命令执行工具
pub struct CommandExecuteTool;

impl Tool for CommandExecuteTool {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "执行终端命令（注意：仅限安全命令）"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "command".to_string(),
                    description: "要执行的命令".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "args".to_string(),
                    description: "命令参数数组".to_string(),
                    param_type: "array".to_string(),
                    required: false,
                },
                ToolParameter {
                    name: "working_directory".to_string(),
                    description: "工作目录".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                },
                ToolParameter {
                    name: "timeout_seconds".to_string(),
                    description: "超时时间（秒，默认30）".to_string(),
                    param_type: "number".to_string(),
                    required: false,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let command = match ctx.get_string("command") {
                Some(c) => c,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: command".to_string()),
                },
            };

            // 安全检查 - 只允许特定的安全命令
            if !is_safe_command(&command) {
                return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Command '{}' is not allowed for security reasons", command)),
                };
            }

            let args = ctx.arguments.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                .unwrap_or_default();

            let working_directory = ctx.get_string("working_directory");
            let timeout_seconds = ctx.get_number("timeout_seconds").unwrap_or(30.0) as u64;

            match execute_command(&command, &args, working_directory.as_deref(), timeout_seconds).await {
                Ok(result) => ToolResult {
                    success: result.success,
                    data: serde_json::json!({
                        "command": command,
                        "args": args,
                        "exit_code": result.exit_code,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                        "duration_ms": result.duration_ms
                    }),
                    error: result.error,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Command execution failed: {}", e)),
                },
            }
        })
    }
}

/// 安全命令检查
fn is_safe_command(command: &str) -> bool {
    let safe_commands = [
        "ls", "dir", "pwd", "echo", "cat", "head", "tail", "grep", "find", "wc", "sort", "uniq",
        "git", "cargo", "npm", "yarn", "python", "python3", "node", "rustc",
        "mkdir", "cp", "mv", "rm", "touch", "chmod", "chown",
        "ps", "top", "df", "du", "free", "uptime",
        "curl", "wget", "ping", "nslookup", "dig",
    ];

    // 危险命令黑名单
    let dangerous_commands = [
        "sudo", "su", "passwd", "chmod +x", "rm -rf", "dd", "mkfs", "fdisk", "format",
        "shutdown", "reboot", "halt", "poweroff", "init", "telinit",
        "useradd", "userdel", "groupadd", "groupdel",
        "mount", "umount", "fsck", "e2fsck",
        "iptables", "firewall-cmd", "ufw",
        "systemctl", "service", "chkconfig",
    ];

    safe_commands.contains(&command) &&
    !dangerous_commands.iter().any(|dangerous| command.contains(dangerous))
}

#[derive(Debug)]
struct CommandResult {
    success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u128,
    error: Option<String>,
}

async fn execute_command(
    command: &str,
    args: &[String],
    working_directory: Option<&str>,
    timeout_seconds: u64,
) -> Result<CommandResult, Box<dyn std::error::Error + Send + Sync>> {
    use std::time::{Duration, Instant};

    let start_time = Instant::now();

    let mut cmd = TokioCommand::new(command);
    cmd.args(args);

    if let Some(cwd) = working_directory {
        cmd.current_dir(cwd);
    }

    // 设置超时
    let timeout_duration = Duration::from_secs(timeout_seconds);

    match tokio::time::timeout(timeout_duration, cmd.output()).await {
        Ok(result) => {
            let output = result?;
            let duration = start_time.elapsed().as_millis();

            let success = output.status.success();
            let exit_code = output.status.code();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(CommandResult {
                success,
                exit_code,
                stdout,
                stderr,
                duration_ms: duration,
                error: None,
            })
        }
        Err(_) => {
            // 超时
            Ok(CommandResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                duration_ms: start_time.elapsed().as_millis(),
                error: Some(format!("Command timed out after {} seconds", timeout_seconds)),
            })
        }
    }
}

/// 环境信息工具
pub struct EnvironmentInfoTool;

impl Tool for EnvironmentInfoTool {
    fn name(&self) -> &str {
        "get_environment_info"
    }

    fn description(&self) -> &str {
        "获取当前环境信息（OS、路径、工具版本等）"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![],
        }
    }

    fn execute(&self, _call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let home_dir = env::var("HOME")
                .or_else(|_| env::var("USERPROFILE"))
                .ok();
            
            let mut info = serde_json::json!({
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "current_dir": std::env::current_dir().unwrap_or_default().to_string_lossy(),
                "home_dir": home_dir,
                "tools": {}
            });

            // 检查常用工具版本
            let tools = [
                ("rustc", vec!["--version"]),
                ("cargo", vec!["--version"]),
                ("git", vec!["--version"]),
                ("node", vec!["--version"]),
                ("npm", vec!["--version"]),
                ("python", vec!["--version"]),
                ("python3", vec!["--version"]),
            ];

            let tools_obj = info["tools"].as_object_mut().unwrap();

            for (tool, args) in tools {
                if let Ok(result) = execute_command(tool, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), None, 5).await {
                    if result.success {
                        let version = result.stdout.lines().next().unwrap_or("unknown").to_string();
                        tools_obj.insert(tool.to_string(), serde_json::json!(version.trim()));
                    }
                }
            }

            ToolResult {
                success: true,
                data: info,
                error: None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_command_check() {
        assert!(is_safe_command("ls"));
        assert!(is_safe_command("git"));
        assert!(is_safe_command("cargo"));
        assert!(!is_safe_command("sudo"));
        assert!(!is_safe_command("rm -rf"));
        assert!(!is_safe_command("shutdown"));
    }

    #[tokio::test]
    async fn test_command_execute() {
        let cmd_tool = CommandExecuteTool;
        let cmd_call = ToolCall {
            tool_name: "execute_command".to_string(),
            arguments: [
                ("command".to_string(), serde_json::json!("echo")),
                ("args".to_string(), serde_json::json!(["Hello, World!"])),
            ].into(),
        };

        let result = cmd_tool.execute(cmd_call).await;
        assert!(result.success);
        assert!(result.data["stdout"].as_str().unwrap().contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_environment_info() {
        let env_tool = EnvironmentInfoTool;
        let env_call = ToolCall {
            tool_name: "get_environment_info".to_string(),
            arguments: Default::default(),
        };

        let result = env_tool.execute(env_call).await;
        assert!(result.success);
        assert!(result.data["os"].is_string());
        assert!(result.data["arch"].is_string());
    }
}