use std::borrow::Borrow;

use crate::app::{self, App};
use itertools::Itertools;
use log::LevelFilter;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, BorderType, Padding, Paragraph, Widget, Wrap},
    Frame,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

#[derive(Clone, Copy)]
pub struct MemoryMapWidget<'a> {
    pub app: &'a App,
    selected_identifier: (usize, usize),
}

impl<'a> MemoryMapWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            app,
            selected_identifier: (0, 0),
        }
    }
}

impl<'a> Widget for MemoryMapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Not correct to unwrap and assume last() here, but I wanna
        // make sure that everything is still working.

        // If the length of the selected_segments is '1' then we assume that
        // we are at a top level element. This means that we would like to display
        // all children elements in the widget. Otherwise we will only display the child element.

        /*
        let segments = match self.app.get_selected_segments() {
            Some(v) => v,
            // TODO: If there are no segments then we should probably display a helpful
            // no segments screen or similar. Leaving this for now.
            None => return,
            Paragraph::new(Span::styled(
                format!("no memorymap"),
                Style::default().fg(Color::White).bold(),
            ))
            .block(
                Block::bordered()
                    .border_style(Style::default())
                    .padding(Padding::new(0, 0, area.height / 2, 0)),
            )
            .alignment(Alignment::Center),
        };
        */

        // Right now I am doing the same thing for both cases. But that doesn't actually make any sense.
        // I think this whole thing needs to move somewhere else. Maybe render function.
        let outer_key = self.selected_identifier.0;
        let inner_key = self.selected_identifier.1;
        let v = self.app.segments.segments[outer_key][inner_key].clone();
        let widget = Paragraph::new(Span::styled(
            format!("{}", app::mmpath_to_string(&v.pathname),),
            Style::default().fg(Color::White).bold(),
        ))
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title_top(format!("{:#x}", v.address.0))
                .title_bottom(format!("{:#x}", v.address.1))
                .title_style(Style::default().fg(Color::LightGreen))
                .title_alignment(Alignment::Center)
                .border_style(Style::default())
                .padding(Padding::new(0, 0, area.height / 2, 0)),
        )
        .alignment(Alignment::Center);

        widget.render(area, buf);
    }
}

#[derive(Clone, Copy)]
pub struct InfoWidget<'a> {
    pub app: &'a App,
}

impl<'a> InfoWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for InfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Not correct to unwrap and assume last() here, but I wanna
        // make sure that everything is still working.
        let widget = match self.app.get_selected_segments().unwrap().last() {
            Some(v) => {
                let mut text = vec![
                    Line::from(format!("start_addr: {:#x}", v.address.0)),
                    Line::from(format!("end_addr: {:#x}", v.address.1)),
                    Line::from(format!("permissions: {}", v.perms.as_str())),
                    Line::from(format!("offset: {}", v.offset)),
                    Line::from(format!("dev: {}:{}", v.dev.0, v.dev.1)),
                    Line::from(format!("inode: {}", v.inode)),
                    Line::from(format!(
                        "vm_flags: {}",
                        v.extension
                            .vm_flags
                            .iter_names()
                            .map(|v| { v.0.to_string() })
                            .collect::<Vec<String>>()
                            .join(" ")
                    )),
                ];
                let mut extensions: Vec<Line> = Vec::new();
                for k in v.extension.map.keys().sorted() {
                    let v = v.extension.map[k];
                    extensions.push(Line::from(format!("{}: {}", &k.to_lowercase(), &v)));
                }
                text.extend(extensions);
                Paragraph::new(text.clone())
                    .alignment(Alignment::Center)
                    .block(
                        Block::bordered()
                            .border_type(BorderType::Double)
                            .title("Info")
                            .title_alignment(Alignment::Center),
                    )
            }
            None => Paragraph::new(format!("no info"))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Double)
                        .title("Info")
                        .title_alignment(Alignment::Center),
                ),
        };

        widget.render(area, buf);
    }
}

#[derive(Clone, Copy)]
pub struct LogWidget<'a> {
    pub app: &'a App,
}

impl<'a> LogWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for LogWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filter_state = TuiWidgetState::new()
            .set_default_display_level(LevelFilter::Off)
            .set_level_for_target("App", LevelFilter::Debug);
        TuiLoggerWidget::default()
            .block(Block::bordered().title("Filtered TuiLoggerWidget"))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White))
            .state(&filter_state)
            .render(area, buf);
    }
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(frame.size());

    // TODO: this logic should really go somewhere else
    // TODO: this no longer supports debug mode. Need to figure that out.
    let memory_layout_constraints: Vec<Constraint> = match app.segments.get_selected_identifiers() {
        Some(indices) => {
            // Size 1 means we are at a root element
            if indices.len() == 1 {
                let outer_key = indices[0].0;
                let group_len = app.segments.segments[outer_key].len();
                vec![Constraint::Percentage(100 / group_len as u16); group_len]
            } else {
                vec![Constraint::Percentage(100)]
            }
        }
        None => vec![Constraint::Percentage(100)],
    };

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(memory_layout_constraints.clone())
        .split(layout[0]);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // TODO: I am thinking both of these should be moved to app.rs just like app.segments
    let mut memory_widget = MemoryMapWidget::new(app);
    let info_widget = InfoWidget::new(app);
    let log_widget = LogWidget::new(app);

    // TODO: I believe that we want to loop over all possible memory_layouts
    // if we are at a parent element. This way we can render all of the children.
    // If we are at a child we only need to render one screen for the chosen item.
    let indices = match app.segments.get_selected_identifiers() {
        Some(v) => v,
        // TODO this is obv bad but trying to test logic
        None => panic!("did not expect to hit this"),
    };
    if memory_layout_constraints.len() > 1 {
        let outer_key = indices[0].0;
        let memory_maps = app.segments.segments[outer_key].clone();
        for (i, _) in memory_maps.iter().enumerate() {
            memory_widget.selected_identifier = (outer_key, i);
            frame.render_widget(memory_widget, memory_layout[i]);
        }
    } else {
        memory_widget.selected_identifier = indices[0];
        frame.render_widget(memory_widget, memory_layout[0]);
    };
    //
    //
    frame.render_widget(info_widget, sidebar_layout[0]);
    frame.render_widget(&mut app.segments, sidebar_layout[1]);
}
