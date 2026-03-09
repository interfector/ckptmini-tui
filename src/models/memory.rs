use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
}

impl Permissions {
    pub fn as_string(&self) -> String {
        format!(
            "{}{}{}",
            if self.read { 'r' } else { '-' },
            if self.write { 'w' } else { '-' },
            if self.exec { 'x' } else { '-' }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub perms: Permissions,
    pub offset: u64,
    pub device: String,
    pub inode: u64,
    pub path: Option<String>,
}

impl MemoryRegion {
    pub fn size(&self) -> u64 {
        self.end - self.start
    }

    pub fn human_size(&self) -> String {
        let size = self.size();
        Self::format_size(size)
    }

    pub fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}
