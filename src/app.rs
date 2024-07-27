use crate::ui::{InfoWidget, ListWidget, LogWidget, MemoryMapWidget};
use procfs::process::MMapPath;
use procfs::process::MMapPath::*;
use procfs::process::MemoryMap;
use procfs::ProcError;
use std::error;
use std::rc::Rc;

pub type MemoryMapMatrix = Vec<Vec<MemoryMap>>;
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    running: bool,
    pub debug: bool,
    pub selected_pane: AppSelectedPane,
    pub memory_maps: Rc<MemoryMapMatrix>,
    pub memory_map_widget: MemoryMapWidget,
    pub list_widget: ListWidget,
    pub info_widget: InfoWidget,
    pub log_widget: LogWidget,
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
            memory_map_widget: MemoryMapWidget::new(Rc::clone(&memory_maps)),
            list_widget: ListWidget::new(Rc::clone(&memory_maps)),
            info_widget: InfoWidget::new(),
            log_widget: LogWidget::new(),
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn switch_pane(&mut self) {
        match self.selected_pane {
            AppSelectedPane::Segment => self.selected_pane = AppSelectedPane::Path,
            AppSelectedPane::Path => self.selected_pane = AppSelectedPane::Segment,
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
        // TODO fix this unwrap
        Path(x) => x.to_str().unwrap().to_string(),
        Heap => "heap".to_string(),
        Stack => "stack".to_string(),
        TStack(x) => format!("thread stack: {0}", x),
        Vdso => "vdso".to_string(),
        Vvar => "vvar".to_string(),
        Vsyscall => "vsyscall".to_string(),
        Rollup => "rollup".to_string(),
        Anonymous => "anonymous".to_string(),
        Vsys(x) => format!("vsys: {0}", x),
        Other(x) => format!("{0}", x),
    }
}
