use crate::app::{App, AppResult, AppSelectedPane};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.path_list_widget.toggle {
        true => match key_event.code {
            KeyCode::Char('/') | KeyCode::Enter => app.path_list_widget.toggle(),
            KeyCode::Backspace => {
                app.path_filter_widget.filter.pop();
            }
            KeyCode::Char(value) => app.path_filter_widget.filter.push(value),
            _ => (),
        },
        false => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.quit();
            }
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
            KeyCode::Char('g') => match app.selected_pane {
                AppSelectedPane::Path => app.path_list_widget.go_top(),
                AppSelectedPane::Segment => app.segment_list_widget.go_top(),
            },
            KeyCode::Char('G') => match app.selected_pane {
                AppSelectedPane::Path => app.path_list_widget.go_bottom(),
                AppSelectedPane::Segment => app.segment_list_widget.go_bottom(),
            },
            KeyCode::Tab => app.switch_pane(),
            KeyCode::Char('/') => app.path_list_widget.toggle(),
            KeyCode::Char('h') => {
                app.help_widget.toggle();
                app.legend_widget.help_toggled();
            }
            KeyCode::Char('v') => app.help_widget.toggle_vm_flags(),
            _ => {}
        },
    }

    Ok(())
}
