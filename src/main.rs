use clap::Parser;
use clap_stdin::MaybeStdin;
use log::*;
use mematlas_rs::app::App;
use mematlas_rs::event::Event;
use mematlas_rs::event::EventHandler;
use mematlas_rs::handler::handle_key_events;
use mematlas_rs::tui::Tui;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io;
use tui_logger::*;

#[derive(Parser, Debug)]
struct Args {
    pid: MaybeStdin<i32>,
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

// https://github.com/ratatui-org/templates/blob/main/simple/src/main.rs
fn main() -> Result<(), Box<dyn Error>> {
    init_logger(LevelFilter::Debug).unwrap();
    set_default_level(LevelFilter::Debug);
    debug!(target:"App", "Logging initialized");

    let args = Args::parse();
    let mut app = App::new(*args.pid, args.debug)?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
