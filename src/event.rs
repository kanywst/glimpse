// Event handling utilities (Placeholder for future expansion)
use crossterm::event::KeyEvent;

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
}
