# 项目结构说明

## 📁 目录概览

```
starfellcode/
├── src/                    # 源代码
│   ├── main.rs            # 应用入口
│   ├── app.rs             # 应用主逻辑
│   ├── ai/                # AI 集成模块
│   ├── ui/                # TUI 用户界面
│   ├── core/              # 核心功能
│   ├── commands/          # 命令系统
│   ├── tools/             # AI 工具系统
│   ├── prompts/           # 提示词模块
│   ├── events/            # 事件处理
│   └── utils/             # 工具函数
├── docs/                  # 文档（新增）
├── examples/              # 示例代码
├── assets/                # 资源文件
├── tests/                 # 测试
├── Cargo.toml            # 项目配置
└── README.md             # 项目说明
```

## 🎯 核心模块详解

### 1. **AI 模块** (`src/ai/`)
LLM 集成和 AI 功能的核心

| 文件 | 功能 | 行数 |
|------|------|------|
| `client.rs` | 基础 LLM 客户端 | 200+ |
| `advanced_client.rs` | 高级客户端（重试、流式） | 200+ |
| `streaming.rs` | SSE 流式处理 | 150+ |
| `code_modification.rs` | 代码修改检测和执行 | 400+ |
| `commands.rs` | 聊天命令解析 | 150+ |
| `config.rs` | LLM 配置管理 | 100+ |
| `tools.rs` | AI 工具系统 | 150+ |
| `fim.rs` | Fill-In-The-Middle 功能 | 100+ |
| `context.rs` | 上下文管理 | 100+ |

**关键特性**：
- ✅ 多 LLM 提供商支持（OpenAI, Google, Claude, Ollama）
- ✅ 流式响应实时显示
- ✅ 自动重试机制
- ✅ 代码修改检测和确认

### 2. **UI 模块** (`src/ui/`)
Ratatui TUI 界面和渲染系统

| 文件 | 功能 | 行数 |
|------|------|------|
| `mod.rs` | 主渲染逻辑 | 300+ |
| `pixel_layout_v2.rs` | 像素艺术布局 | 450+ |
| `svg_avatar.rs` | SVG 头像渲染 | 120+ |
| `command_hints.rs` | 命令提示系统 | 150+ |
| `file_search.rs` | 文件搜索引擎 | 200+ |
| `chat_display.rs` | 聊天显示系统 | 400+ |
| `editor.rs` | 编辑器组件 | 100+ |

**关键特性**：
- ✅ 像素艺术风格头像
- ✅ 命令提示自动完成
- ✅ 文件搜索和提及
- ✅ 三种聊天显示风格

### 3. **核心模块** (`src/core/`)
应用核心逻辑和数据管理

| 文件 | 功能 | 行数 |
|------|------|------|
| `history.rs` | 聊天历史管理 | 150+ |
| `context_optimizer.rs` | 上下文优化 | 200+ |
| `buffer.rs` | 文本缓冲区 | 100+ |
| `cursor.rs` | 光标管理 | 100+ |
| `message.rs` | 消息数据结构 | 80+ |
| `integration.rs` | 模块集成 | 100+ |

**关键特性**：
- ✅ 滑动窗口上下文管理
- ✅ 消息历史持久化
- ✅ 令牌计数和优化

### 4. **命令系统** (`src/commands/`)
聊天命令和文件操作

| 文件 | 功能 | 行数 |
|------|------|------|
| `file_commands.rs` | 文件操作命令 | 200+ |
| `mod.rs` | 命令导出 | 50+ |

**支持的命令**：
- `/help` - 显示帮助
- `/clear` - 清除历史
- `/status` - 显示状态
- `/model` - 设置模型
- `/provider` - 切换提供商
- `/temp` - 设置温度
- `/tokens` - 设置令牌数
- `/history` - 显示历史

### 5. **工具系统** (`src/tools/`)
AI 函数调用工具系统

| 文件 | 功能 | 行数 |
|------|------|------|
| `tool.rs` | Tool trait 定义 | 150+ |
| `tool_registry.rs` | 工具注册表 | 130+ |
| `file_tools.rs` | 文件操作工具 | 200+ |
| `code_tools.rs` | 代码分析工具 | 200+ |
| `terminal_tools.rs` | 终端命令工具 | 150+ |
| `project_tools.rs` | 项目管理工具 | 150+ |
| `tool_examples.rs` | 工具示例 | 100+ |

**工具列表**：
- ✅ FileReadTool - 读取文件
- ✅ FileWriteTool - 写入文件
- ✅ FileListTool - 列出目录
- ✅ CodeSearchTool - 搜索代码
- ✅ FunctionFinderTool - 查找函数
- ✅ CommandExecuteTool - 执行命令
- ✅ ProjectStructureTool - 分析项目

### 6. **提示词模块** (`src/prompts/`)
动态提示词生成系统

| 文件 | 功能 | 行数 |
|------|------|------|
| `mod.rs` | PromptGenerator trait | 50+ |
| `pair_programming.rs` | 配对编程提示词 | 150+ |
| `code_review.rs` | 代码审查提示词 | 150+ |
| `debugging.rs` | 调试提示词 | 150+ |

**特性**：
- ✅ 根据对话历史动态调整
- ✅ 多种提示词类型
- ✅ 易于扩展

### 7. **事件处理** (`src/events/`)
键盘输入和事件处理

| 文件 | 功能 | 行数 |
|------|------|------|
| `handler.rs` | 事件处理器 | 300+ |
| `mod.rs` | 模块导出 | 20+ |

**事件优先级**：
1. 文件命令确认对话（最高）
2. 命令提示导航
3. 普通文本输入（最低）

## 🔄 数据流

```
用户输入
   ↓
事件处理 (src/events/handler.rs)
   ↓
命令解析 (src/commands/, src/ai/commands.rs)
   ↓
AI 处理 (src/ai/)
   ├─ LLM 请求
   ├─ 流式响应
   └─ 代码修改检测
   ↓
UI 渲染 (src/ui/)
   ├─ 聊天历史
   ├─ 输入框
   └─ 状态栏
   ↓
显示输出
```

## 📊 代码统计

| 模块 | 文件数 | 代码行数 | 功能 |
|------|--------|---------|------|
| AI | 9 | 2000+ | LLM 集成 |
| UI | 7 | 2500+ | TUI 界面 |
| Core | 7 | 1000+ | 核心逻辑 |
| Commands | 2 | 300+ | 命令系统 |
| Tools | 8 | 1500+ | 工具系统 |
| Prompts | 4 | 600+ | 提示词 |
| Events | 2 | 350+ | 事件处理 |
| Utils | 5 | 300+ | 工具函数 |
| **总计** | **44** | **8500+** | **完整应用** |

## 🚀 快速开始

### 编译
```bash
cargo build --release
```

### 运行
```bash
cargo run
```

### 测试
```bash
cargo test
```

### 检查
```bash
cargo check
```

## 📚 关键文件

### 应用入口
- `src/main.rs` - 应用启动和事件循环

### 应用主逻辑
- `src/app.rs` - App 结构体和核心方法

### 配置
- `.env` - 环境变量配置
- `Cargo.toml` - 项目依赖

## 🔧 技术栈

- **Rust** 1.70+ - 编程语言
- **Tokio** 1.x - 异步运行时
- **Ratatui** 0.26 - TUI 框架
- **Crossterm** 0.27 - 终端事件
- **Reqwest** 0.11 - HTTP 客户端
- **Serde** 1.0 - 序列化
- **Tree-Sitter** - 代码分析

## 📖 相关文档

- `README.md` - 项目概览
- `RATATUI_V2_IMPLEMENTATION_GUIDE.md` - UI 实现指南
- `TREE_SITTER_IMPLEMENTATION.md` - 代码分析指南
- `PROMPTS_ARCHITECTURE.md` - 提示词架构
- `LLM_CLIENT_IMPROVEMENTS.md` - LLM 客户端改进

## 💡 设计原则

1. **模块化** - 功能清晰分离
2. **可扩展** - 易于添加新功能
3. **可测试** - 完整的单元测试
4. **可维护** - 清晰的代码结构
5. **高性能** - 优化的渲染和缓存
6. **用户优先** - 所有修改都需确认

## 🎯 下一步

- [ ] 添加更多 LLM 提供商
- [ ] 实现消息持久化
- [ ] 添加快捷键系统
- [ ] 性能优化
- [ ] 集成日志系统
- [ ] 添加指标收集
