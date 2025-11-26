use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

/// 提及建议系统
#[derive(Debug, Clone)]
pub struct MentionSuggestions {
    pub visible: bool,
    pub trigger: char,  // '@' 或 '/'
    pub query: String,
    pub suggestions: Vec<String>,
    pub selected_index: usize,
    pub state: ListState,  // 列表状态
}

impl MentionSuggestions {
    pub fn new() -> Self {
        Self {
            visible: false,
            trigger: '@',
            query: String::new(),
            suggestions: Vec::new(),
            selected_index: 0,
            state: ListState::default(),
        }
    }

    /// 激活提及建议（当检测到 @ 时）
    pub fn activate(&mut self, trigger: char) {
        self.visible = true;
        self.trigger = trigger;
        self.query = "@".to_string();  // 初始查询为 @
        self.selected_index = 0;
        self.refresh_suggestions();  // 立即刷新建议
    }

    /// 更新查询字符串并刷新建议
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
        self.refresh_suggestions();
    }

    /// 刷新建议列表
    fn refresh_suggestions(&mut self) {
        self.suggestions.clear();

        if self.trigger == '@' {
            // 文件/文件夹建议
            self.suggestions = self.get_file_suggestions(&self.query);
            
            // 如果没有找到建议，添加测试信息
            if self.suggestions.is_empty() {
                self.suggestions.push(format!("DEBUG: Query='{}' trigger='{}'", self.query, self.trigger));
                self.suggestions.push("No files found - checking directory...".to_string());
            }
        }

        // 始终保持可见（即使没有建议也显示提示）
        // 重要：这必须在添加测试信息之后
        self.visible = !self.suggestions.is_empty();
    }

    /// 获取文件建议
    fn get_file_suggestions(&self, query: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 移除 @ 符号和空格，获取搜索路径
        let search_query = query.trim_start_matches('@').trim();
        
        // 简单的逻辑：
        // @src -> 列出当前目录中以 "src" 开头的文件
        // @src/m -> 列出 src 目录中以 "m" 开头的文件
        
        let (search_dir, filter_prefix) = if search_query.is_empty() {
            // 只有 @ - 列出当前目录所有文件
            (".".to_string(), String::new())
        } else if search_query.contains('/') {
            // 包含 / - 分割目录和前缀
            if let Some(last_slash) = search_query.rfind('/') {
                let dir = &search_query[..last_slash];
                let prefix = &search_query[last_slash + 1..];
                (dir.to_string(), prefix.to_string())
            } else {
                (".".to_string(), search_query.to_string())
            }
        } else {
            // 没有 / - 在当前目录搜索
            (".".to_string(), search_query.to_string())
        };

        // 尝试列出目录
        match std::fs::read_dir(&search_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            // 跳过 target 目录
                            if file_name == "target" {
                                continue;
                            }

                            // 检查前缀匹配
                            if !filter_prefix.is_empty() && !file_name.starts_with(&filter_prefix) {
                                continue;
                            }

                            // 构建显示路径
                            let display_path = if search_dir == "." {
                                format!("@{}", file_name)
                            } else {
                                format!("@{}/{}", search_dir, file_name)
                            };

                            // 添加目录标记
                            let display = if metadata.is_dir() {
                                format!("{}/", display_path)
                            } else {
                                display_path
                            };

                            suggestions.push(display);
                        }
                    }
                }
            }
            Err(_) => {
                // 目录不存在或无法读取
                suggestions.push(format!("Cannot read directory: {}", search_dir));
            }
        }

        // 排序并限制数量
        suggestions.sort();
        suggestions.truncate(10);
        suggestions
    }

    /// 向上选择
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.suggestions.is_empty() {
            self.selected_index = self.suggestions.len() - 1;
        }
        self.state.select(Some(self.selected_index));
    }

    /// 向下选择
    pub fn select_next(&mut self) {
        if self.selected_index < self.suggestions.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
        self.state.select(Some(self.selected_index));
    }

    /// 获取当前选中的建议
    pub fn get_selected(&self) -> Option<String> {
        self.suggestions.get(self.selected_index).cloned()
    }

    /// 关闭建议
    pub fn close(&mut self) {
        self.visible = false;
        self.suggestions.clear();
        self.query.clear();
    }

    /// 渲染建议列表
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.visible || self.suggestions.is_empty() {
            return;
        }

        let items: Vec<ListItem> = self
            .suggestions
            .iter()
            .map(|suggestion| {
                ListItem::new(suggestion.clone())
                    .style(Style::default().fg(Color::Cyan))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title("📁 文件建议")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }
}
