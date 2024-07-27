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
        KeyCode::Char('j') | KeyCode::Down => app.list_widget.next(),
        KeyCode::Char('k') | KeyCode::Up => app.list_widget.previous(),
        KeyCode::Char('l') | KeyCode::Right => app.list_widget.open(),
        KeyCode::Char('h') | KeyCode::Left => app.list_widget.close(),
        KeyCode::Enter => app.list_widget.toggle_selected(),
        KeyCode::Char('g') => app.list_widget.go_top(),
        KeyCode::Char('G') => app.list_widget.go_bottom(),
        KeyCode::Tab => app.switch_pane(),

        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
