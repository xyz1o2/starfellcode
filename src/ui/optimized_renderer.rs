/// 高效的 Ratatui 优化渲染器
/// 
/// 核心优化：
/// - 预计算样式对象（避免重复创建）
/// - 缓存常用字符串
/// - 增量渲染
/// - 流式响应优化

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::sync::OnceLock;

/// 预计算的样式对象 - 避免重复创建
pub struct StyleCache {
    // 基础样式
    pub header_title: Style,
    pub header_info: Style,
    pub separator: Style,
    
    // 消息样式
    pub user_prefix: Style,
    pub user_content: Style,
    pub ai_prefix: Style,
    pub ai_content: Style,
    pub system_prefix: Style,
    pub system_content: Style,
    
    // 代码块样式
    pub code_border: Style,
    pub code_content: Style,
    
    // Diff 样式
    pub diff_added: Style,
    pub diff_removed: Style,
    pub diff_border: Style,
    
    // 输入框样式
    pub input_hint: Style,
    pub input_border: Style,
    
    // 流式响应
    pub streaming_indicator: Style,
    pub streaming_content: Style,
}

impl StyleCache {
    pub fn new() -> Self {
        Self {
            // Header
            header_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            header_info: Style::default().fg(Color::Yellow),
            separator: Style::default().fg(Color::DarkGray),
            
            // Messages
            user_prefix: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            user_content: Style::default().fg(Color::Blue),
            ai_prefix: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            ai_content: Style::default().fg(Color::Green),
            system_prefix: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            system_content: Style::default().fg(Color::Yellow),
            
            // Code
            code_border: Style::default().fg(Color::Magenta),
            code_content: Style::default().fg(Color::Yellow),
            
            // Diff
            diff_added: Style::default().fg(Color::Green),
            diff_removed: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::DIM),
            diff_border: Style::default().fg(Color::Magenta),
            
            // Input
            input_hint: Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            input_border: Style::default().fg(Color::Cyan),
            
            // Streaming
            streaming_indicator: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
            streaming_content: Style::default().fg(Color::Cyan),
        }
    }
}

/// 全局样式缓存
pub fn get_style_cache() -> &'static StyleCache {
    static STYLE_CACHE: OnceLock<StyleCache> = OnceLock::new();
    STYLE_CACHE.get_or_init(StyleCache::new)
}

/// 字符串缓存 - 避免重复分配
pub struct StringCache {
    pub separator_line: String,
    pub code_start: String,
    pub code_end: String,
    pub diff_start: String,
    pub diff_end: String,
}

impl StringCache {
    pub fn new(width: u16) -> Self {
        let width = width as usize;
        Self {
            separator_line: "─".repeat(width),
            code_start: "┌─ Code".to_string(),
            code_end: "└─".to_string(),
            diff_start: "┌─ Diff 对比".to_string(),
            diff_end: "└─".to_string(),
        }
    }

    pub fn update_width(&mut self, width: u16) {
        let width = width as usize;
        self.separator_line = "─".repeat(width);
    }
}

/// 高效的行构建器 - 减少临时分配
pub struct LineBuilder {
    spans: Vec<Span<'static>>,
}

impl LineBuilder {
    pub fn new() -> Self {
        Self {
            spans: Vec::with_capacity(8),
        }
    }

    pub fn add_styled(mut self, text: String, style: Style) -> Self {
        self.spans.push(Span::styled(text, style));
        self
    }

    pub fn add_raw(mut self, text: &str) -> Self {
        self.spans.push(Span::raw(text.to_string()));
        self
    }

    pub fn build(self) -> Line<'static> {
        Line::from(self.spans)
    }

    pub fn clear(&mut self) {
        self.spans.clear();
    }
}

/// 消息行生成器 - 优化消息渲染
pub struct MessageLineGenerator {
    style_cache: &'static StyleCache,
}

impl MessageLineGenerator {
    pub fn new() -> Self {
        Self {
            style_cache: get_style_cache(),
        }
    }

    /// 生成消息头部
    pub fn generate_message_header(
        &self,
        prefix: &str,
        role_color: Color,
        width: u16,
    ) -> Line<'static> {
        let style = Style::default()
            .fg(role_color)
            .add_modifier(Modifier::BOLD);
        
        LineBuilder::new()
            .add_styled(format!("▶ {}", prefix), style)
            .add_styled(
                format!(" {}", "─".repeat((width as usize).saturating_sub(prefix.len() + 4))),
                style,
            )
            .build()
    }

    /// 生成消息内容行
    pub fn generate_content_line(
        &self,
        content: &str,
        role_color: Color,
    ) -> Line<'static> {
        LineBuilder::new()
            .add_styled("  ".to_string(), Style::default().fg(role_color))
            .add_raw(content)
            .build()
    }

    /// 生成分隔线
    pub fn generate_separator(&self, role_color: Color, width: u16) -> Line<'static> {
        LineBuilder::new()
            .add_styled("─".repeat(width as usize), Style::default().fg(role_color))
            .build()
    }
}

/// 代码块渲染器 - 优化代码块显示
pub struct CodeBlockRenderer {
    style_cache: &'static StyleCache,
}

impl CodeBlockRenderer {
    pub fn new() -> Self {
        Self {
            style_cache: get_style_cache(),
        }
    }

    /// 生成代码块开始行
    pub fn generate_start_line(&self, language: &str) -> Line<'static> {
        let lang_display = if language.is_empty() { "text" } else { language };
        
        Line::from(vec![
            Span::styled(
                "  ┌─ Code".to_string(),
                self.style_cache.code_border,
            ),
            Span::styled(
                format!(" ({})", lang_display),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::DIM),
            ),
        ])
    }

    /// 生成代码行
    pub fn generate_code_line(&self, code: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  │ ".to_string(),
                self.style_cache.code_border,
            ),
            Span::styled(
                code.to_string(),
                self.style_cache.code_content,
            ),
        ])
    }

    /// 生成代码块结束行
    pub fn generate_end_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  └─".to_string(),
                self.style_cache.code_border,
            ),
        ])
    }
}

/// Diff 渲染器 - 优化 Diff 显示
pub struct DiffRenderer {
    style_cache: &'static StyleCache,
}

impl DiffRenderer {
    pub fn new() -> Self {
        Self {
            style_cache: get_style_cache(),
        }
    }

    /// 生成 Diff 开始行
    pub fn generate_start_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  ┌─ Diff 对比".to_string(),
                self.style_cache.diff_border,
            ),
        ])
    }

    /// 生成移除行
    pub fn generate_removed_line(&self, content: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  │ - ".to_string(),
                self.style_cache.diff_border,
            ),
            Span::styled(
                content.to_string(),
                self.style_cache.diff_removed,
            ),
        ])
    }

    /// 生成添加行
    pub fn generate_added_line(&self, content: &str) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  │ + ".to_string(),
                self.style_cache.diff_border,
            ),
            Span::styled(
                content.to_string(),
                self.style_cache.diff_added,
            ),
        ])
    }

    /// 生成 Diff 结束行
    pub fn generate_end_line(&self) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                "  └─".to_string(),
                self.style_cache.diff_border,
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_cache() {
        let cache = get_style_cache();
        assert_eq!(cache.header_title.fg, Some(Color::Cyan));
    }

    #[test]
    fn test_line_builder() {
        let builder = LineBuilder::new();
        let line = builder
            .add_styled("Hello".to_string(), Style::default())
            .add_raw(" World")
            .build();
        
        assert_eq!(line.spans.len(), 2);
    }

    #[test]
    fn test_code_block_renderer() {
        let renderer = CodeBlockRenderer::new();
        let start = renderer.generate_start_line("rust");
        assert!(!start.spans.is_empty());
    }
}
