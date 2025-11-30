# 🎯 Ratatui 快速参考卡 - V2.html 复刻

## 1. 核心概念速查

### Layout（布局）
```rust
// 垂直分割
Layout::vertical([
    Constraint::Min(0),      // 占据所有剩余空间
    Constraint::Length(1),   // 固定1行
    Constraint::Percentage(50), // 50%
])

// 水平分割
Layout::horizontal([
    Constraint::Length(16),  // 固定16列
    Constraint::Min(20),     // 最少20列
])
```

### Widget（组件）
```rust
// 文本段落
Paragraph::new("text")
    .wrap(Wrap { trim: true })
    .scroll((offset_x, offset_y))

// 块（带边框）
Block::default()
    .borders(Borders::ALL)
    .title("Title")
    .border_style(Style::default().fg(Color::Cyan))

// 清空区域
Clear
```

### Style（样式）
```rust
// 颜色
Color::Rgb(r, g, b)
Color::White / Color::Black
Color::Cyan / Color::Magenta

// 修饰符
Modifier::BOLD
Modifier::ITALIC
Modifier::UNDERLINED
Modifier::DIM

// 组合
Style::default()
    .fg(Color::Cyan)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD)
```

### Text（文本）
```rust
// Span（单个样式段）
Span::raw("text")
Span::styled("text", style)

// Line（一行）
Line::from("text")
Line::from(vec![span1, span2, span3])

// Text（多行）
Text::from(vec![line1, line2])
```

---

## 2. 像素头像快速实现

```rust
// 数据
pub struct PixelData {
    pub color: Color,
    pub map: [u8; 64], // 8x8 = 64
}

// 渲染
fn render_avatar(data: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for row in 0..8 {
        let mut spans = Vec::new();
        for col in 0..8 {
            let pixel = data.map[row * 8 + col];
            let style = match pixel {
                0 => Style::default(),
                1 => Style::default().bg(data.color),
                2 => Style::default().bg(Color::White),
                _ => Style::default(),
            };
            spans.push(Span::styled("  ", style));
        }
        lines.push(Line::from(spans));
    }
    lines
}
```

---

## 3. 消息渲染快速模板

```rust
fn render_message(msg: &Message, avatar_data: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    // 头像
    let avatar_lines = render_avatar(avatar_data);
    
    // 内容
    let mut content_lines = vec![];
    content_lines.push(Line::from(Span::styled(
        "USER",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    for line in msg.content.lines() {
        content_lines.push(Line::from(line));
    }
    
    // 合并
    let max_height = avatar_lines.len().max(content_lines.len());
    for i in 0..max_height {
        let mut spans = Vec::new();
        
        // 头像列
        if let Some(line) = avatar_lines.get(i) {
            spans.extend(line.spans.clone());
        } else {
            spans.push(Span::raw(" ".repeat(16)));
        }
        
        // 间隔
        spans.push(Span::raw("  "));
        
        // 内容列
        if let Some(line) = content_lines.get(i) {
            spans.extend(line.spans.clone());
        }
        
        lines.push(Line::from(spans));
    }
    
    lines
}
```

---

## 4. 主循环框架

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    
    let mut app = App::new();
    
    // 主循环
    loop {
        // 绘制
        terminal.draw(|f| ui(f, &app))?;
        
        // 事件
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Enter => handle_submit(&mut app).await,
                    KeyCode::Char(c) => app.input_text.push(c),
                    KeyCode::Backspace => { app.input_text.pop(); }
                    KeyCode::Up => app.scroll_offset = app.scroll_offset.saturating_add(3),
                    KeyCode::Down => app.scroll_offset = app.scroll_offset.saturating_sub(3),
                    _ => {}
                }
            }
        }
    }
    
    // 清理
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(8),
    ]).split(f.size());
    
    render_history(f, app, chunks[0]);
    render_status_bar(f, app, chunks[1]);
    render_input_area(f, app, chunks[2]);
}
```

---

## 5. 颜色常量速查

```rust
// 背景
#0c0c0c → Color::Rgb(12, 12, 12)
#111    → Color::Rgb(17, 17, 17)
#222    → Color::Rgb(34, 34, 34)
#333    → Color::Rgb(51, 51, 51)

// 强调色
#22d3ee (Cyan)   → Color::Rgb(34, 211, 238)
#f472b6 (Pink)   → Color::Rgb(244, 114, 182)
#facc15 (Yellow) → Color::Rgb(250, 204, 21)
#ef4444 (Red)    → Color::Rgb(239, 68, 68)

// Diff
Added:   #0f391c (bg) / #4ade80 (fg)
Removed: #3f1313 (bg) / #f87171 (fg)
```

---

## 6. 常见问题速解

### Q: 头像不显示？
A: 检查 `map` 数组是否正确，确保使用 `bg()` 而不是 `fg()`

### Q: 文本没有换行？
A: 使用 `.wrap(Wrap { trim: true })`

### Q: 光标位置错误？
A: 使用 `frame.set_cursor(x + offset, y)` 而不是绝对位置

### Q: 滚动不工作？
A: 确保使用 `.scroll((offset_x, offset_y))` 在 Paragraph 上

### Q: 消息重叠？
A: 检查 `max_height` 计算，确保头像和内容正确对齐

---

## 7. 调试技巧

```rust
// 打印调试信息到日志
eprintln!("Debug: {:?}", value);

// 显示区域边界（用于调试布局）
f.render_widget(
    Block::default().borders(Borders::ALL),
    area
);

// 检查消息高度
let height = msg.content.lines().count();
eprintln!("Message height: {}", height);
```

---

## 8. 性能优化检查清单

- [ ] 使用 `scroll()` 而不是重新渲染所有行
- [ ] 缓存头像渲染结果
- [ ] 避免在每一帧重新计算布局
- [ ] 使用异步处理AI响应
- [ ] 限制历史消息数量（例如，只保留最后100条）

---

## 9. 常用快捷键映射

```rust
KeyCode::Char('c') + CONTROL → 退出
KeyCode::Enter               → 提交
KeyCode::Backspace           → 删除
KeyCode::Up / Down           → 滚动
KeyCode::Left / Right        → 光标移动
KeyCode::Home / End          → 行首/行尾
KeyCode::PageUp / PageDown   → 快速滚动
```

---

## 10. 完整颜色主题

```rust
pub struct Theme {
    pub bg: Color,           // #0c0c0c
    pub panel_bg: Color,     // #111
    pub border: Color,       // #333
    pub accent_ai: Color,    // #22d3ee
    pub accent_user: Color,  // #f472b6
    pub diff_add: Color,     // #0f391c
    pub diff_add_text: Color,// #4ade80
    pub diff_rem: Color,     // #3f1313
    pub diff_rem_text: Color,// #f87171
}

impl Theme {
    pub fn new() -> Self {
        Self {
            bg: Color::Rgb(12, 12, 12),
            panel_bg: Color::Rgb(17, 17, 17),
            border: Color::Rgb(51, 51, 51),
            accent_ai: Color::Rgb(34, 211, 238),
            accent_user: Color::Rgb(244, 114, 182),
            diff_add: Color::Rgb(15, 57, 28),
            diff_add_text: Color::Rgb(74, 222, 128),
            diff_rem: Color::Rgb(63, 19, 19),
            diff_rem_text: Color::Rgb(248, 113, 113),
        }
    }
}
```

