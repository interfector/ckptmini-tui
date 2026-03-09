use crate::models::{MemoryRegion, Permissions};

pub fn parse_memory_regions(output: &str) -> Vec<MemoryRegion> {
    let mut regions = Vec::new();

    for line in output.lines() {
        let line = line.trim();

        if line.is_empty()
            || line.starts_with("PID ")
            || line.starts_with("START")
            || line.starts_with("─")
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let start_hex = parts[0];
        let end_hex = parts[1];
        let perms_str = parts[2];

        let start = u64::from_str_radix(start_hex, 16).unwrap_or(0);
        let end = u64::from_str_radix(end_hex, 16).unwrap_or(0);

        let perms = Permissions {
            read: perms_str.chars().nth(0).unwrap_or('-') == 'r',
            write: perms_str.chars().nth(1).unwrap_or('-') == 'w',
            exec: perms_str.chars().nth(2).unwrap_or('-') == 'x',
        };

        let path = if parts.len() > 4 {
            let idx =
                if parts[3].ends_with("K") || parts[3].ends_with("M") || parts[3].ends_with("G") {
                    4
                } else {
                    3
                };
            if idx < parts.len() {
                Some(parts[idx..].join(" "))
            } else {
                None
            }
        } else {
            None
        };

        regions.push(MemoryRegion {
            start,
            end,
            perms,
            offset: 0,
            device: String::new(),
            inode: 0,
            path,
        });
    }

    regions
}

pub fn list_checkpoints(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut checkpoints = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.join("mem").exists() && path.join("regs.bin").exists() {
                    checkpoints.push(path);
                }
            }
        }
    }

    checkpoints.sort_by(|a, b| {
        let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    checkpoints
}
