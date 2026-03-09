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

    pub fn inject_shellcode(&self, pid: u32) -> Result<()> {
        let output = Command::new(&self.binary_path)
            .arg("inject_shellcode")
            .arg(pid.to_string())
            .output()
            .context("Failed to run ckptmini inject_shellcode")?;

        if !output.status.success() {
            anyhow::bail!(
                "ckptmini inject_shellcode failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}
