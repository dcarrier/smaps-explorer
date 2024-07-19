use crate::app::{self, App};
use itertools::Itertools;
use log::LevelFilter;
use ratatui::style::palette::tailwind;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, BorderType, Padding, Paragraph, Widget, Wrap},
    Frame,
};
use std::rc::Rc;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};
use tui_tree_widget::{Tree, TreeItem};

const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;

#[derive(Clone, Copy)]
pub struct MemoryMapWidget<'a> {
    pub app: &'a App,
    child_idx: Option<usize>,
}

impl<'a> MemoryMapWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self {
            app,
            // child_idx represents the next child we should render when we have selected
            // a root element. This allows us to render the proper metadata on the multi
            // memorymap overview screen.
            child_idx: None,
        }
    }

    fn render_memory_widget(mut self, layout: &Rc<[Rect]>, frame: &mut Frame) {
        let memory_layout_constraints: Vec<Constraint> = match self.app.selected_identifiers() {
            Some(indices) => {
                // Size 1 means we are at a root element
                if indices.len() == 1 {
                    let outer_key = indices[0].0;
                    let group_len = self.app.memory_maps[outer_key].len();
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

        let indices = match self.app.selected_identifiers() {
            Some(v) => v,
            None => {
                vec![(0, 0)]
            }
        };

        if indices.len() == 1 {
            let outer_key = indices[0].0;
            let mut inner_key = indices[0].1;
            let memory_maps_len = self.app.memory_maps[outer_key].len();
            for _ in 0..memory_maps_len {
                self.child_idx = Some(inner_key);
                frame.render_widget(self, memory_layout[inner_key]);
                inner_key += 1;
            }
        } else {
            frame.render_widget(self, memory_layout[0]);
        }
    }
}

impl<'a> Widget for MemoryMapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let indices = match self.app.selected_identifiers() {
            Some(v) => match v.last() {
                Some(v) => v.clone(),
                None => (0, 0),
            },
            None => (0, 0),
        };

        let outer_key = indices.0;
        let inner_key = self.child_idx.unwrap_or(indices.1);
        let v = self.app.memory_maps[outer_key][inner_key].clone();
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

    fn render_info_widget(self, layout: &Rc<[Rect]>, frame: &mut Frame) {
        frame.render_widget(self, layout[0]);
    }
}

impl<'a> Widget for InfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget = match self.app.selected_segments() {
            Some(s) => match s.last() {
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
            },
            // TODO: not in love with the duplicated None case
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

#[derive(Debug)]
pub struct ListWidget<'a> {
    app: &'a mut App,
}

impl<'a> ListWidget<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    fn render_list_widget(&mut self, layout: &Rc<[Rect]>, frame: &mut Frame) {
        frame.render_widget(self, layout[1]);
    }
}

impl<'a> Widget for &mut ListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut branches = Vec::new();
        for (i, branch) in self.app.memory_maps.iter().enumerate() {
            let mut children = Vec::with_capacity(branch.len() - 1);
            for (j, item) in branch.iter().enumerate() {
                let child_name = app::mmpath_to_string(&item.pathname);
                children.push(TreeItem::new_leaf((i, j), child_name));
            }
            let parent_name = format!(
                "{:#x} {}",
                branch[0].address.0,
                app::mmpath_to_string(&branch[0].pathname)
            );
            let tree_item = TreeItem::new((i, 0), parent_name, children.clone()).unwrap();
            branches.push(tree_item.clone());
        }

        let inner_block = Block::bordered()
            .title("Segments")
            .title_alignment(Alignment::Center);

        let tree = Tree::new(&branches)
            .unwrap()
            .block(inner_block)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(SELECTED_STYLE_FG),
            );

        StatefulWidget::render(tree, area, buf, &mut self.app.state)
    }
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(frame.size());

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // Immutable Borrows
    let info_widget = InfoWidget::new(app);
    // TODO need to re-add log functionality
    let log_widget = LogWidget::new(app);
    info_widget.render_info_widget(&sidebar_layout, frame);

    // Mutable Borrows
    let mut memory_map_widget = MemoryMapWidget::new(app);
    memory_map_widget.render_memory_widget(&layout, frame);
    let mut list_widget = ListWidget::new(app);
    list_widget.render_list_widget(&sidebar_layout, frame);
}
