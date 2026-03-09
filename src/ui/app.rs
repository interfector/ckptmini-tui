use crate::models::{CheckpointInfo, MemoryRegion, ProcessInfo};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    List,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Memory,
    Pid,
    Name,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Processes,
    Memory,
    Checkpoints,
}

impl Tab {
    pub fn get_headers() -> &'static [&'static str] {
        &["Processes", "Memory", "Checkpoints"]
    }
}

impl From<usize> for Tab {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::Processes,
            1 => Self::Memory,
            2 => Self::Checkpoints,
            _ => Self::Processes,
        }
    }
}

pub struct App {
    pub tab: Tab,
    pub focus: Focus,
    pub processes: Vec<ProcessInfo>,
    pub process_scroll: usize,
    pub memory_regions: Vec<MemoryRegion>,
    pub memory_scroll: usize,
    pub checkpoints: Vec<CheckpointInfo>,
    pub checkpoint_scroll: usize,
    pub show_help: bool,
    pub status_message: Option<String>,
    pub status_is_error: bool,
    pub ckptmini_path: String,
    pub checkpoint_dir: String,
    pub sort_by: SortBy,
    pub sort_ascending: bool,
    pub search_query: String,
    pub is_searching: bool,
    pub show_hex_view: bool,
    pub hex_data: String,
    pub hex_scroll: usize,
    pub hex_search: String,
    pub is_hex_searching: bool,
    pub dump_output: String,
    pub output_scroll: usize,
    pub process_info: String,
    pub output_log: Vec<String>,
    pub input_mode: bool,
    pub input_buffer: String,
}

impl App {
    pub fn new(ckptmini_path: String, checkpoint_dir: String) -> Self {
        Self {
            tab: Tab::Processes,
            focus: Focus::List,
            processes: Vec::new(),
            process_scroll: 0,
            memory_regions: Vec::new(),
            memory_scroll: 0,
            checkpoints: Vec::new(),
            checkpoint_scroll: 0,
            show_help: false,
            status_message: None,
            status_is_error: false,
            ckptmini_path,
            checkpoint_dir,
            sort_by: SortBy::Memory,
            sort_ascending: false,
            search_query: String::new(),
            is_searching: false,
            show_hex_view: false,
            hex_data: String::new(),
            hex_scroll: 0,
            hex_search: String::new(),
            is_hex_searching: false,
            dump_output: String::new(),
            output_scroll: 0,
            process_info: String::new(),
            output_log: Vec::new(),
            input_mode: false,
            input_buffer: String::new(),
        }
    }

    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.processes.get(self.process_scroll)
    }

    pub fn selected_memory_region(&self) -> Option<&MemoryRegion> {
        self.memory_regions.get(self.memory_scroll)
    }

    pub fn selected_checkpoint(&self) -> Option<&CheckpointInfo> {
        self.checkpoints.get(self.checkpoint_scroll)
    }

    pub fn add_output(&mut self, line: String) {
        self.output_log.push(line);
        while self.output_log.len() > 500 {
            self.output_log.remove(0);
        }
    }

    pub fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Memory => {
                if self.sort_ascending {
                    self.processes
                        .sort_by(|a, b| a.memory_total.cmp(&b.memory_total));
                } else {
                    self.processes
                        .sort_by(|a, b| b.memory_total.cmp(&a.memory_total));
                }
            }
            SortBy::Pid => {
                if self.sort_ascending {
                    self.processes.sort_by(|a, b| a.pid.cmp(&b.pid));
                } else {
                    self.processes.sort_by(|a, b| b.pid.cmp(&a.pid));
                }
            }
            SortBy::Name => {
                if self.sort_ascending {
                    self.processes
                        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                } else {
                    self.processes
                        .sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
                }
            }
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message.clone());
        self.status_is_error = false;
        self.add_output(format!("[status] {}", message));
    }

    pub fn set_error(&mut self, message: String) {
        self.status_message = Some(message.clone());
        self.status_is_error = true;
        self.add_output(format!("[error] {}", message));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_is_error = false;
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Processes => Tab::Memory,
            Tab::Memory => Tab::Checkpoints,
            Tab::Checkpoints => Tab::Processes,
        };
        self.add_output(format!("[mode] Switched to {:?}", self.tab));
    }

    pub fn prev_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Processes => Tab::Checkpoints,
            Tab::Memory => Tab::Processes,
            Tab::Checkpoints => Tab::Memory,
        };
        self.add_output(format!("[mode] Switched to {:?}", self.tab));
    }
}
