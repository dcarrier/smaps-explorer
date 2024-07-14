use procfs::process::MMapPath;
use procfs::process::MMapPath::*;
use procfs::process::MemoryMap;
use procfs::ProcError;
use std::error;
use tui_tree_widget::TreeState;

type MemoryMapMatrix = Vec<Vec<MemoryMap>>;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;
/// Application.
#[derive(Debug)]
pub struct App {
    running: bool,
    debug: bool,
    pub memory_maps: MemoryMapMatrix,
    pub state: TreeState<(usize, usize)>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(pid: i32, debug: bool) -> AppResult<Self> {
        let process = procfs::process::Process::new(pid)?;
        let memory_maps = match smaps_rollup(&process)? {
            Some(v) => {
                let mut rollup_prefix = vec![vec![v.clone()]];
                let memory_maps = smaps(&process)?;
                rollup_prefix.extend(memory_maps);
                rollup_prefix
            }
            None => smaps(&process)?,
        };
        let mut state = TreeState::default();
        state.select(vec![(0, 0)]);

        Ok(Self {
            running: true,
            debug,
            memory_maps,
            state,
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

    pub fn selected_segments(&self) -> Option<Vec<MemoryMap>> {
        let indices = self.selected_identifiers();
        match indices {
            Some(v) => Some(Vec::from_iter(
                v.iter()
                    .map(|item| self.memory_maps[item.0][item.1].clone())
                    .collect::<Vec<MemoryMap>>(),
            )),
            None => None,
        }
    }

    pub fn selected_identifiers(&self) -> Option<Vec<(usize, usize)>> {
        let indices = self.state.selected();
        if indices.len() == 0 {
            return None;
        }

        return Some(indices.to_vec());
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
