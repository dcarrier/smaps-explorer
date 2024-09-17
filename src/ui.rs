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
        TableState, Widget, Wrap,
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
            let outer = self.selected_identifier.unwrap_or(0);
            let idx = (v + 1) % self.memory_maps[outer].len();
            self.state.select(Some(idx));
        };
    }

    pub fn previous(&mut self) {
        if let Some(v) = self.state.selected() {
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
        let outer = self.selected_identifier.unwrap_or(0);
        let inner = self.state.selected().unwrap_or(0);
        Some(self.memory_maps[outer][inner].clone())
    }
}

impl Widget for &mut SegmentTableWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
    pub toggle: bool,
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
            toggle: false,
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

    pub fn toggle(&mut self) {
        self.toggle = !self.toggle;
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

        let list = List::new(paths).block(inner_block).highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(SELECTED_STYLE_FG),
        );

        StatefulWidget::render(list, area, buf, &mut self.state)
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
            "tab/enter - switch pane\t\t j - down\t\t k - up\t\t g - top\t\t G - bottom\t\t / - filter path\t\t h - help",
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
    fn render_path_filter_widget(&self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout);
    }
}

impl Widget for &PathFilterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)])
            .split(area);
        let term_block = Block::default()
            .title("Search for Path")
            .borders(Borders::ALL);
        let term_text = Paragraph::new(self.filter.clone()).block(term_block);
        // Important to Clear before painting a new widget on top of existing layout.
        Clear.render(area, buf);
        Widget::render(term_text, popup_chunks[0], buf);
    }
}

#[derive(Default)]
pub struct HelpWidget {
    pub toggle: bool,
}

impl HelpWidget {
    fn render_help_widget(&self, layout: Rect, frame: &mut Frame) {
        frame.render_widget(self, layout)
    }

    pub fn toggle(&mut self) {
        self.toggle = !self.toggle;
    }
}

impl Widget for &HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)])
            .split(area);
        let term_block = Block::default()
            .title("SMAPS Explorer Help")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        let term_text = Paragraph::new(Text::from(vec![
            Line::from(vec![Span::raw("* start_addr: ").bold(), Span::raw("starting memory address in hex.")]),
            Line::from(vec![Span::raw("* end_addr: ").bold(), Span::raw("ending memory address in hex.")]),
            Line::from(vec![Span::raw("* permissions: ").bold(), Span::raw("is a set of permissions, r-read, w-write, x-execute, s=shared, p=private (copy on write)")]),
            Line::from(vec![Span::raw("* offset: ").bold(), Span::raw("the offset into the mapping")]),
            Line::from(vec![Span::raw("* dev: ").bold(), Span::raw("the device (major:minor)")]),
            Line::from(vec![Span::raw("* inode: ").bold(), Span::raw("the inode on that device. 0 indicates that no inode is associated with the memory region, as the case would be with BSS (uninitialized data)")]),
            Line::from(vec![Span::raw("* vm_flags: ").bold(), Span::raw("this member represents the kernel flags associated with the particular virtual memory area in two letter encoded manner. Press \"v\" to show flags.")]),
            Line::from(vec![Span::raw("* anonhugepages: ").bold(), Span::raw("shows the amount of memory backed by transparent hugepage.")]),
            Line::from(vec![Span::raw("* anonymous: ").bold(), Span::raw("shows the amount of memory that does not belong to any file. Even a mapping associated with a file may contain anonymous pages: when MAP_PRIVATE and a page is modified, the file page is replaced by a private anonymous copy.")]),
            Line::from(vec![Span::raw("* filepmdmapped: ").bold(), Span::raw("page cache mapped into userspace with huge pages")]),
            Line::from(vec![Span::raw("* ksm: ").bold().bold(), Span::raw("reports how many of the pages are KSM pages. Note that KSM-placed zeropages are not included, only actual KSM pages.")]),
            Line::from(vec![Span::raw("* lazyfree: ").bold(), Span::raw("shows the amount of memory which is marked by madvise(MADV_FREE). The memory isn’t freed immediately with madvise(). It’s freed in memory pressure if the memory is clean. Please note that the printed value might be lower than the real value due to optimizations used in the current implementation. If this is not desirable please file a bug report.")]),
            Line::from(vec![Span::raw("* locked: ").bold(), Span::raw("indicates whether the mapping is locked in memory or not.")]),
            Line::from(vec![Span::raw("* private_clean: ").bold(), Span::raw("the number of clean private pages in the mapping")]),
            Line::from(vec![Span::raw("* private_dirty: ").bold(), Span::raw("the number of dirty private pages in the mapping")]),
            Line::from(vec![Span::raw("* private_hugetlb: ").bold(), Span::raw("show the amounts of memory backed by hugetlbfs page which is not counted in “RSS” or “PSS” field for historical reasons. And these are not included in {Shared,Private}_{Clean,Dirty} field.")]),
            Line::from(vec![Span::raw("* pss: ").bold(), Span::raw("the process’ proportional share of this mapping. The count of pages it has in memory, where each page is divided by the number of processes sharing it.")]),
            Line::from(vec![Span::raw("* pss_anon: ").bold(), Span::raw("proportional share of anonymous.")]),
            Line::from(vec![Span::raw("* pss_dirty: ").bold(), Span::raw("proportional share of dirty.")]),
            Line::from(vec![Span::raw("* pss_file: ").bold(), Span::raw("proporotional share of file.")]),
            Line::from(vec![Span::raw("* pss_shmem: ").bold(), Span::raw("proportional share of of shmem.")]),
            Line::from(vec![Span::raw("* referenced: ").bold(), Span::raw("indicates the amount of memory currently marked as referenced or accessed")]),
            Line::from(vec![Span::raw("* rss: ").bold(), Span::raw("the amount of the mapping that is currently resident in RAM.")]),
            Line::from(vec![Span::raw("* shared_clean: ").bold(), Span::raw("the number of clean shared pages in the mapping")]),
            Line::from(vec![Span::raw("* shared_dirty: ").bold(), Span::raw("the number of dirty shared pages in the mapping")]),
            Line::from(vec![Span::raw("* shared_hugetlb: ").bold(), Span::raw("show the amounts of memory backed by hugetlbfs page which is not counted in “RSS” or “PSS” field for historical reasons. And these are not included in {Shared,Private}_{Clean,Dirty} field.")]),
            Line::from(vec![Span::raw("* shmempmdmapped: ").bold(), Span::raw("shows the amount of shared (shmem/tmpfs) memory backed by huge pages.")]),
            Line::from(vec![Span::raw("* size: ").bold(), Span::raw("the size of the mapping")]),
            Line::from(vec![Span::raw("* swap: ").bold(), Span::raw("shows how much would-be-anonymous memory is also used, but out on swap.")]),
            Line::from(vec![Span::raw("* swappss: ").bold(), Span::raw("shows proportional swap share of this mapping. Unlike “Swap”, this does not take into account swapped out page of underlying shmem objects.")]),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::raw("source: ").bold(), Span::raw("https://www.kernel.org/doc/html/latest/filesystems/proc.html")]),
        ])).wrap(Wrap{trim: true})
        .block(term_block);
        // Important to Clear before painting a new widget on top of existing layout.
        Clear.render(area, buf);
        Widget::render(term_text, popup_chunks[0], buf);
    }
}

fn selected_pane_color(active_pane: &bool) -> Style {
    match active_pane {
        true => Style::default().fg(Color::Green),
        false => Style::default().fg(Color::White),
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
        .split(frame.size());

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(75),
            Constraint::Percentage(25),
            Constraint::Length(2),
        ])
        .split(base_layout[0]);

    let legend_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(base_layout[1]);

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
    let main_layout = main_layout.split(content_layout[0]);

    let info_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(content_layout[1]);

    let selected_segment = app.segment_list_widget.selected_segment();
    let indices = app.path_list_widget.selected_identifiers();
    if app.debug {
        app.info_widget
            .render_info_widget(info_layout[0], frame, selected_segment);
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
        if app.path_list_widget.toggle {
            app.path_filter_widget
                .render_path_filter_widget(main_layout[0], frame);
        }
        if app.help_widget.toggle {
            app.help_widget.render_help_widget(content_layout[0], frame);
        }
    } else {
        app.info_widget
            .render_info_widget(info_layout[0], frame, selected_segment);
        app.segment_list_widget
            .render_memory_widget(main_layout[0], frame, indices);
        app.path_list_widget.render_list_widget(
            main_layout[1],
            frame,
            app.path_filter_widget.filter.clone(),
        );
        app.legend_widget
            .render_legend_widget(legend_layout[0], frame);
        if app.path_list_widget.toggle {
            app.path_filter_widget
                .render_path_filter_widget(main_layout[0], frame);
        }
        if app.help_widget.toggle {
            app.help_widget.render_help_widget(content_layout[0], frame);
        }
    }
}
