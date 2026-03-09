use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInfo {
    pub path: PathBuf,
    pub pid: u32,
    pub created: u64,
    pub regions: u32,
    pub size_bytes: u64,
    pub command: String,
}

impl CheckpointInfo {
    pub fn from_dir(path: &PathBuf) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let mem_dir = path.join("mem");
        if !mem_dir.exists() {
            return None;
        }

        let regions = std::fs::read_dir(&mem_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "bin"))
                    .count() as u32
            })
            .unwrap_or(0);

        let size_bytes = std::fs::read_dir(&mem_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0);

        let created = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.created().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let pid = std::fs::read_to_string(path.join("meta.json"))
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("pid").and_then(|p| p.as_u64()))
            .map(|p| p as u32)
            .unwrap_or(0);

        let command = std::fs::read_to_string(path.join("meta.json"))
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("command").cloned())
            .and_then(|c| c.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            });

        Some(CheckpointInfo {
            path: path.clone(),
            pid,
            created,
            regions,
            size_bytes,
            command,
        })
    }

    pub fn age_string(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let elapsed = now.saturating_sub(self.created);

        if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}h ago", elapsed / 3600)
        } else {
            format!("{}d ago", elapsed / 86400)
        }
    }

    pub fn human_size(&self) -> String {
        crate::models::memory::MemoryRegion::format_size(self.size_bytes)
    }
}
