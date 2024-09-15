use crate::ui::{
    HelpWidget, InfoWidget, LegendWidget, LogWidget, PathFilterWidget, PathListWidget,
    SegmentTableWidget,
};
use procfs::process::MMapPath;
use procfs::process::MMapPath::*;
use procfs::process::MemoryMap;
use procfs::ProcError;
use std::error;
use std::rc::Rc;

pub type MemoryMapMatrix = Vec<Vec<MemoryMap>>;
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub struct App {
    running: bool,
    pub debug: bool,
    pub selected_pane: AppSelectedPane,
    pub memory_maps: Rc<MemoryMapMatrix>,
    pub segment_list_widget: SegmentTableWidget,
    pub path_list_widget: PathListWidget,
    pub path_filter_widget: PathFilterWidget,
    pub info_widget: InfoWidget,
    pub log_widget: LogWidget,
    pub legend_widget: LegendWidget,
    pub help_widget: HelpWidget,
}

#[derive(Debug)]
pub enum AppSelectedPane {
    Segment,
    Path,
}

impl App {
    pub fn new(pid: i32, debug: bool) -> AppResult<Self> {
        let process = procfs::process::Process::new(pid)?;
        let memory_maps = match smaps_rollup(&process)? {
            Some(v) => {
                let mut rollup_prefix = vec![vec![v.clone()]];
                let memory_maps = smaps(&process)?;
                rollup_prefix.extend(memory_maps);
                Rc::new(rollup_prefix)
            }
            None => Rc::new(smaps(&process)?),
        };

        Ok(Self {
            running: true,
            debug,
            selected_pane: AppSelectedPane::Path,
            memory_maps: Rc::clone(&memory_maps),
            segment_list_widget: SegmentTableWidget::new(Rc::clone(&memory_maps)),
            path_list_widget: PathListWidget::new(Rc::clone(&memory_maps)),
            path_filter_widget: PathFilterWidget::default(),
            info_widget: InfoWidget::default(),
            log_widget: LogWidget::default(),
            legend_widget: LegendWidget::default(),
            help_widget: HelpWidget::default(),
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // TODO: Need to tick on the searcher each time.
        // Re-evaluate this timeout value.
        self.path_list_widget.searcher.tick(10);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn switch_pane(&mut self) {
        match self.selected_pane {
            AppSelectedPane::Segment => {
                // Import to reset the Segment selection so you don't go
                // out of bounds on a smaller segment as you navigate the
                // Path pane.
                self.segment_list_widget.active_pane(false);
                self.segment_list_widget.reset_select();
                self.selected_pane = AppSelectedPane::Path;
                self.path_list_widget.active_pane(true);
            }
            AppSelectedPane::Path => {
                self.path_list_widget.active_pane(false);
                self.selected_pane = AppSelectedPane::Segment;
                self.segment_list_widget.active_pane(true);
            }
        }
    }
}

fn smaps_rollup(process: &procfs::process::Process) -> Result<Option<MemoryMap>, ProcError> {
    let mut rollup = process.smaps_rollup()?;
    Ok(rollup.memory_map_rollup.0.pop())
}

fn smaps(process: &procfs::process::Process) -> Result<MemoryMapMatrix, ProcError> {
    let maps = process.smaps()?.0;

    // We want to merge consecutive memorymaps with the same name.
    // This allows us to create summaries and nested lists of maps.
    let mut merged: MemoryMapMatrix = Vec::new();
    let mut idx = 0;
    while idx < maps.len() {
        let mut map_group: Vec<MemoryMap> = Vec::new();
        let parent = &maps[idx];
        map_group.push(parent.clone());
        let parent_name = mmpath_to_string(&parent.pathname);
        idx += 1;
        while idx < maps.len() && parent_name == mmpath_to_string(&maps[idx].pathname) {
            let child = &maps[idx];
            map_group.push(child.clone());
            idx += 1;
        }
        merged.push(map_group);
    }
    Ok(merged)
}

pub fn mmpath_to_string(name: &MMapPath) -> String {
    match name {
        Path(x) => x.to_string_lossy().into_owned(),
        Heap => "heap".into(),
        Stack => "stack".into(),
        TStack(x) => format!("thread stack: {0}", x),
        Vdso => "vdso".into(),
        Vvar => "vvar".into(),
        Vsyscall => "vsyscall".into(),
        Rollup => "rollup".into(),
        Anonymous => "anonymous".into(),
        Vsys(x) => format!("vsys: {0}", x),
        Other(x) => x.to_string(),
    }
}
