use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Navigate,
    Edit,
    Connect,  // For creating connections with search
    OpenFile,  // For opening files
    SaveFile,  // For entering filename to save
    ConfirmDelete,  // For confirming place deletion
}

#[derive(Debug)]
pub enum Action {
    None,
    Quit,
    NavigateUp,
    NavigateDown,
    NavigateRight,  // Tab - go into affordances
    NavigateLeft,   // Shift+Tab - go to parent place
    Select,
    Back,
    NewPlace,
    NewAffordance,
    ToggleCollapsed,
    Filter,
    Save,
    SaveAs,
    Open,
    EnterEditMode,
    EnterConnectMode,
    RemoveConnection,
    Delete,
    Edit(String),
}

pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn read_action(&self, mode: Mode) -> Result<Action> {
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(Action::None);
        }

        let event = event::read()?;

        if let event::Event::Key(key) = event {
            return Ok(self.handle_key_event(key, mode));
        }

        Ok(Action::None)
    }

    fn handle_key_event(&self, key: KeyEvent, mode: Mode) -> Action {
        match mode {
            Mode::Navigate => self.handle_navigate_key(key, mode),
            Mode::Edit => self.handle_edit_key(key),
            Mode::Connect => self.handle_connect_key(key),
            Mode::OpenFile => self.handle_open_file_key(key),
            Mode::SaveFile => self.handle_save_file_key(key),
            Mode::ConfirmDelete => self.handle_confirm_delete_key(key),
        }
    }

    fn handle_navigate_key(&self, key: KeyEvent, mode: Mode) -> Action {
        match key.code {
            KeyCode::Up => Action::NavigateUp,
            KeyCode::Down => Action::NavigateDown,
            KeyCode::Tab => Action::NavigateRight,
            KeyCode::BackTab => Action::NavigateLeft,
            KeyCode::Enter => Action::Select,
            KeyCode::Char('e') => {
                if mode == Mode::Navigate {
                    Action::EnterEditMode
                } else {
                    Action::Edit('e'.to_string())
                }
            },
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::Delete // Ctrl+D to delete (works on all keyboards)
            }
            KeyCode::Delete => Action::Delete, // Also support Delete key if available
            KeyCode::Backspace => {
                if mode == Mode::Edit {
                    Action::Edit(String::from("backspace"))
                } else {
                    Action::Back
                }
            },
            KeyCode::Esc => {
                if mode == Mode::Edit {
                    Action::Back // Cancel edit
                } else {
                    Action::Back
                }
            },

            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::EnterConnectMode
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::RemoveConnection
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::NewPlace
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::NewAffordance
            }
            KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::ToggleCollapsed
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::Filter
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT) => {
                Action::SaveAs
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::Save
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::Open
            }
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Action::Quit
            }

            // Any other character starts place search
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL)
                             && !key.modifiers.contains(KeyModifiers::ALT) => {
                Action::Edit(c.to_string())
            }

            _ => Action::None,
        }
    }

    fn handle_edit_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Select, // Save changes and exit edit mode
            KeyCode::Esc => Action::Back, // Cancel edit
            KeyCode::Backspace => Action::Edit(String::from("backspace")),
            KeyCode::Delete => Action::Edit(String::from("delete")),
            KeyCode::Left => Action::Edit(String::from("left")),
            KeyCode::Right => Action::Edit(String::from("right")),
            KeyCode::Home => Action::Edit(String::from("home")),
            KeyCode::End => Action::Edit(String::from("end")),

            KeyCode::Char(c) => Action::Edit(c.to_string()),

            _ => Action::None,
        }
    }

    fn handle_connect_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Select, // Create connection with selected place
            KeyCode::Esc => Action::Back, // Cancel connection mode
            KeyCode::Backspace => Action::Edit(String::from("backspace")),
            KeyCode::Delete => Action::Edit(String::from("delete")),
            KeyCode::Up => Action::NavigateUp, // Navigate search results
            KeyCode::Down => Action::NavigateDown, // Navigate search results
            KeyCode::Left => Action::Edit(String::from("left")),
            KeyCode::Right => Action::Edit(String::from("right")),
            KeyCode::Home => Action::Edit(String::from("home")),
            KeyCode::End => Action::Edit(String::from("end")),

            KeyCode::Char(c) => Action::Edit(c.to_string()),

            _ => Action::None,
        }
    }

    fn handle_open_file_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Select, // Open selected file
            KeyCode::Esc => Action::Back, // Cancel file opening
            KeyCode::Up => Action::NavigateUp, // Navigate file list
            KeyCode::Down => Action::NavigateDown, // Navigate file list
            KeyCode::Left => Action::Edit(String::from("left")),
            KeyCode::Right => Action::Edit(String::from("right")),
            KeyCode::Home => Action::Edit(String::from("home")),
            KeyCode::End => Action::Edit(String::from("end")),

            _ => Action::None,
        }
    }

    fn handle_save_file_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => Action::Select, // Save with entered filename
            KeyCode::Esc => Action::Back, // Cancel save
            KeyCode::Backspace => Action::Edit(String::from("backspace")),
            KeyCode::Delete => Action::Edit(String::from("delete")),
            KeyCode::Left => Action::Edit(String::from("left")),
            KeyCode::Right => Action::Edit(String::from("right")),
            KeyCode::Home => Action::Edit(String::from("home")),
            KeyCode::End => Action::Edit(String::from("end")),

            KeyCode::Char(c) => Action::Edit(c.to_string()),

            _ => Action::None,
        }
    }

    fn handle_confirm_delete_key(&self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Action::Select, // Confirm deletion
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::Back, // Cancel deletion
            _ => Action::None,
        }
    }
}