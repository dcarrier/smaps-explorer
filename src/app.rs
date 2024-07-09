use procfs::process::MMapPath;
use procfs::process::MMapPath::*;
use procfs::process::MemoryMap;
use procfs::ProcError;
use ratatui::style::palette::tailwind;
use std::error;

use ratatui::{
    prelude::*,
    widgets::{Block, StatefulWidget, Widget},
};

use tui_tree_widget::{Tree, TreeItem, TreeState};

const SELECTED_STYLE_FG: Color = tailwind::BLUE.c300;

type MemoryMapMatrix = Vec<Vec<MemoryMap>>;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;
/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub debug: bool,
    pub memory_maps: MemoryMapMatrix,
    pub segments: SegmentList,
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
        let segments = SegmentList::new(memory_maps.clone());

        Ok(Self {
            running: true,
            debug,
            memory_maps,
            segments,
        })
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn go_top(&mut self) {
        self.segments.state.select_first();
    }

    pub fn go_bottom(&mut self) {
        self.segments.state.select_last();
    }

    pub fn get_selected_segment(&self) -> Option<MemoryMap> {
        let indices = self.segments.get_selected_identifier();
        match indices {
            Some(v) => {
                let (outer_key, inner_key) = v;
                return Some(self.segments.segments[outer_key][inner_key].clone());
            }
            None => None,
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

#[derive(Debug)]
pub struct SegmentList {
    // indices into the MemoryMapMatrix
    pub state: TreeState<(usize, usize)>,
    pub last_selected: Option<(usize, usize)>,
    pub segments: MemoryMapMatrix,
}

impl SegmentList {
    fn new(segments: MemoryMapMatrix) -> Self {
        let mut state = TreeState::default();
        state.select(vec![(0, 0)]);
        Self {
            state,
            segments,
            last_selected: None,
        }
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

    pub fn get_selected_identifier(&self) -> Option<(usize, usize)> {
        let indices = self.state.selected().last();
        match indices {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let mut branches = Vec::new();
        for (i, branch) in self.segments.iter().enumerate() {
            let mut children = Vec::with_capacity(branch.len() - 1);
            for (j, item) in branch.iter().enumerate() {
                let child_name = mmpath_to_string(&item.pathname);
                children.push(TreeItem::new_leaf((i, j), child_name));
            }
            let parent_name = format!(
                "{:#x} {}",
                branch[0].address.0,
                mmpath_to_string(&branch[0].pathname)
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

        StatefulWidget::render(tree, area, buf, &mut self.state)
    }
}

impl Widget for &mut SegmentList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_list(area, buf)
    }
}
