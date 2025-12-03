use ratatui::{
    prelude::{CrosstermBackend, Terminal, Rect},
    Terminal as RatatuiTerminal,
    widgets::{Block, Borders, Paragraph, List, ListItem},
    layout::{Layout, Direction, Constraint},
    style::{Style, Color},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::GrokAgent;
use crate::types::{ChatEntry, ChatEntryType};

pub struct ChatState {
    chat_history: Vec<ChatEntry>,
    input: String,
    scroll: u16,
    show_command_hints: bool,
    command_hints: Vec<String>,
    selected_hint: usize,
}

const AVAILABLE_COMMANDS: &[&str] = &[
    "/help - Show help information",
    "/clear - Clear chat history",
    "/models - Switch Grok Model",
    "/commit-and-push - AI commit & push to remote",
    "/exit - Exit the application",
];

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
            let size = f.size();

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
            let header_block = Block::default()
                .borders(Borders::BOTTOM)
                .title("🤖 Grok CLI - Conversational AI Assistant");
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
                .block(Block::default().borders(Borders::TOP | Borders::BOTTOM));
            f.render_widget(chat_list, chunks[1]);

            // Input area
            let input_text = format!("> {}_", state.input);
            let input_paragraph = Paragraph::new(input_text)
                .block(Block::default().borders(Borders::TOP).title("Input"));
            f.render_widget(input_paragraph, chunks[2]);
            
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
                            
                            // Update command hints when user types '/'
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
                            // Navigate up in command hints
                            if state.show_command_hints && !state.command_hints.is_empty() {
                                if state.selected_hint > 0 {
                                    state.selected_hint -= 1;
                                }
                            }
                        },
                        KeyCode::Down => {
                            // Navigate down in command hints
                            if state.show_command_hints && !state.command_hints.is_empty() {
                                if state.selected_hint < state.command_hints.len() - 1 {
                                    state.selected_hint += 1;
                                }
                            }
                        },
                        KeyCode::Tab => {
                            // Auto-complete selected command
                            if state.show_command_hints && !state.command_hints.is_empty() {
                                let selected = &state.command_hints[state.selected_hint];
                                // Extract just the command part (e.g., "/help" from "/help - Show this help message")
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
                                            state.chat_history.extend(entries);
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