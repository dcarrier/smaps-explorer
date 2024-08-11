use crate::app::{self, App, MemoryMapMatrix};
use humansize::{format_size, DECIMAL};
use itertools::Itertools;
use log::LevelFilter;
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config, Nucleo, Utf32String};
use procfs::process::MemoryMap;
use ratatui::style::palette::tailwind;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Widget,
    },
    Frame,
};
use std::rc::Rc;
use std::sync::Arc;
use std::thread::available_parallelism;
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;

#[derive(Clone, Debug)]
pub struct SegmentTableWidget {
    memory_maps: Rc<MemoryMapMatrix>,
    selected_identifier: Option<usize>,
    state: TableState,
    active_pane: bool,
}

impl SegmentTableWidget {
    pub fn new(memory_map_matrix: Rc<MemoryMapMatrix>) -> Self {
        Self {
            memory_maps: memory_map_matrix,
            selected_identifier: None,
            state: TableState::default().with_selected(0),
            active_pane: false,
        }
    }

    fn render_memory_widget(
        &mut self,
        layout: Rect,
        frame: &mut Frame,
        selected_identifier: Option<usize>,
    ) {
        self.selected_identifier(selected_identifier);
        let memory_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(Constraint::from_percentages([100]))
            .split(layout);

        frame.render_widget(self, memory_layout[0]);
    }

    pub fn active_pane(&mut self, active: bool) {
        self.active_pane = active;
    }

    pub fn next(&mut self) {
        if let Some(v) = self.state.selected() {
            // TODO: I am not sure this is correct: (default 0)
            let outer = self.selected_identifier.unwrap_or(0);
            let idx = (v + 1) % self.memory_maps[outer].len();
            self.state.select(Some(idx));
        };
    }

    pub fn previous(&mut self) {
        if let Some(v) = self.state.selected() {
            // TODO: I am not sure this is correct: (default 0)
            let outer = self.selected_identifier.unwrap_or(0);
            let idx = if v == 0 {
                self.memory_maps[outer].len() - 1
            } else {
                v - 1
            };
            self.state.select(Some(idx));
        };
    }

    pub fn go_top(&mut self) {
        self.reset_select();
    }

    pub fn go_bottom(&mut self) {
        // TODO: I am not sure this is correct: (default 0)
        let outer = self.selected_identifier.unwrap_or(0);
        let idx = self.memory_maps[outer].len() - 1;
        self.state.select(Some(idx));
    }

    pub fn reset_select(&mut self) {
        self.state.select(Some(0));
    }

    fn selected_identifier(&mut self, id: Option<usize>) {
        self.selected_identifier = id;
    }

    fn selected_segment(&self) -> Option<MemoryMap> {
        // TODO: I am not sure this is correct: (default 0)
        let outer = self.selected_identifier.unwrap_or(0);
        let inner = self.state.selected().unwrap_or(0);
        Some(self.memory_maps[outer][inner].clone())
    }
}

impl Widget for &mut SegmentTableWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: I am not sure this is correct: (default 0)
        let outer_key = self.selected_identifier.unwrap_or(0);
        let mut rows = Vec::new();
        for mm in self.memory_maps[outer_key].iter() {
            let size = *mm.extension.map.get("Size").unwrap_or(&0);
            let rss = *mm.extension.map.get("Rss").unwrap_or(&0);
            let start_addr = format!("{:#x}", mm.address.0);
            let end_addr = format!("{:#x}", mm.address.1);
            rows.push(Row::new(vec![
                start_addr,
                end_addr,
                format_size(size, DECIMAL),
                format_size(rss, DECIMAL),
            ]));
        }

        let widths = vec![
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Length(25),
        ];

        let table = Table::new(rows, widths)
            .block(
                Block::bordered()
                    .title_top("Segment")
                    .title_style(selected_pane_color(&self.active_pane))
                    .title_alignment(Alignment::Center)
                    .border_style(selected_pane_color(&self.active_pane)),
            )
            .highlight_style(Style::new().light_yellow())
            .header(Row::new(vec!["Start", "End", "Size", "RSS"]).style(Style::new().bold()));

        StatefulWidget::render(table, area, buf, &mut self.state)
    }
}

#[derive(Clone, Debug)]
pub struct InfoWidget {
    selected_segment: Option<MemoryMap>,
}

impl Default for InfoWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl InfoWidget {
    pub fn new() -> Self {
        Self {
            selected_segment: None,
        }
    }

    fn render_info_widget(
        &mut self,
        layout: Rect,
        frame: &mut Frame,
        selected_segment: Option<MemoryMap>,
    ) {
        self.selected_segments(selected_segment);
        frame.render_widget(self, layout);
    }

    fn selected_segments(&mut self, selected_segment: Option<MemoryMap>) {
        self.selected_segment = selected_segment
    }
}

impl Widget for &mut InfoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: I don't love this clone, needs to be a better way to do this.
        match self.selected_segment.clone() {
            Some(v) => {
                let mut rows: Vec<Row> = vec![
                    Row::new(["start_addr".to_string(), format!("{:#x}", v.address.0)]),
                    Row::new(["end_addr".to_string(), format!("{:#x}", v.address.1)]),
                    Row::new(["permissions".to_string(), v.perms.as_str().to_string()]),
                    Row::new(["offset".to_string(), format!("{}", v.offset)]),
                    Row::new(["dev".to_string(), format!("{}:{}", v.dev.0, v.dev.1)]),
                    Row::new(["inode".to_string(), format!("{}", v.inode)]),
                    Row::new([
                        "vm_flags".to_string(),
                        v.extension
                            .vm_flags
                            .iter_names()
                            .map(|v| v.0.to_string())
                            .collect::<Vec<String>>()
                            .join(" ")
                            .to_string(),
                    ]),
                ];
                for k in v.extension.map.keys().sorted() {
                    let v = v.extension.map[k];
                    rows.push(Row::new([
                        k.to_lowercase().to_string(),
                        format_size(v, DECIMAL),
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
                let widget = Paragraph::new("no info".to_string())
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

impl Default for LogWidget {
    fn default() -> Self {
        Self::new()
    }
}

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

pub struct PathListWidget {
    memory_maps: Rc<MemoryMapMatrix>,
    pub state: ListState,
    pub searching: bool,
    pub searcher: Nucleo<(u64, String)>,
    filter: String,
    active_pane: bool,
}

impl PathListWidget {
    pub fn new(memory_map_matrix: Rc<MemoryMapMatrix>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let num_threads = Some(available_parallelism().unwrap().get());
        let mut searcher = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), num_threads, 2);
        for mm in memory_map_matrix.iter() {
            let values = (mm[0].address.0, app::mmpath_to_string(&mm[0].pathname));
            searcher.injector().push(values, |values, c| {
                c[0] = Utf32String::Ascii(values.0.to_string().as_str().into());
                c[1] = Utf32String::Ascii(values.1.to_string().as_str().into());
            });
        }
        // Immediatly tick() so we paint the ui at startup.
        searcher.tick(10);
        Self {
            memory_maps: memory_map_matrix,
            state,
            searcher,
            searching: false,
            filter: String::new(),
            active_pane: true,
        }
    }

    fn render_list_widget(&mut self, layout: Rect, frame: &mut Frame, filter: String) {
        self.filter(filter);
        frame.render_widget(self, layout);
    }

    fn filter(&mut self, input: String) {
        self.filter = input;
    }

    pub fn active_pane(&mut self, active: bool) {
        self.active_pane = active;
    }

    pub fn searching_toggle(&mut self) {
        self.searching = !self.searching;
    }

    pub fn go_top(&mut self) {
        self.state.select_first();
    }

    pub fn go_bottom(&mut self) {
        self.state.select_last();
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn selected_identifiers(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_segments(&self) -> Option<Vec<MemoryMap>> {
        self.selected_identifiers()
            .map(|v| self.memory_maps[v].clone())
    }
}

impl Widget for &mut PathListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut paths = Vec::new();
        self.searcher.pattern.reparse(
            1,
            &self.filter,
            CaseMatching::Ignore,
            Normalization::Never,
            false,
        );
        for item in self
            .searcher
            .snapshot()
            .matched_items(0..self.searcher.snapshot().matched_item_count())
        {
            let path_item = ListItem::new(format!("{:#x}  {}", item.data.0, item.data.1));
            paths.push(path_item.clone());
        }

        let inner_block = Block::bordered()
            .border_style(selected_pane_color(&self.active_pane))
            .title("Path")
            .title_alignment(Alignment::Center);

        let tree = List::new(paths).block(inner_block).highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(SELECTED_STYLE_FG),
        );

        StatefulWidget::render(tree, area, buf, &mut self.state)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LegendWidget {}

impl LegendWidget {
    fn render_legend_widget(self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout);
    }
}

impl Widget for LegendWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = Text::from(vec![Line::from(
            "tab/enter - switch pane\t\t j - down\t\t k - up\t\t g - top\t\t G - bottom\t\t / - filter path",
        )]);
        let widget = Paragraph::new(text)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .border_type(BorderType::Double),
            )
            .centered();
        Widget::render(widget, area, buf);
    }
}

#[derive(Clone, Debug, Default)]
pub struct PathFilterWidget {
    pub filter: String,
}

impl PathFilterWidget {
    fn render_path_filter_widget(&mut self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout);
    }
}

impl Widget for &mut PathFilterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(88), Constraint::Percentage(12)])
            .split(area);
        let term_block = Block::default()
            .title("Search for Path")
            .borders(Borders::ALL);
        let term_text = Paragraph::new(self.filter.clone()).block(term_block);
        // Important to Clear before painting a new widget on top of existing layout.
        Clear.render(area, buf);
        Widget::render(term_text, popup_chunks[1], buf);
    }
}

fn selected_pane_color(active_pane: &bool) -> Style {
    match active_pane {
        true => Style::default().fg(Color::Green),
        false => Style::default().fg(Color::White),
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    // TODO: Horrible variable naming
    let initial_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
        .split(frame.size());

    let base_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(75),
            Constraint::Percentage(25),
            Constraint::Length(2),
        ])
        .split(initial_layout[0]);

    let main_layout = if app.debug {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Fill(1),
            ])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(45), Constraint::Fill(1)])
    };
    let main_layout = main_layout.split(base_layout[0]);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(base_layout[1]);

    let legend_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(initial_layout[1]);

    let selected_segment = app.segment_list_widget.selected_segment();
    let indices = app.path_list_widget.selected_identifiers();
    if app.debug {
        app.info_widget
            .render_info_widget(sidebar_layout[0], frame, selected_segment);
        app.segment_list_widget
            .render_memory_widget(main_layout[0], frame, indices);
        app.log_widget.render_log_widget(main_layout[1], frame);
        app.path_list_widget.render_list_widget(
            main_layout[1],
            frame,
            app.path_filter_widget.filter.clone(),
        );
        app.legend_widget
            .render_legend_widget(legend_layout[0], frame);
    } else {
        app.info_widget
            .render_info_widget(sidebar_layout[0], frame, selected_segment);
        app.segment_list_widget
            .render_memory_widget(main_layout[0], frame, indices);
        app.path_list_widget.render_list_widget(
            main_layout[1],
            frame,
            app.path_filter_widget.filter.clone(),
        );
        app.legend_widget
            .render_legend_widget(legend_layout[0], frame);
        if app.path_list_widget.searching {
            app.path_filter_widget
                .render_path_filter_widget(main_layout[0], frame);
        }
    }
}
