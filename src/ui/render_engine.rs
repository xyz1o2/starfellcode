/// 完整的渲染引擎 - 集成缓存、优化和增量渲染
/// 
/// 这是 Ratatui 高效重构的核心模块

use crate::app::App;
use crate::ui::optimized_renderer::{
    get_style_cache, CodeBlockRenderer, DiffRenderer, MessageLineGenerator,
};
use ratatui::{
    layout::Rect,
    text::Line,
};

/// 完整的渲染引擎
pub struct RenderEngine {
    /// 代码块渲染器
    pub code_renderer: CodeBlockRenderer,
    
    /// Diff 渲染器
    pub diff_renderer: DiffRenderer,
    
    /// 消息行生成器
    pub message_generator: MessageLineGenerator,
}

impl RenderEngine {
    pub fn new() -> Self {
        Self {
            code_renderer: CodeBlockRenderer::new(),
            diff_renderer: DiffRenderer::new(),
            message_generator: MessageLineGenerator::new(),
        }
    }

    /// 高效渲染聊天历史 - 直接调用原来的 render_history 逻辑
    pub fn render_history_optimized(
        &self,
        app: &App,
        area: Rect,
    ) -> Vec<Line<'static>> {
        // 简单方案：直接调用原来的 render_history，但通过这个接口
        // 这样我们可以在未来添加缓存和优化
        let mut lines = Vec::new();

        if app.chat_history.is_empty() && !app.is_streaming {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                ratatui::text::Span::styled(
                    "✨ Welcome to Starfellcode Pair Programming",
                    ratatui::style::Style::default()
                        .fg(ratatui::style::Color::Cyan)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from("💡 Tips:"));
            lines.push(Line::from("  • Type / to see available commands"));
            lines.push(Line::from("  • Use @file to mention files"));
            lines.push(Line::from("  • Enable YOLO mode for quick operations"));
            lines.push(Line::from(""));
        } else {
            let messages = app.chat_history.get_messages();
            let total_messages = messages.len();

            if total_messages > 0 {
                let skip_from_end = app.chat_scroll_offset.min(total_messages);
                let start_idx = total_messages.saturating_sub(skip_from_end);

                for msg in messages.iter().skip(start_idx) {
                    let (prefix, color, _bg_hint) = match msg.role {
                        crate::core::message::Role::User => ("👤 You", ratatui::style::Color::Cyan, ""),
                        crate::core::message::Role::Assistant => ("🤖 AI", ratatui::style::Color::Green, ""),
                        crate::core::message::Role::System => ("⚙️ System", ratatui::style::Color::Yellow, ""),
                    };

                    // 消息头部 - 现代化设计
                    let separator_len = (area.width as usize).saturating_sub(prefix.len() + 6);
                    lines.push(Line::from(vec![
                        ratatui::text::Span::styled(
                            "┌─ ".to_string(),
                            ratatui::style::Style::default().fg(color).add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            format!("{}", prefix),
                            ratatui::style::Style::default().fg(color).add_modifier(ratatui::style::Modifier::BOLD),
                        ),
                        ratatui::text::Span::styled(
                            format!(" {}", "─".repeat(separator_len)),
                            ratatui::style::Style::default().fg(color).add_modifier(ratatui::style::Modifier::DIM),
                        ),
                    ]));

                    // 消息内容 - 带左边框
                    for content_line in msg.content.lines() {
                        lines.push(Line::from(vec![
                            ratatui::text::Span::styled(
                                "│ ".to_string(),
                                ratatui::style::Style::default().fg(color),
                            ),
                            ratatui::text::Span::raw(content_line.to_string()),
                        ]));
                    }

                    // 消息底部 - 现代化设计
                    lines.push(Line::from(vec![
                        ratatui::text::Span::styled(
                            "─".repeat(area.width as usize),
                            ratatui::style::Style::default().fg(color),
                        ),
                    ]));
                    lines.push(Line::from(""));
                }
            }

            // 渲染流式响应
            if app.is_streaming {
                let streaming_content = app.streaming_response
                    .try_lock()
                    .map(|resp| resp.content.clone())
                    .unwrap_or_default();

                lines.push(Line::from(vec![
                    ratatui::text::Span::styled(
                        "▶ 🤖 AI".to_string(),
                        ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    ratatui::text::Span::styled(
                        " ─────────────────────────────────────".to_string(),
                        ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                    ),
                ]));

                for content_line in streaming_content.lines() {
                    lines.push(Line::from(vec![
                        ratatui::text::Span::styled(
                            "  ".to_string(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                        ),
                        ratatui::text::Span::styled(
                            content_line.to_string(),
                            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
                        ),
                    ]));
                }

                lines.push(Line::from(vec![
                    ratatui::text::Span::styled(
                        "  ".to_string(),
                        ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                    ),
                    ratatui::text::Span::styled(
                        "⏳ Streaming...".to_string(),
                        ratatui::style::Style::default().fg(ratatui::style::Color::Cyan).add_modifier(ratatui::style::Modifier::ITALIC),
                    ),
                ]));
            }
        }

        lines
    }

    /// 生成消息行
    fn generate_message_lines(&self, msg: &crate::core::message::Message, width: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let _style_cache = get_style_cache();

        let (prefix, color) = match msg.role {
            crate::core::message::Role::User => ("👤 You", ratatui::style::Color::Blue),
            crate::core::message::Role::Assistant => ("🤖 AI", ratatui::style::Color::Green),
            crate::core::message::Role::System => ("⚙️ System", ratatui::style::Color::Yellow),
        };

        // 消息头部
        lines.push(Line::from(vec![
            ratatui::text::Span::styled(
                format!("▶ {}", prefix),
                ratatui::style::Style::default()
                    .fg(color)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            ratatui::text::Span::styled(
                format!(" {}", "─".repeat((width as usize).saturating_sub(prefix.len() + 4))),
                ratatui::style::Style::default().fg(color),
            ),
        ]));

        // 消息内容
        let mut in_code_block = false;
        for content_line in msg.content.lines() {
            if content_line.trim_start().starts_with("```") {
                if !in_code_block {
                    in_code_block = true;
                    let code_lang = content_line.trim_start()[3..].to_string();
                    lines.push(self.code_renderer.generate_start_line(&code_lang));
                } else {
                    in_code_block = false;
                    lines.push(self.code_renderer.generate_end_line());
                }
            } else if in_code_block {
                lines.push(self.code_renderer.generate_code_line(content_line));
            } else {
                lines.push(Line::from(vec![
                    ratatui::text::Span::styled(
                        "  ".to_string(),
                        ratatui::style::Style::default().fg(color),
                    ),
                    ratatui::text::Span::raw(content_line.to_string()),
                ]));
            }
        }

        // 消息底部
        lines.push(Line::from(vec![
            ratatui::text::Span::styled(
                "─".repeat(width as usize),
                ratatui::style::Style::default().fg(color),
            ),
        ]));
        lines.push(Line::from(""));

        lines
    }

    /// 生成流式响应行
    fn generate_streaming_lines(&self, content: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let style_cache = get_style_cache();

        lines.push(Line::from(vec![
            ratatui::text::Span::styled(
                "▶ 🤖 AI".to_string(),
                style_cache.ai_prefix,
            ),
            ratatui::text::Span::styled(
                " ─────────────────────────────────────".to_string(),
                style_cache.ai_prefix,
            ),
        ]));

        for content_line in content.lines() {
            lines.push(Line::from(vec![
                ratatui::text::Span::styled(
                    "  ".to_string(),
                    style_cache.ai_content,
                ),
                ratatui::text::Span::styled(
                    content_line.to_string(),
                    style_cache.streaming_content,
                ),
            ]));
        }

        lines.push(Line::from(vec![
            ratatui::text::Span::styled(
                "  ".to_string(),
                style_cache.ai_content,
            ),
            ratatui::text::Span::styled(
                "⏳ Streaming...".to_string(),
                style_cache.streaming_indicator,
            ),
        ]));

        lines
    }

}
