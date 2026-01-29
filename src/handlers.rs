use crate::app::{App, InputMode, ZoomLevel};
use crossterm::event::{Event, KeyCode, KeyEventKind};

pub fn handle_event(app: &mut App, event: &Event) -> bool {
    if let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        // Handle Editing Mode (Search)
        if app.input_mode == InputMode::Editing {
            match key.code {
                KeyCode::Enter => {
                    app.exit_search();
                }
                KeyCode::Esc => {
                    app.cancel_search();
                }
                KeyCode::Backspace => {
                    app.search_query.pop();
                    app.update_search();
                }
                KeyCode::Char(c) => {
                    app.search_query.push(c);
                    app.update_search();
                }
                _ => {}
            }
            return true;
        }

        // Handle Normal Mode
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.previous();
            }
            KeyCode::Enter => {
                app.zoom_in();
            }
            KeyCode::Backspace | KeyCode::Left => {
                app.zoom_out();
            }
            KeyCode::Char(' ') => {
                app.toggle_stage();
            }
            KeyCode::Char('+' | '=') => {
                app.increase_context();
            }
            KeyCode::Char('-' | '_') => {
                app.decrease_context();
            }
            // Search Trigger
            KeyCode::Char('/') => {
                if app.zoom_level == ZoomLevel::Structure {
                    app.enter_search();
                }
            }
            _ => {}
        }
    }
    true
}
