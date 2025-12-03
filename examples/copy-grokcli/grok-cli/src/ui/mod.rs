use ratatui::{
    prelude::{CrosstermBackend, Rect},
    Terminal as RatatuiTerminal,
    widgets::{Block, Borders, Paragraph, List, ListItem},
    layout::{Layout, Direction, Constraint},
    style::{Style, Color},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use crate::agent::GrokAgent;
use crate::types::{ChatEntry, ChatEntryType};

pub struct ChatState {
    chat_history: Vec<ChatEntry>,
    input: String,
    scroll: u16,
    show_command_hints: bool,
    command_hints: Vec<String>,
    selected_hint: usize,
    show_mention_hints: bool,
    mention_hints: Vec<String>,
    selected_mention_hint: usize,
}

const AVAILABLE_COMMANDS: &[&str] = &[
    "/help - Show help information",
    "/clear - Clear chat history",
    "/models - Switch Grok Model",
    "/commit-and-push - AI commit & push to remote",
    "/exit - Exit the application",
];

const AVAILABLE_MENTIONS: &[&str] = &[
    "@file - Mention a file",
    "@model - Mention current model",
    "@provider - Mention current provider",
    "@history - Mention chat history",
];

fn get_welcome_message() -> String {
    "🤖 Welcome to starfellcode CLI!\n\n\
    Tips for getting started:\n\
    1. Ask questions, edit files, or run commands.\n\
    2. Be specific for the best results.\n\
    3. Create GROK.md files to customize your interactions.\n\
    4. Press Shift+Tab to toggle auto-edit mode.\n\
    5. /help for more information.\n\n\
    Type your request in natural language. Ctrl+C to clear, 'exit' to quit.".to_string()
}

pub async fn run_app(mut agent: GrokAgent, initial_message: String) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = RatatuiTerminal::new(backend)?;

    let mut chat_state = ChatState {
        chat_history: vec![],
        input: String::new(),
        scroll: 0,
        show_command_hints: false,
        command_hints: vec![],
        selected_hint: 0,
        show_mention_hints: false,
        mention_hints: vec![],
        selected_mention_hint: 0,
    };

    // If there's an initial message, process it first
    if !initial_message.trim().is_empty() {
        chat_state.chat_history.push(ChatEntry {
            entry_type: ChatEntryType::User,
            content: initial_message.clone(),
            timestamp: chrono::Utc::now(),
            tool_calls: None,
            tool_call: None,
            tool_result: None,
            is_streaming: None,
        });

        match agent.process_user_message(&initial_message).await {
            Ok(entries) => {
                chat_state.chat_history.extend(entries);
            }
            Err(e) => {
                chat_state.chat_history.push(ChatEntry {
                    entry_type: ChatEntryType::Assistant,
                    content: format!("Error: {}", e),
                    timestamp: chrono::Utc::now(),
                    tool_calls: None,
                    tool_call: None,
                    tool_result: None,
                    is_streaming: None,
                });
            }
        }
    }

    // Run the main UI loop
    let result = run_ui_loop(&mut terminal, &mut agent, &mut chat_state).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_ui_loop(
    terminal: &mut RatatuiTerminal<CrosstermBackend<std::io::Stdout>>,
    agent: &mut GrokAgent,
    state: &mut ChatState,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw UI
        terminal.draw(|f| {
            let size = f.area();

            // Create vertical layout: header, chat area, input
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(10),   // Chat history
                    Constraint::Length(3), // Input
                ])
                .split(size);

            // Header
            let header_block = Block::default();
            f.render_widget(header_block, chunks[0]);

            // Chat history
            let chat_items: Vec<ListItem> = state.chat_history.iter()
                .map(|entry| {
                    let content = match &entry.entry_type {
                        ChatEntryType::User => format!("👤 You: {}", entry.content),
                        ChatEntryType::Assistant => format!("🤖 Grok: {}", entry.content),
                        ChatEntryType::ToolResult => format!("🔧 Tool Result: {}", entry.content),
                        ChatEntryType::ToolCall => format!("🔧 Tool Call: {}", entry.content),
                    };

                    ListItem::new(content)
                        .style(match &entry.entry_type {
                            ChatEntryType::User => Style::default().fg(Color::Green),
                            ChatEntryType::Assistant => Style::default().fg(Color::Cyan),
                            ChatEntryType::ToolResult => Style::default().fg(Color::Yellow),
                            ChatEntryType::ToolCall => Style::default().fg(Color::Magenta),
                        })
                })
                .collect();

            let chat_list = List::new(chat_items)
                .block(Block::default().borders(Borders::BOTTOM));
            f.render_widget(chat_list, chunks[1]);

            // Input area
            let input_text = format!("> {}_", state.input);
            let input_paragraph = Paragraph::new(input_text)
                .block(Block::default());
            f.render_widget(input_paragraph, chunks[2]);
            
            // Show mention hints as overlay if available
            if state.show_mention_hints && !state.mention_hints.is_empty() {
                // Create a popup area for hints (above the input)
                let hints_height = (state.mention_hints.len() as u16).min(5) + 2; // +2 for border
                let popup_area = Rect {
                    x: chunks[2].x,
                    y: chunks[2].y.saturating_sub(hints_height),
                    width: chunks[2].width,
                    height: hints_height,
                };
                
                // Render mention hints popup
                let hint_items: Vec<ListItem> = state.mention_hints.iter().take(5).enumerate()
                    .map(|(idx, hint)| {
                        let style = if idx == state.selected_mention_hint {
                            Style::default().fg(Color::Black).bg(Color::Magenta)
                        } else {
                            Style::default().fg(Color::Magenta)
                        };
                        ListItem::new(hint.clone()).style(style)
                    })
                    .collect();
                
                let hints_list = List::new(hint_items)
                    .block(Block::default().borders(Borders::ALL).title("Mentions"));
                f.render_widget(hints_list, popup_area);
            }
            
            // Show command hints as overlay if available
            if state.show_command_hints && !state.command_hints.is_empty() {
                // Create a popup area for hints (above the input)
                let hints_height = (state.command_hints.len() as u16).min(5) + 2; // +2 for border
                let popup_area = Rect {
                    x: chunks[2].x,
                    y: chunks[2].y.saturating_sub(hints_height),
                    width: chunks[2].width,
                    height: hints_height,
                };
                
                // Render hints popup
                let hint_items: Vec<ListItem> = state.command_hints.iter().take(5).enumerate()
                    .map(|(idx, hint)| {
                        let style = if idx == state.selected_hint {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::Yellow)
                        };
                        ListItem::new(hint.clone()).style(style)
                    })
                    .collect();
                
                let hints_list = List::new(hint_items)
                    .block(Block::default().borders(Borders::ALL).title("Commands"));
                f.render_widget(hints_list, popup_area);
            }
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Only process Press events, ignore Release and Repeat
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' => {
                            return Ok(());
                        },
                        KeyCode::Char(c) => {
                            state.input.push(c);
                            
                            // Check for @ mentions
                            if let Some(at_pos) = state.input.rfind('@') {
                                let after_at = &state.input[at_pos..];
                                if !after_at.contains(' ') {
                                    // We're in a mention
                                    state.show_mention_hints = true;
                                    let mention_lower = after_at.to_lowercase();
                                    state.mention_hints = AVAILABLE_MENTIONS
                                        .iter()
                                        .filter(|mention| {
                                            let mention_name = mention.split(" - ").next().unwrap_or("");
                                            mention_name.to_lowercase().starts_with(&mention_lower)
                                        })
                                        .map(|s| s.to_string())
                                        .collect();
                                    state.selected_mention_hint = 0;
                                    state.show_command_hints = false;
                                } else {
                                    state.show_mention_hints = false;
                                    state.mention_hints.clear();
                                }
                            } else {
                                state.show_mention_hints = false;
                                state.mention_hints.clear();
                            }
                            
                            // Update command hints when user types '/'
                            if state.input.starts_with('/') && !state.show_mention_hints {
                                state.show_command_hints = true;
                                let input_lower = state.input.to_lowercase();
                                state.command_hints = AVAILABLE_COMMANDS
                                    .iter()
                                    .filter(|cmd| {
                                        // Extract command name (before the dash)
                                        let cmd_name = cmd.split(" - ").next().unwrap_or("");
                                        cmd_name.to_lowercase().starts_with(&input_lower)
                                    })
                                    .map(|s| s.to_string())
                                    .collect();
                                state.selected_hint = 0;
                            } else if !state.show_mention_hints {
                                state.show_command_hints = false;
                                state.command_hints.clear();
                            }
                        },
                        KeyCode::Backspace => {
                            state.input.pop();
                            
                            // Update command hints after backspace
                            if state.input.starts_with('/') {
                                state.show_command_hints = true;
                                let input_lower = state.input.to_lowercase();
                                state.command_hints = AVAILABLE_COMMANDS
                                    .iter()
                                    .filter(|cmd| {
                                        // Extract command name (before the dash)
                                        let cmd_name = cmd.split(" - ").next().unwrap_or("");
                                        cmd_name.to_lowercase().starts_with(&input_lower)
                                    })
                                    .map(|s| s.to_string())
                                    .collect();
                                state.selected_hint = 0;
                            } else {
                                state.show_command_hints = false;
                                state.command_hints.clear();
                            }
                        },
                        KeyCode::Up => {
                            // Navigate up in mention hints
                            if state.show_mention_hints && !state.mention_hints.is_empty() {
                                if state.selected_mention_hint > 0 {
                                    state.selected_mention_hint -= 1;
                                }
                            }
                            // Navigate up in command hints
                            else if state.show_command_hints && !state.command_hints.is_empty() {
                                if state.selected_hint > 0 {
                                    state.selected_hint -= 1;
                                }
                            }
                        },
                        KeyCode::Down => {
                            // Navigate down in mention hints
                            if state.show_mention_hints && !state.mention_hints.is_empty() {
                                if state.selected_mention_hint < state.mention_hints.len() - 1 {
                                    state.selected_mention_hint += 1;
                                }
                            }
                            // Navigate down in command hints
                            else if state.show_command_hints && !state.command_hints.is_empty() {
                                if state.selected_hint < state.command_hints.len() - 1 {
                                    state.selected_hint += 1;
                                }
                            }
                        },
                        KeyCode::Tab => {
                            // Auto-complete selected mention
                            if state.show_mention_hints && !state.mention_hints.is_empty() {
                                let selected = &state.mention_hints[state.selected_mention_hint];
                                let mention = selected.split_whitespace().next().unwrap_or("");
                                // Replace from the last @ to the end
                                if let Some(at_pos) = state.input.rfind('@') {
                                    state.input.truncate(at_pos);
                                    state.input.push_str(mention);
                                    state.input.push(' ');
                                }
                                state.show_mention_hints = false;
                                state.mention_hints.clear();
                            }
                            // Auto-complete selected command
                            else if state.show_command_hints && !state.command_hints.is_empty() {
                                let selected = &state.command_hints[state.selected_hint];
                                let cmd = selected.split_whitespace().next().unwrap_or("");
                                state.input = cmd.to_string();
                                state.show_command_hints = false;
                                state.command_hints.clear();
                            }
                        },
                        KeyCode::Enter => {
                            if !state.input.trim().is_empty() {
                                let user_input = state.input.clone();
                                state.show_command_hints = false;
                                state.command_hints.clear();
                                state.show_mention_hints = false;
                                state.mention_hints.clear();
                                
                                // Check if input is a command
                                if user_input.starts_with('/') {
                                    let cmd_response = match user_input.trim() {
                                        "/help" => {
                                            "Available commands:\n\
                                            /help - Show this help message\n\
                                            /clear - Clear chat history\n\
                                            /status - Show application status\n\
                                            /model - Show current model\n\
                                            /exit - Exit the application".to_string()
                                        },
                                        "/clear" => {
                                            state.chat_history.clear();
                                            "Chat history cleared.".to_string()
                                        },
                                        "/status" => {
                                            "Status: Running\n\
                                            Model: Grok\n\
                                            Ready for input.".to_string()
                                        },
                                        "/model" => {
                                            "Current model: grok-2\n\
                                            Available models: grok-2, grok-vision".to_string()
                                        },
                                        "/exit" => {
                                            return Ok(());
                                        },
                                        _ => format!("Unknown command: {}. Type /help for available commands.", user_input),
                                    };
                                    
                                    state.chat_history.push(ChatEntry {
                                        entry_type: ChatEntryType::Assistant,
                                        content: cmd_response,
                                        timestamp: chrono::Utc::now(),
                                        tool_calls: None,
                                        tool_call: None,
                                        tool_result: None,
                                        is_streaming: None,
                                    });
                                } else {
                                    // Add user message to chat
                                    state.chat_history.push(ChatEntry {
                                        entry_type: ChatEntryType::User,
                                        content: user_input.clone(),
                                        timestamp: chrono::Utc::now(),
                                        tool_calls: None,
                                        tool_call: None,
                                        tool_result: None,
                                        is_streaming: None,
                                    });

                                    // Process with agent
                                    let agent_response = agent.process_user_message(&user_input).await;

                                    match agent_response {
                                        Ok(entries) => {
                                            // Filter out empty entries and combine consecutive assistant messages
                                            let mut filtered_entries = Vec::new();
                                            let mut last_assistant_content = String::new();
                                            
                                            for entry in entries {
                                                // Skip empty content entries
                                                if entry.content.trim().is_empty() {
                                                    continue;
                                                }
                                                
                                                // Combine consecutive assistant messages
                                                if entry.entry_type == ChatEntryType::Assistant {
                                                    if !last_assistant_content.is_empty() {
                                                        last_assistant_content.push('\n');
                                                    }
                                                    last_assistant_content.push_str(&entry.content);
                                                } else {
                                                    // Flush accumulated assistant content
                                                    if !last_assistant_content.is_empty() {
                                                        filtered_entries.push(ChatEntry {
                                                            entry_type: ChatEntryType::Assistant,
                                                            content: last_assistant_content.clone(),
                                                            timestamp: chrono::Utc::now(),
                                                            tool_calls: None,
                                                            tool_call: None,
                                                            tool_result: None,
                                                            is_streaming: None,
                                                        });
                                                        last_assistant_content.clear();
                                                    }
                                                    filtered_entries.push(entry);
                                                }
                                            }
                                            
                                            // Flush any remaining assistant content
                                            if !last_assistant_content.is_empty() {
                                                filtered_entries.push(ChatEntry {
                                                    entry_type: ChatEntryType::Assistant,
                                                    content: last_assistant_content,
                                                    timestamp: chrono::Utc::now(),
                                                    tool_calls: None,
                                                    tool_call: None,
                                                    tool_result: None,
                                                    is_streaming: None,
                                                });
                                            }
                                            
                                            state.chat_history.extend(filtered_entries);
                                        }
                                        Err(e) => {
                                            state.chat_history.push(ChatEntry {
                                                entry_type: ChatEntryType::Assistant,
                                                content: format!("Error: {}", e),
                                                timestamp: chrono::Utc::now(),
                                                tool_calls: None,
                                                tool_call: None,
                                                tool_result: None,
                                                is_streaming: None,
                                            });
                                        }
                                    }
                                }
                                
                                state.input.clear();
                            }
                        },
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }
}