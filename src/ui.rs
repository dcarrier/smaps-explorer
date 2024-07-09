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
}

impl<'a> MemoryMapWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for MemoryMapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = match self.app.get_selected_segment() {
            Some(v) => Paragraph::new(Span::styled(
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
            .alignment(Alignment::Center),
            None => Paragraph::new(Span::styled(
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
        let widget = match self.app.get_selected_segment() {
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
    let memory_layout_constraints: Vec<Constraint> = match app.segments.get_selected_identifier() {
        Some(indices) => {
            let (outer_key, inner_key) = indices;
            if inner_key == 0 {
                let group_len = app.segments.segments[outer_key].len();
                let mut constraints: Vec<Constraint> = Vec::with_capacity(group_len);
                for i in 0..group_len {
                    constraints.push(Constraint::Percentage(100 / group_len as u16));
                }
                constraints
            } else {
                vec![Constraint::Percentage(100)]
            }
        }
        None => vec![Constraint::Percentage(100)],
    };

    let memory_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(memory_layout_constraints)
        .split(layout[0]);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // TODO: I am thinking both of these should be moved to app.rs just like app.segments
    let memory_widget = MemoryMapWidget::new(app);
    let info_widget = InfoWidget::new(app);
    let log_widget = LogWidget::new(app);

    if app.debug {
        frame.render_widget(log_widget, memory_layout[1]);
    }
    frame.render_widget(memory_widget, memory_layout[0]);
    frame.render_widget(info_widget, sidebar_layout[0]);
    frame.render_widget(&mut app.segments, sidebar_layout[1]);
}
