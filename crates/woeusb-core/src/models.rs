use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TargetFilesystem {
    Fat,
    Ntfs,
}

impl fmt::Display for TargetFilesystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetFilesystem::Fat => write!(f, "FAT32"),
            TargetFilesystem::Ntfs => write!(f, "NTFS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMode {
    Device,
    Partition,
}

impl fmt::Display for InstallMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallMode::Device => write!(f, "device"),
            InstallMode::Partition => write!(f, "partition"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub filesystem: Option<String>,
    pub mountpoint: Option<PathBuf>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drive {
    pub name: String,
    pub path: PathBuf,
    pub model: String,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub partitions: Vec<Partition>,
    pub mountpoints: Vec<PathBuf>,
}

impl Drive {
    pub fn is_system_drive(&self) -> bool {
        self.mountpoints.iter().any(|p| {
            p == &PathBuf::from("/")
                || p == &PathBuf::from("/home")
                || p == &PathBuf::from("/boot")
                || p == &PathBuf::from("/etc")
                || p == &PathBuf::from("/usr")
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ProgressEvent {
    StepStarted {
        step: u32,
        total_steps: u32,
        message: String,
    },
    ProgressUpdated {
        bytes_copied: u64,
        total_bytes: u64,
        percentage: f64,
        speed_bytes_per_sec: u64,
        current_file: String,
    },
    StepFinished {
        step: u32,
        message: String,
    },
    LogMessage {
        level: String,
        message: String,
    },
    Completed {
        message: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashConfig {
    pub source: PathBuf,
    pub target: PathBuf,
    pub install_mode: InstallMode,
    pub target_filesystem: TargetFilesystem,
    pub filesystem_label: String,
    pub workaround_bios_boot_flag: bool,
    pub skip_grub: bool,
    pub dry_run: bool,
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            source: PathBuf::new(),
            target: PathBuf::new(),
            install_mode: InstallMode::Device,
            target_filesystem: TargetFilesystem::Fat,
            filesystem_label: "Windows USB".to_string(),
            workaround_bios_boot_flag: false,
            skip_grub: false,
            dry_run: false,
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}
