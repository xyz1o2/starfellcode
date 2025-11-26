use crate::ui::types::{
    InfoSection, ModelInfoSection, TokenStatsSection, HelpInfoSection, 
    ErrorLogSection, SessionStatsSection, ErrorEntry, ErrorLevel, 
    ConnectionStatus
};
use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use chrono::Utc;
use std::time::Duration;

pub struct InfoPanel {
    pub sections: Vec<InfoSection>,
    pub active_section: usize,
}

impl InfoPanel {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            active_section: 0,
        }
    }

    /// Render the info panel
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme) {
        if self.sections.is_empty() {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ℹ️ Info ")
            .title_alignment(Alignment::Left)
            .border_style(theme.get_border_style(false));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &self.sections[self.active_section] {
            InfoSection::ModelInfo(section) => self.render_model_info_section(frame, section, inner, theme),
            InfoSection::TokenStats(section) => self.render_token_stats_section(frame, section, inner, theme),
            InfoSection::HelpInfo(section) => self.render_help_info_section(frame, section, inner, theme),
            InfoSection::ErrorLog(section) => self.render_error_log_section(frame, section, inner, theme),
            InfoSection::SessionStats(section) => self.render_session_stats_section(frame, section, inner, theme),
        }
    }

    /// Render model info section
    fn render_model_info_section(
        &self,
        frame: &mut Frame,
        section: &ModelInfoSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Model: ", theme.typography.body_style),
            Span::styled(&section.current_model, Style::default().fg(theme.colors.primary)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Provider: ", theme.typography.body_style),
            Span::styled(&section.provider, Style::default().fg(theme.colors.secondary)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Temperature: ", theme.typography.body_style),
            Span::styled(
                format!("{:.2}", section.temperature),
                theme.typography.body_style,
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Max Tokens: ", theme.typography.body_style),
            Span::styled(
                section.max_tokens.to_string(),
                theme.typography.body_style,
            ),
        ]));

        let (status_icon, status_color) = match &section.connection_status {
            ConnectionStatus::Connected => ("✅", theme.colors.success),
            ConnectionStatus::Connecting => ("⏳", theme.colors.warning),
            ConnectionStatus::Disconnected => ("❌", theme.colors.error),
            ConnectionStatus::Error(_) => ("🚨", theme.colors.error),
        };

        lines.push(Line::from(vec![
            Span::styled("Status: ", theme.typography.body_style),
            Span::styled(status_icon, Style::default().fg(status_color)),
        ]));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Render token stats section
    fn render_token_stats_section(
        &self,
        frame: &mut Frame,
        section: &TokenStatsSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Session Tokens: ", theme.typography.body_style),
            Span::styled(
                section.session_tokens.to_string(),
                Style::default().fg(theme.colors.primary),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Total Used: ", theme.typography.body_style),
            Span::styled(
                section.tokens_used.to_string(),
                theme.typography.body_style,
            ),
        ]));

        if let Some(remaining) = section.tokens_remaining {
            lines.push(Line::from(vec![
                Span::styled("Remaining: ", theme.typography.body_style),
                Span::styled(remaining.to_string(), theme.typography.body_style),
            ]));
        }

        if let Some(cost) = section.cost_estimate {
            lines.push(Line::from(vec![
                Span::styled("Est. Cost: $", theme.typography.body_style),
                Span::styled(
                    format!("{:.4}", cost),
                    Style::default().fg(theme.colors.warning),
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Render help info section
    fn render_help_info_section(
        &self,
        frame: &mut Frame,
        section: &HelpInfoSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),   // Context
                Constraint::Min(3),      // Shortcuts
                Constraint::Min(2),      // Tips
            ])
            .split(area);

        // Current context
        let context_line = Line::from(vec![
            Span::styled("Context: ", theme.typography.body_style),
            Span::styled(&section.current_context, Style::default().fg(theme.colors.primary)),
        ]);
        let context_paragraph = Paragraph::new(vec![context_line]);
        frame.render_widget(context_paragraph, chunks[0]);

        // Shortcuts
        let mut shortcut_items = Vec::new();
        for shortcut in &section.available_shortcuts {
            let item_text = format!("{}: {}", shortcut.key, shortcut.description);
            shortcut_items.push(ListItem::new(Line::from(Span::styled(
                item_text,
                theme.typography.body_style,
            ))));
        }

        let shortcuts_block = Block::default()
            .title("Shortcuts")
            .borders(Borders::TOP);
        let shortcuts_list = List::new(shortcut_items).block(shortcuts_block);
        frame.render_widget(shortcuts_list, chunks[1]);

        // Tips
        let mut tip_lines = Vec::new();
        for tip in &section.tips {
            tip_lines.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(theme.colors.info)),
                Span::styled(tip, theme.typography.caption_style),
            ]));
        }

        let tips_block = Block::default()
            .title("Tips")
            .borders(Borders::TOP);
        let tips_paragraph = Paragraph::new(tip_lines)
            .block(tips_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(tips_paragraph, chunks[2]);
    }

    /// Render error log section
    fn render_error_log_section(
        &self,
        frame: &mut Frame,
        section: &ErrorLogSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        if section.errors.is_empty() {
            let no_errors = Paragraph::new(Line::from(Span::styled(
                "No errors logged ✅",
                Style::default().fg(theme.colors.success),
            )));
            frame.render_widget(no_errors, area);
            return;
        }

        let mut error_items = Vec::new();
        for error in section.errors.iter().rev().take(10) { // Show last 10 errors
            let (level_icon, level_color) = match error.level {
                ErrorLevel::Info => ("ℹ️", theme.colors.info),
                ErrorLevel::Warning => ("⚠️", theme.colors.warning),
                ErrorLevel::Error => ("❌", theme.colors.error),
                ErrorLevel::Critical => ("🚨", theme.colors.error),
            };

            let timestamp = error.timestamp.format("%H:%M:%S").to_string();
            let item_text = format!("{} [{}] {}", level_icon, timestamp, error.message);
            
            error_items.push(ListItem::new(Line::from(Span::styled(
                item_text,
                Style::default().fg(level_color),
            ))));
        }

        let error_list = List::new(error_items);
        frame.render_widget(error_list, area);
    }

    /// Render session stats section
    fn render_session_stats_section(
        &self,
        frame: &mut Frame,
        section: &SessionStatsSection,
        area: Rect,
        theme: &ModernTheme,
    ) {
        let mut lines = Vec::new();

        // Session duration
        let duration_str = format!(
            "{}h {}m {}s",
            section.session_duration.as_secs() / 3600,
            (section.session_duration.as_secs() % 3600) / 60,
            section.session_duration.as_secs() % 60
        );

        lines.push(Line::from(vec![
            Span::styled("Duration: ", theme.typography.body_style),
            Span::styled(duration_str, Style::default().fg(theme.colors.primary)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Messages Sent: ", theme.typography.body_style),
            Span::styled(
                section.messages_sent.to_string(),
                theme.typography.body_style,
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Messages Received: ", theme.typography.body_style),
            Span::styled(
                section.messages_received.to_string(),
                theme.typography.body_style,
            ),
        ]));

        if let Some(avg_time) = section.average_response_time {
            lines.push(Line::from(vec![
                Span::styled("Avg Response: ", theme.typography.body_style),
                Span::styled(
                    format!("{:.1}s", avg_time.as_secs_f64()),
                    theme.typography.body_style,
                ),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Handle input events
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                if self.active_section > 0 {
                    self.active_section -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.active_section < self.sections.len().saturating_sub(1) {
                    self.active_section += 1;
                }
                true
            }
            KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') | KeyCode::Char('5') => {
                if let KeyCode::Char(c) = key.code {
                    let index = (c as u8 - b'1') as usize;
                    if index < self.sections.len() {
                        self.active_section = index;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Update model info
    pub fn update_model_info(&mut self, model: String, provider: String, connection: ConnectionStatus) {
        for section in &mut self.sections {
            if let InfoSection::ModelInfo(model_section) = section {
                model_section.current_model = model;
                model_section.provider = provider;
                model_section.connection_status = connection;
                break;
            }
        }
    }

    /// Update token stats
    pub fn update_token_stats(&mut self, session_tokens: u32, total_tokens: u32) {
        for section in &mut self.sections {
            if let InfoSection::TokenStats(token_section) = section {
                token_section.session_tokens = session_tokens;
                token_section.tokens_used = total_tokens;
                break;
            }
        }
    }

    /// Add error to log
    pub fn add_error(&mut self, level: ErrorLevel, message: String, details: Option<String>) {
        for section in &mut self.sections {
            if let InfoSection::ErrorLog(error_section) = section {
                let error_entry = ErrorEntry {
                    timestamp: Utc::now(),
                    level,
                    message,
                    details,
                };
                
                error_section.errors.push(error_entry);
                
                // Limit error log size
                if error_section.errors.len() > error_section.max_entries {
                    error_section.errors.remove(0);
                }
                break;
            }
        }
    }

    /// Update session stats
    pub fn update_session_stats(&mut self, duration: Duration, sent: u32, received: u32, avg_response: Option<Duration>) {
        for section in &mut self.sections {
            if let InfoSection::SessionStats(stats_section) = section {
                stats_section.session_duration = duration;
                stats_section.messages_sent = sent;
                stats_section.messages_received = received;
                stats_section.average_response_time = avg_response;
                break;
            }
        }
    }

    /// Cycle to next section
    pub fn cycle_section(&mut self) {
        self.active_section = (self.active_section + 1) % self.sections.len();
    }

    /// Get active section index
    pub fn get_active_section(&self) -> usize {
        self.active_section
    }

    /// Set active section
    pub fn set_active_section(&mut self, index: usize) {
        if index < self.sections.len() {
            self.active_section = index;
        }
    }
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self::new()
    }
}