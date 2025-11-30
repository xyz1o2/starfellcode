# 像素艺术布局 - 快速开始

## 启动应用

```bash
cargo run
```

## 界面说明

### 顶部状态栏
```
STATUS: CONNECTED                  CTRL+C to EXIT
```
- 左侧显示连接状态
- 右侧显示退出快捷键

### 中间 - 聊天历史区域

显示所有消息，包括：
- **SYSTEM_O1** - 系统初始化消息（Cyan 颜色）
- **USER** - 用户消息（Magenta 颜色）
- **ASSISTANT** - AI 回复（Cyan 颜色）

### 底部 - 输入区域

```
┌─────────────────────────────┐
│ [头像] ▶ 输入框             │
└─────────────────────────────┘
```

- 左侧：用户头像（8x8 像素网格）
- 中间：脉冲箭头 `▶` 或 `▸`（动画效果）
- 右侧：输入框

## 基本操作

### 发送消息

1. 在输入框中输入文本
2. 按 **Enter** 发送
3. 等待 AI 回复

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Enter` | 发送消息 |
| `Ctrl+C` | 退出应用 |
| `Backspace` | 删除字符 |
| `/` | 显示命令提示 |
| `@` | 文件提及 |

### 命令

输入 `/` 查看可用命令：

```
/help      - 显示帮助
/clear     - 清除历史
/status    - 显示状态
/model     - 设置模型
/provider  - 切换提供商
/temp      - 设置温度
/tokens    - 设置令牌数
/history   - 显示历史
```

## 头像类型

### 系统消息
```
SYSTEM_O1
System initialized. Waiting for code instructions.
```
头像颜色：**Cyan** (蓝绿色)

### 用户消息
```
USER
你好，这是一条用户消息
```
头像颜色：**Magenta** (洋红色)

### AI 消息
```
ASSISTANT
这是 AI 的回复
```
头像颜色：**Cyan** (蓝绿色)

## 代码块显示

AI 回复中的代码块会自动显示，支持 diff 颜色：

```
💻 rust
  1 │ fn main() {
  2 │     println!("Hello");  // 绿色 - 添加
  3 │ }
```

- **绿色背景** - 添加的代码行
- **红色背景** - 删除的代码行
- **无背景** - 普通代码行

## 配置

### 环境变量

创建 `.env` 文件配置 LLM：

```env
# OpenAI
OPENAI_API_KEY=sk-...
OPENAI_MODEL=gpt-4

# 或本地服务器
LLM_BASE_URL=http://localhost:1234/v1
LLM_MODEL=model-name
```

### 本地服务器配置

使用命令配置本地服务器：

```
/config-local http://localhost:1234/v1
```

## 常见问题

### Q: 如何改变颜色？
A: 编辑 `src/ui/pixel_layout.rs` 中的颜色定义。

### Q: 如何添加新头像？
A: 在 `PixelAvatar` 中添加新的 `pub fn` 方法。

### Q: 如何调整布局大小？
A: 修改 `render_pixel_layout()` 中的 `Constraint` 值。

### Q: 脉冲箭头速度太快/太慢？
A: 修改 `render_input_area()` 中的 `frame_count / 10` 值。

## 示例交互

```
STATUS: CONNECTED                  CTRL+C to EXIT
─────────────────────────────────────────────────

SYSTEM_O1
System initialized. Waiting for code instructions.

USER
你好，请帮我写一个 Rust 函数

ASSISTANT
当然可以！这是一个简单的例子：

💻 rust
  1 │ fn add(a: i32, b: i32) -> i32 {
  2 │     a + b
  3 │ }

─────────────────────────────────────────────────
[头像] ▶ 
```

## 下一步

1. 配置 LLM API 密钥
2. 尝试不同的命令
3. 查看完整文档：`PIXEL_LAYOUT_GUIDE.md`
4. 探索代码修改功能

## 支持

遇到问题？查看：
- `PIXEL_LAYOUT_GUIDE.md` - 详细文档
- `README.md` - 项目概述
- `ENV_CONFIG.md` - 配置指南
