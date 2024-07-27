use crate::app::{self, mmpath_to_string, App, AppSelectedPane, MemoryMapMatrix};
use itertools::Itertools;
use log::LevelFilter;
use procfs::process::MemoryMap;
use ratatui::style::palette::tailwind;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Padding, Paragraph, Row, Table, TableState, Widget, Wrap},
    Frame,
};
use std::rc::Rc;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};
use tui_tree_widget::{Tree, TreeItem, TreeState};

const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;

#[derive(Clone, Debug)]
pub struct MemoryMapWidget {
    memory_maps: Rc<MemoryMapMatrix>,
    child_idx: Option<usize>,
    selected_identifier: Option<(usize, usize)>,
    table_state: TableState,
    is_active_pane: bool,
}

impl MemoryMapWidget {
    pub fn new(memory_map_matrix: Rc<MemoryMapMatrix>) -> Self {
        Self {
            memory_maps: memory_map_matrix,
            // child_idx represents the next child we should render when we have selected
            // a root element. This allows us to render the proper metadata on the multi
            // memorymap overview screen.
            child_idx: None,
            selected_identifier: None,
            table_state: TableState::default().with_selected(0),
            is_active_pane: false,
        }
    }

    fn render_memory_widget(
        &mut self,
        layout: Rect,
        frame: &mut Frame,
        selected_identifier: Option<(usize, usize)>,
    ) {
        self.selected_identifier(selected_identifier);
        let memory_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(Constraint::from_percentages([100]))
            .split(layout);

        frame.render_widget(self, memory_layout[0]);
    }

    fn is_active_pane(&mut self, pane: &AppSelectedPane) {
        match pane {
            AppSelectedPane::Segment => self.is_active_pane = true,
            _ => self.is_active_pane = false,
        }
    }

    fn selected_identifier(&mut self, id: Option<(usize, usize)>) {
        self.selected_identifier = id;
    }
}

impl<'a> Widget for &mut MemoryMapWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let indices = match self.selected_identifier {
            Some(v) => v.clone(),
            None => (0, 0),
        };

        let outer_key = indices.0;
        let inner_key = self.child_idx.unwrap_or(indices.1);

        if inner_key == 0 {
            // TODO: Messing around with visualizing a table if at a top level.
            let mut rows = Vec::new();
            let total_size: f32 = self.memory_maps[outer_key]
                .iter()
                .map(|v| *v.extension.map.get("Size").unwrap_or(&0) as f32)
                .sum();

            for mm in self.memory_maps[outer_key].iter() {
                let path_name = mmpath_to_string(&mm.pathname);
                let perc_sz = format!(
                    "{:.1}",
                    // TODO: adding 1 because rollup does not have a total size. Fix this
                    // f32 conversions feel messy here as well.
                    (*mm.extension.map.get("Size").unwrap_or(&0) as f32) / (total_size + 1.0)
                        * 100.0
                );
                let start_addr = format!("{:#x}", mm.address.0);
                let end_addr = format!("{:#x}", mm.address.0);
                rows.push(Row::new(vec![path_name, perc_sz, start_addr, end_addr]));
            }

            let widths = vec![
                Constraint::Percentage(40),
                Constraint::Length(10),
                Constraint::Length(14),
                Constraint::Length(14),
            ];

            let table = Table::new(rows, widths)
                .block(
                    Block::bordered()
                        .title_top("Segment")
                        .title_style(selected_pane_color(&self.is_active_pane))
                        .title_alignment(Alignment::Center)
                        .border_style(selected_pane_color(&self.is_active_pane)),
                )
                .highlight_style(Style::new().light_yellow())
                .header(Row::new(vec!["Path", "Percentage", "Start", "End"]));

            StatefulWidget::render(table, area, buf, &mut self.table_state)
            // // //
        } else {
            let v = self.memory_maps[outer_key][inner_key].clone();
            let widget = Paragraph::new(vec![
                Line::from(format!("{}", v.perms.as_str())),
                // TODO: not great that I need to know the exact key here (with casing)
                Line::from(format!("{}", v.extension.map.get("Size").unwrap_or(&0))),
            ])
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title_top(format!("{:#x}", v.address.0))
                    .title_bottom(format!("{:#x}", v.address.1))
                    .title_style(selected_pane_color(&self.is_active_pane))
                    .title_alignment(Alignment::Center)
                    .border_style(selected_pane_color(&self.is_active_pane))
                    .padding(Padding::new(0, 0, area.height / 2, 0)),
            )
            .alignment(Alignment::Center);

            widget.render(area, buf);
        }
    }
}

#[derive(Clone, Debug)]
pub struct InfoWidget {
    selected_segments: Option<Vec<MemoryMap>>,
}

impl InfoWidget {
    pub fn new() -> Self {
        Self {
            selected_segments: None,
        }
    }

    fn render_info_widget(
        &mut self,
        layout: Rect,
        frame: &mut Frame,
        selected_segments: Option<Vec<MemoryMap>>,
    ) {
        self.selected_segments(selected_segments);
        frame.render_widget(self, layout);
    }

    fn selected_segments(&mut self, selected_segments: Option<Vec<MemoryMap>>) {
        self.selected_segments = selected_segments
    }
}

impl Widget for &mut InfoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: I don't love this clone, needs to be a better way to do this.
        match self.selected_segments.clone() {
            Some(s) => match s.last() {
                Some(v) => {
                    let mut rows: Vec<Row> = vec![
                        Row::new(["start_addr".to_string(), format!("{:#x}", v.address.0)]),
                        Row::new(["end_addr".to_string(), format!("{:#x}", v.address.1)]),
                        Row::new(["permissions".to_string(), format!("{}", v.perms.as_str())]),
                        Row::new(["offset".to_string(), format!("{}", v.offset)]),
                        Row::new(["dev".to_string(), format!("{}:{}", v.dev.0, v.dev.1)]),
                        Row::new(["inode".to_string(), format!("{}", v.inode)]),
                        Row::new([
                            "vm_flags".to_string(),
                            format!(
                                "{}",
                                v.extension
                                    .vm_flags
                                    .iter_names()
                                    .map(|v| { v.0.to_string() })
                                    .collect::<Vec<String>>()
                                    .join(" ")
                            ),
                        ]),
                    ];
                    for k in v.extension.map.keys().sorted() {
                        let v = v.extension.map[k];
                        rows.push(Row::new([
                            format!("{}", &k.to_lowercase()),
                            format!("{}", &v),
                        ]));
                    }
                    let widths = vec![Constraint::Percentage(50); 2];
                    let widget = Table::new(rows, widths).block(
                        Block::bordered()
                            .title("Info")
                            .title_alignment(Alignment::Center),
                    );
                    Widget::render(widget, area, buf)
                }
                None => {
                    let widget = Paragraph::new(format!("no info"))
                        .alignment(Alignment::Center)
                        .block(
                            Block::bordered()
                                .title("Info")
                                .title_alignment(Alignment::Center),
                        );
                    Widget::render(widget, area, buf)
                }
            },
            // TODO: not in love with the duplicated None case
            None => {
                let widget = Paragraph::new(format!("no info"))
                    .alignment(Alignment::Center)
                    .block(
                        Block::bordered()
                            .title("Info")
                            .title_alignment(Alignment::Center),
                    );
                Widget::render(widget, area, buf)
            }
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogWidget {}

impl LogWidget {
    pub fn new() -> Self {
        Self {}
    }

    fn render_log_widget(self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout);
    }
}

impl Widget for LogWidget {
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
pub struct ListWidget {
    memory_maps: Rc<MemoryMapMatrix>,
    pub state: TreeState<(usize, usize)>,
    is_active_pane: bool,
}

impl ListWidget {
    pub fn new(memory_map_matrix: Rc<MemoryMapMatrix>) -> Self {
        let mut state = TreeState::default();
        state.select(vec![(0, 0)]);

        Self {
            memory_maps: memory_map_matrix,
            state,
            is_active_pane: false,
        }
    }

    fn render_list_widget(&mut self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout);
    }

    fn is_active_pane(&mut self, pane: &AppSelectedPane) {
        match pane {
            AppSelectedPane::Path => self.is_active_pane = true,
            _ => self.is_active_pane = false,
        }
    }

    pub fn go_top(&mut self) {
        self.state.select_first();
    }

    pub fn go_bottom(&mut self) {
        self.state.select_last();
    }

    pub fn next(&mut self) {
        self.state.key_down();
    }

    pub fn previous(&mut self) {
        self.state.key_up();
    }

    pub fn open(&mut self) {
        self.state.key_right();
    }

    pub fn close(&mut self) {
        self.state.key_left();
    }

    pub fn toggle_selected(&mut self) {
        self.state.toggle_selected();
    }

    pub fn selected_identifiers(&self) -> Option<(usize, usize)> {
        let indices = self.state.selected();
        if indices.len() == 0 {
            return None;
        }

        return Some(indices[0]);
    }

    pub fn selected_segments(&self) -> Option<Vec<MemoryMap>> {
        let indices = self.selected_identifiers();
        match indices {
            Some(v) => Some(self.memory_maps[v.0].clone()),
            None => None,
        }
    }
}

impl Widget for &mut ListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut branches = Vec::new();
        for (i, branch) in self.memory_maps.iter().enumerate() {
            let parent_name = format!(
                "{:#x} {}",
                branch[0].address.0,
                app::mmpath_to_string(&branch[0].pathname)
            );
            // TODO: this probably doesnt need to be a TreeWidget as we are just
            // at a single level now. Simpler with a ratatui::List i'd presume.
            let tree_item = TreeItem::new((i, 0), parent_name, vec![]).unwrap();
            branches.push(tree_item.clone())
        }

        let inner_block = Block::bordered()
            .border_style(selected_pane_color(&self.is_active_pane))
            .title("Path")
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

        StatefulWidget::render(tree, area, buf, &mut self.state)
    }
}

fn selected_pane_color(active_pane: &bool) -> Style {
    match active_pane {
        true => Style::default().fg(Color::Green),
        false => Style::default().fg(Color::White),
    }
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    let base_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(frame.size());

    let main_layout = if app.debug {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(40),
                Constraint::Percentage(50),
                Constraint::Percentage(10),
            ])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
    };
    let main_layout = main_layout.split(base_layout[0]);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(base_layout[1]);

    // Set active pane for this render
    app.list_widget.is_active_pane(&app.selected_pane);
    app.memory_map_widget.is_active_pane(&app.selected_pane);

    // TODO: testing out some dependency injection from App
    let selected_segments = app.list_widget.selected_segments();
    let indices = app.list_widget.selected_identifiers();
    if app.debug {
        app.info_widget
            .render_info_widget(sidebar_layout[0], frame, selected_segments);
        app.memory_map_widget
            .render_memory_widget(main_layout[0], frame, indices);
        app.log_widget.render_log_widget(main_layout[1], frame);
        // TODO: this is broken for non debug use cases
        app.list_widget.render_list_widget(main_layout[2], frame);
    } else {
        app.info_widget
            .render_info_widget(sidebar_layout[0], frame, selected_segments);
        app.memory_map_widget
            .render_memory_widget(main_layout[0], frame, indices);
        // TODO: this is broken for non debug use cases
        app.list_widget.render_list_widget(main_layout[1], frame);
    }
}
