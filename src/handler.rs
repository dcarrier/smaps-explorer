use crate::app::{App, AppResult};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        //KeyCode::Char('h') | KeyCode::Left => app.segments.unselect(),
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('l') | KeyCode::Right => app.open(),
        KeyCode::Char('h') | KeyCode::Left => app.close(),
        KeyCode::Enter => app.toggle_selected(),
        KeyCode::Char('g') => app.go_top(),
        KeyCode::Char('G') => app.go_bottom(),

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
