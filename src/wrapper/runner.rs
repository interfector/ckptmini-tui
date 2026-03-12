use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct CkptminiRunner {
    binary_path: PathBuf,
}

impl CkptminiRunner {
    pub fn new(binary_path: PathBuf) -> Self {
        Self { binary_path }
    }

    pub fn dump(&self, pid: u32) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("dump")
            .arg(pid.to_string())
            .output()
            .context("Failed to run ckptmini dump")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn show(&self, pid: u32) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("show")
            .arg(pid.to_string())
            .output()
            .context("Failed to run ckptmini show")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn save(&self, pid: u32, dir: &PathBuf) -> Result<()> {
        let output = Command::new(&self.binary_path)
            .arg("save")
            .arg(pid.to_string())
            .arg(dir)
            .output()
            .context("Failed to run ckptmini save")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini save failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    pub fn restore(&self, pid: u32, dir: &PathBuf) -> Result<()> {
        let output = Command::new(&self.binary_path)
            .arg("restore")
            .arg(pid.to_string())
            .arg(dir)
            .output()
            .context("Failed to run ckptmini restore")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini restore failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    pub fn read_memory(&self, pid: u32, addr: u64, len: usize) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("read")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr))
            .arg(len.to_string())
            .output()
            .context("Failed to run ckptmini read")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn resolve(&self, pid: u32, symbol: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("resolve")
            .arg(pid.to_string())
            .arg(symbol)
            .output()
            .context("Failed to run ckptmini resolve")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini resolve failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn parasite(&self, pid: u32, dir: &PathBuf) -> Result<()> {
        let output = Command::new(&self.binary_path)
            .arg("parasite")
            .arg(pid.to_string())
            .arg(dir)
            .output()
            .context("Failed to run ckptmini parasite")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini parasite failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    pub fn inject_shellcode(&self, pid: u32, shellcode: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("inject_shellcode")
            .arg(pid.to_string())
            .arg(shellcode)
            .output()
            .context("Failed to run ckptmini inject_shellcode")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini inject_shellcode failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn write(&self, pid: u32, addr: u64, hex: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("write")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr))
            .arg(hex)
            .output()
            .context("Failed to run ckptmini write")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini write failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn write_str(&self, pid: u32, addr: u64, s: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("write_str")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr))
            .arg(s)
            .output()
            .context("Failed to run ckptmini write_str")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini write_str failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn write_dump(&self, dir: &PathBuf, addr: u64, hex: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("write_dump")
            .arg(dir)
            .arg(format!("0x{:x}", addr))
            .arg(hex)
            .output()
            .context("Failed to run ckptmini write_dump")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini write_dump failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn write_dump_str(&self, dir: &PathBuf, addr: u64, s: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("write_dump_str")
            .arg(dir)
            .arg(format!("0x{:x}", addr))
            .arg(s)
            .output()
            .context("Failed to run ckptmini write_dump_str")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini write_dump_str failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn upload(&self, pid: u32, hex: &str, perms: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("upload").arg(pid.to_string()).arg(hex);
        if let Some(p) = perms {
            cmd.arg(p);
        }
        let output = cmd.output().context("Failed to run ckptmini upload")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini upload failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn upload_str(&self, pid: u32, s: &str, perms: Option<&str>) -> Result<String> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("upload").arg(pid.to_string()).arg("--str").arg(s);
        if let Some(p) = perms {
            cmd.arg(p);
        }
        let output = cmd.output().context("Failed to run ckptmini upload")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini upload failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn breakpoint(&self, pid: u32, addr: u64) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("breakpoint")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr))
            .output()
            .context("Failed to run ckptmini breakpoint")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini breakpoint failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn call(&self, pid: u32, addr: u64, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("call")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr));
        for arg in args {
            cmd.arg(arg);
        }
        let output = cmd.output().context("Failed to run ckptmini call")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini call failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn load_so(&self, pid: u32, path: &str) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("load_so")
            .arg(pid.to_string())
            .arg(path)
            .output()
            .context("Failed to run ckptmini load_so")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini load_so failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn watch(&self, pid: u32, addr: u64) -> Result<String> {
        let output = Command::new(&self.binary_path)
            .arg("watch")
            .arg(pid.to_string())
            .arg(format!("0x{:x}", addr))
            .output()
            .context("Failed to run ckptmini watch")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini watch failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
