use crate::app::{App, AppAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_chat_event(app: &mut App, key: KeyEvent) -> AppAction {
        // 优先处理文件命令确认对话
        if app.file_command_handler.has_pending_confirmation() {
            match key.code {
                KeyCode::Up => {
                    app.file_command_handler.move_confirmation_up();
                    return AppAction::None;
                }
                KeyCode::Down => {
                    app.file_command_handler.move_confirmation_down();
                    return AppAction::None;
                }
                KeyCode::Enter => {
                    // 执行确认选择
                    let choice = app.file_command_handler.get_confirmation_choice();
                    let cmd = crate::commands::FileCommand::ConfirmModify;
                    // 这里会在后续的命令处理中执行
                    return AppAction::SubmitChat;
                }
                KeyCode::Esc => {
                    // 取消确认
                    let cmd = crate::commands::FileCommand::CancelModify;
                    let _ = app.file_command_handler.execute(cmd);
                    return AppAction::None;
                }
                _ => return AppAction::None,
            }
        }

        if app.command_hints.visible {
            match key.code {
                KeyCode::Up => {
                    app.command_hints.select_previous();
                    return AppAction::None;
                }
                KeyCode::Down => {
                    app.command_hints.select_next();
                    return AppAction::None;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    if let Some(completed) = app.command_hints.get_selected_item() {
                        app.input_text = completed;
                    }
                    app.command_hints.visible = false;
                    if key.code == KeyCode::Enter {
                        return AppAction::SubmitChat;
                    }
                    return AppAction::None;
                }
                KeyCode::Esc => {
                    app.command_hints.visible = false;
                    return AppAction::None;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => AppAction::Quit,
            KeyCode::Enter => AppAction::SubmitChat,
            KeyCode::Backspace => {
                app.input_text.pop();
                app.command_hints.update_input(&app.input_text);
                AppAction::None
            }
            KeyCode::Char(c) => {
                app.input_text.push(c);
                app.command_hints.update_input(&app.input_text);
                AppAction::None
            }
            _ => AppAction::None,
        }
    }
}