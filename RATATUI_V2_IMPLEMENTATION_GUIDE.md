# 🎮 Ratatui 像素CLI聊天界面完整实现指南

## 目录
1. [核心依赖](#核心依赖)
2. [应用程序结构](#应用程序结构)
3. [整体布局](#整体布局)
4. [组件详解](#组件详解)
5. [样式与特效](#样式与特效)
6. [完整代码示例](#完整代码示例)

---

## 核心依赖

### Cargo.toml 配置

```toml
[dependencies]
ratatui = "0.26"
crossterm = { version = "0.27", features = ["event-stream"] }
tokio = { version = "1", features = ["full"] }
```

**为什么这些？**
- **ratatui**: 核心TUI库，提供 Layout、Widget、Style 等
- **crossterm**: 终端事件处理（键盘输入、鼠标）
- **tokio**: 异步运行时，用于处理流式AI响应

---

## 应用程序结构

### 1. 数据结构 (State)

```rust
use ratatui::style::Color;

// 消息结构体
#[derive(Clone, Debug)]
pub struct Message {
    pub role: Role,           // User / Assistant / System
    pub avatar_key: String,   // "user" / "ai" / "sys"
    pub content: String,      // 消息文本
    pub code_block: Option<CodeBlock>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

// 代码块结构体
#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub language: String,
    pub lines: Vec<CodeLine>,
}

#[derive(Clone, Debug)]
pub struct CodeLine {
    pub number: usize,
    pub content: String,
    pub status: LineStatus, // "add" / "rem" / ""
}

#[derive(Clone, Debug, PartialEq)]
pub enum LineStatus {
    Added,
    Removed,
    Normal,
}

// 像素头像数据
#[derive(Clone)]
pub struct PixelData {
    pub color: Color,
    pub map: [u8; 64], // 8x8 = 64 像素
}

// 应用状态
pub struct App {
    pub messages: Vec<Message>,
    pub input_text: String,
    pub scroll_offset: u16,
    pub input_cursor: usize,
    pub avatars: HashMap<String, PixelData>,
}

impl App {
    pub fn new() -> Self {
        let mut avatars = HashMap::new();
        
        // 初始化头像数据（从 v2.html 复制）
        avatars.insert("sys".to_string(), PixelData {
            color: Color::Rgb(34, 211, 238),  // #22d3ee
            map: [0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, ...],
        });
        
        avatars.insert("user".to_string(), PixelData {
            color: Color::Rgb(244, 114, 182), // #f472b6
            map: [0,0,1,1,1,1,0,0, 0,1,1,1,1,1,1,0, ...],
        });
        
        // ... 其他头像
        
        Self {
            messages: vec![],
            input_text: String::new(),
            scroll_offset: 0,
            input_cursor: 0,
            avatars,
        }
    }
}
```

### 2. 主循环结构

```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化终端
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;
    
    // 应用状态
    let mut app = App::new();
    
    // 主循环
    loop {
        // 1. 绘制UI
        terminal.draw(|f| ui(f, &app))?;
        
        // 2. 处理事件
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Enter => {
                        handle_submit(&mut app).await;
                    }
                    KeyCode::Backspace => {
                        if app.input_cursor > 0 {
                            app.input_text.remove(app.input_cursor - 1);
                            app.input_cursor -= 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input_text.insert(app.input_cursor, c);
                        app.input_cursor += 1;
                    }
                    KeyCode::Up => {
                        app.scroll_offset = app.scroll_offset.saturating_add(3);
                    }
                    KeyCode::Down => {
                        app.scroll_offset = app.scroll_offset.saturating_sub(3);
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}

async fn handle_submit(app: &mut App) {
    // 添加用户消息
    app.messages.push(Message {
        role: Role::User,
        avatar_key: "user".to_string(),
        content: app.input_text.clone(),
        code_block: None,
    });
    
    // 清空输入
    app.input_text.clear();
    app.input_cursor = 0;
    
    // 模拟AI响应（实际应该调用LLM API）
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    app.messages.push(Message {
        role: Role::Assistant,
        avatar_key: "ai".to_string(),
        content: "这是AI的响应...".to_string(),
        code_block: None,
    });
}
```

---

## 整体布局

### 主布局分割

```rust
use ratatui::layout::{Layout, Direction, Constraint};

fn ui(f: &mut Frame, app: &App) {
    // 背景
    f.render_widget(
        Block::default().bg(Color::Rgb(12, 12, 12)),
        f.size()
    );
    
    // 垂直分割：历史 | 状态栏 | 输入
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),      // 历史区（占据所有剩余空间）
            Constraint::Length(1),   // 状态栏
            Constraint::Length(8),   // 输入区（8行高，容纳8x8头像）
        ])
        .split(f.size());
    
    // 渲染各部分
    render_history(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
    render_input_area(f, app, chunks[2]);
}
```

---

## 组件详解

### 1. 历史区域渲染

```rust
fn render_history(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    
    for msg in &app.messages {
        // 获取头像数据
        let avatar_data = &app.avatars[&msg.avatar_key];
        let avatar_lines = render_avatar(avatar_data);
        
        // 构建消息行
        let mut msg_lines = vec![];
        
        // 角色标签
        let role_color = match msg.role {
            Role::User => Color::Rgb(244, 114, 182),
            Role::Assistant => Color::Rgb(34, 211, 238),
            Role::System => Color::Yellow,
        };
        
        msg_lines.push(Line::from(Span::styled(
            format!("{:?}", msg.role).to_uppercase(),
            Style::default().fg(role_color).add_modifier(Modifier::BOLD),
        )));
        
        // 消息内容
        for line in msg.content.lines() {
            msg_lines.push(Line::from(line));
        }
        
        // 代码块
        if let Some(code) = &msg.code_block {
            msg_lines.push(Line::from(""));
            for code_line in &code.lines {
                let style = match code_line.status {
                    LineStatus::Added => Style::default()
                        .bg(Color::Rgb(15, 57, 28))
                        .fg(Color::Rgb(74, 222, 128)),
                    LineStatus::Removed => Style::default()
                        .bg(Color::Rgb(63, 19, 19))
                        .fg(Color::Rgb(248, 113, 113)),
                    LineStatus::Normal => Style::default(),
                };
                
                msg_lines.push(Line::from(Span::styled(
                    format!("{:3} | {}", code_line.number, code_line.content),
                    style,
                )));
            }
        }
        
        // 合并头像和消息（水平布局）
        let max_height = avatar_lines.len().max(msg_lines.len());
        for i in 0..max_height {
            let mut spans = Vec::new();
            
            // 头像列
            if let Some(avatar_line) = avatar_lines.get(i) {
                spans.extend(avatar_line.spans.clone());
            } else {
                spans.push(Span::raw(" ".repeat(16)));
            }
            
            // 间隔
            spans.push(Span::raw("  "));
            
            // 消息列
            if let Some(msg_line) = msg_lines.get(i) {
                spans.extend(msg_line.spans.clone());
            }
            
            lines.push(Line::from(spans));
        }
        
        // 消息间隔
        lines.push(Line::from(""));
    }
    
    // 渲染
    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset, 0));
    
    f.render_widget(para, area);
}
```

### 2. 像素头像渲染（核心技巧）

```rust
fn render_avatar(avatar_data: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    for row in 0..8 {
        let mut spans = Vec::new();
        
        for col in 0..8 {
            let idx = row * 8 + col;
            let pixel = avatar_data.map[idx];
            
            let style = match pixel {
                0 => Style::default(),                    // 透明
                1 => Style::default().bg(avatar_data.color), // 主体色
                2 => Style::default().bg(Color::White),   // 眼睛
                _ => Style::default(),
            };
            
            // 关键：用两个空格来模拟像素
            spans.push(Span::styled("  ", style));
        }
        
        lines.push(Line::from(spans));
    }
    
    lines
}
```

### 3. 状态栏

```rust
fn render_status_bar(f: &mut Frame, _app: &App, area: Rect) {
    let status_line = Line::from(vec![
        Span::styled(
            "STATUS: CONNECTED",
            Style::default().fg(Color::Rgb(119, 119, 119)),
        ),
        Span::raw(" ".repeat(area.width.saturating_sub(30) as usize)),
        Span::styled(
            "CTRL+C to EXIT",
            Style::default().fg(Color::Rgb(119, 119, 119)),
        ),
    ]);
    
    let para = Paragraph::new(status_line)
        .style(Style::default().bg(Color::Rgb(34, 34, 34)));
    
    f.render_widget(para, area);
}
```

### 4. 输入区域

```rust
fn render_input_area(f: &mut Frame, app: &App, area: Rect) {
    // 水平分割：头像 | 箭头 | 输入框
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(16),  // 头像
            Constraint::Length(2),   // 箭头
            Constraint::Min(10),     // 输入框
        ])
        .split(area);
    
    // 背景
    f.render_widget(
        Block::default().bg(Color::Rgb(8, 8, 8)),
        area
    );
    
    // 1. 渲染头像
    let user_avatar = &app.avatars["user"];
    let avatar_lines = render_avatar(user_avatar);
    f.render_widget(Paragraph::new(avatar_lines), chunks[0]);
    
    // 2. 渲染箭头（带脉冲效果）
    let arrow = "▶";
    f.render_widget(
        Paragraph::new(arrow).style(
            Style::default()
                .fg(Color::Rgb(244, 114, 182))
                .add_modifier(Modifier::BOLD)
        ),
        chunks[1]
    );
    
    // 3. 渲染输入框
    let input_para = Paragraph::new(app.input_text.as_str())
        .style(Style::default().fg(Color::White));
    f.render_widget(input_para, chunks[2]);
    
    // 4. 显示光标
    f.set_cursor(
        chunks[2].x + app.input_cursor as u16,
        chunks[2].y,
    );
}
```

---

## 样式与特效

### 颜色常量

```rust
pub mod colors {
    use ratatui::style::Color;
    
    pub const BG_COLOR: Color = Color::Rgb(12, 12, 12);       // #0c0c0c
    pub const PANEL_BG: Color = Color::Rgb(17, 17, 17);       // #111
    pub const BORDER: Color = Color::Rgb(51, 51, 51);         // #333
    pub const ACCENT_AI: Color = Color::Rgb(34, 211, 238);    // #22d3ee
    pub const ACCENT_USER: Color = Color::Rgb(244, 114, 182); // #f472b6
    pub const DIFF_ADD: Color = Color::Rgb(15, 57, 28);       // #0f391c
    pub const DIFF_ADD_TEXT: Color = Color::Rgb(74, 222, 128);// #4ade80
    pub const DIFF_REM: Color = Color::Rgb(63, 19, 19);       // #3f1313
    pub const DIFF_REM_TEXT: Color = Color::Rgb(248, 113, 113);// #f87171
}
```

### 扫描线效果（可选）

```rust
fn add_scanlines(f: &mut Frame) {
    let buffer = f.buffer_mut();
    
    for y in 0..buffer.area.height {
        // 每4行中的2行变暗
        if y % 4 >= 2 {
            for x in 0..buffer.area.width {
                let cell = buffer.get_mut(x, y);
                // 混合背景色
                if let Some(bg) = cell.bg {
                    cell.set_bg(Color::Rgb(10, 10, 10));
                }
            }
        }
    }
}
```

---

## 完整代码示例

### 最小化可运行示例

```rust
use ratatui::prelude::*;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io;

fn main() -> io::Result<()> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // 应用循环
    let mut app = App::new();
    
    loop {
        terminal.draw(|f| ui(f, &app))?;
        
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => break,
                    KeyCode::Char(c) => {
                        app.input_text.push(c);
                        app.input_cursor += 1;
                    }
                    KeyCode::Backspace => {
                        if app.input_cursor > 0 {
                            app.input_text.remove(app.input_cursor - 1);
                            app.input_cursor -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    
    Ok(())
}
```

---

## 关键要点总结

| 概念 | HTML/CSS | Ratatui |
|------|----------|---------|
| 容器 | `<div class="tui-container">` | `Layout::vertical(...)` |
| 消息行 | `.msg-row` (flex) | `Layout::horizontal(...)` |
| 头像 | `.avatar-box` (grid) | `render_avatar()` 函数 |
| 文本 | `<div>` | `Paragraph` |
| 样式 | CSS 变量 | `Style::default().fg(...).bg(...)` |
| 滚动 | CSS `overflow-y` | `Paragraph::scroll(...)` |
| 光标 | 浏览器原生 | `frame.set_cursor(...)` |

---

## 下一步

1. ✅ 实现基本布局和消息渲染
2. ⏳ 集成真实的LLM API调用
3. ⏳ 添加流式响应支持
4. ⏳ 实现代码块语法高亮
5. ⏳ 添加扫描线和其他视觉效果

