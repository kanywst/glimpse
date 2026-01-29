use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use std::{fmt, io, time::Duration};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: Box<dyn Iterator<Item = Event> + Send>,
}

impl<B: Backend> fmt::Debug for Tui<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tui")
            .field("terminal", &"Terminal<...>")
            .finish_non_exhaustive()
    }
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self {
            terminal,
            events: Box::new(std::iter::empty()),
        }
    }

    /// Enters the terminal interface mode.
    ///
    /// # Errors
    /// Returns an error if raw mode execution fails.
    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(())
    }

    /// Exits the terminal interface mode.
    ///
    /// # Errors
    /// Returns an error if raw mode disable fails.
    pub fn exit(&mut self) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        self.terminal
            .show_cursor()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    /// Draws the UI.
    ///
    /// # Errors
    /// Returns an error if the terminal draw operation fails.
    pub fn draw(&mut self, app: &crate::app::App) -> Result<()> {
        self.terminal
            .draw(|frame| crate::ui::render(app, frame))
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(())
    }

    pub fn next_event(&mut self) -> Option<Event> {
        // Simple polling for the demo
        if event::poll(Duration::from_millis(250)).unwrap_or(false) {
            return event::read().ok();
        }
        None
    }
}
