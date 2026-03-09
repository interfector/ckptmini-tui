use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessState {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

impl From<&str> for ProcessState {
    fn from(s: &str) -> Self {
        match s {
            "R" => ProcessState::Running,
            "S" => ProcessState::Sleeping,
            "T" => ProcessState::Stopped,
            "Z" => ProcessState::Zombie,
            _ => ProcessState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_total: u64,
    pub threads: u32,
    pub state: ProcessState,
}

pub fn list_processes() -> std::io::Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy().to_string();

        let pid: u32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let cmdline_path = path.join("cmdline");
        let name = if cmdline_path.exists() {
            if let Ok(data) = std::fs::read(&cmdline_path) {
                let cmdline = String::from_utf8_lossy(&data).replace('\0', " ");
                if cmdline.is_empty() {
                    continue;
                }
                cmdline
            } else {
                continue;
            }
        } else {
            continue;
        };

        let status_path = path.join("status");
        let (memory_total, threads, state) = if status_path.exists() {
            let mut memory_total = 0u64;
            let mut threads = 1u32;
            let mut state = ProcessState::Unknown;

            if let Ok(content) = std::fs::read_to_string(&status_path) {
                for line in content.lines() {
                    if let Some(val) = line.strip_prefix("VmRSS:") {
                        memory_total = val.trim().trim_end_matches(" kB").parse().unwrap_or(0);
                    } else if let Some(val) = line.strip_prefix("Threads:") {
                        threads = val.trim().parse().unwrap_or(1);
                    } else if let Some(val) = line.strip_prefix("State:") {
                        let state_str = val.trim().split_whitespace().next().unwrap_or("U");
                        state = ProcessState::from(state_str);
                    }
                }
            }
            (memory_total * 1024, threads, state)
        } else {
            (0, 1, ProcessState::Unknown)
        };

        processes.push(ProcessInfo {
            pid,
            name,
            memory_total,
            threads,
            state,
        });
    }

    processes.sort_by(|a, b| b.memory_total.cmp(&a.memory_total));

    Ok(processes)
}
