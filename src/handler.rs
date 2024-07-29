use crate::app::{App, AppResult, AppSelectedPane};
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
        KeyCode::Char('j') | KeyCode::Down => match app.selected_pane {
            AppSelectedPane::Path => app.path_list_widget.next(),
            AppSelectedPane::Segment => app.segment_list_widget.next(),
        },

        KeyCode::Char('k') | KeyCode::Up => match app.selected_pane {
            AppSelectedPane::Path => app.path_list_widget.previous(),
            AppSelectedPane::Segment => app.segment_list_widget.previous(),
        },
        KeyCode::Char('l') | KeyCode::Right => app.path_list_widget.open(),
        KeyCode::Char('h') | KeyCode::Left => app.path_list_widget.close(),
        KeyCode::Char('g') => match app.selected_pane {
            AppSelectedPane::Path => app.path_list_widget.go_top(),
            AppSelectedPane::Segment => app.segment_list_widget.go_top(),
        },
        KeyCode::Char('G') => match app.selected_pane {
            AppSelectedPane::Path => app.path_list_widget.go_bottom(),
            AppSelectedPane::Segment => app.segment_list_widget.go_bottom(),
        },
        KeyCode::Tab | KeyCode::Enter => app.switch_pane(),

        _ => {}
    }
    Ok(())
}
