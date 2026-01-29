use anyhow::Result;
use clap::Parser;
use glim::app::App;
use glim::tui::Tui;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the git repository to analyze
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize the terminal interface
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let terminal = ratatui::Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.enter()?;

    // Create application state with the specified path
    let mut app = App::new(args.path);

    // Main event loop
    loop {
        tui.draw(&app)?;

        // Handle events
        if let Some(event) = tui.next_event()
            && !glim::handlers::handle_event(&mut app, &event)
        {
            break;
        }
    }

    // Exit gracefully
    tui.exit()?;
    Ok(())
}
