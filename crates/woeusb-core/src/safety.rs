use crate::error::{Result, WoeError};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    pub device: PathBuf,
    pub mountpoint: PathBuf,
    pub fs_type: String,
}

#[derive(Debug, Clone)]
pub struct SafetyReport {
    pub target_device: PathBuf,
    pub mounted_partitions: Vec<(PathBuf, PathBuf)>, // (partition_dev, mountpoint)
    pub warnings: Vec<String>,
}

pub fn parse_mounts(content: &str) -> Vec<MountEntry> {
    let mut entries = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            entries.push(MountEntry {
                device: PathBuf::from(parts[0]),
                mountpoint: PathBuf::from(parts[1]),
                fs_type: parts[2].to_string(),
            });
        }
    }
    entries
}

pub fn get_system_mounts() -> Result<Vec<MountEntry>> {
    let content = fs::read_to_string("/proc/mounts")
        .or_else(|_| fs::read_to_string("/etc/mtab"))
        .unwrap_or_default();
    Ok(parse_mounts(&content))
}

pub fn parse_swaps(content: &str) -> Vec<PathBuf> {
    let mut swaps = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if i == 0 {
            // header line: Filename Type Size Used Priority
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            swaps.push(PathBuf::from(parts[0]));
        }
    }
    swaps
}

pub fn get_system_swaps() -> Result<Vec<PathBuf>> {
    let content = fs::read_to_string("/proc/swaps").unwrap_or_default();
    Ok(parse_swaps(&content))
}

pub fn verify_device_safety(
    target: &Path,
    mounts: &[MountEntry],
    swaps: &[PathBuf],
) -> Result<SafetyReport> {
    let target_str = target.to_string_lossy();
    let target_file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| WoeError::InvalidSource(format!("Invalid target path: {:?}", target)))?;

    let mut mounted_partitions = Vec::new();
    let mut warnings = Vec::new();

    // Check against swap devices
    for swap in swaps {
        let swap_str = swap.to_string_lossy();
        if swap_str == target_str || swap_str.starts_with(&*target_str) {
            return Err(WoeError::SafetyViolation(format!(
                "Target device {:?} or one of its partitions ({}) is in active use as SWAP memory. Refusing to operate.",
                target, swap_str
            )));
        }
    }

    // Check against mounted partitions
    for entry in mounts {
        let dev_str = entry.device.to_string_lossy();
        let dev_name = entry.device.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check if the mount entry belongs to the target device or its partition
        let is_target_or_partition = dev_str == target_str
            || (dev_name.starts_with(target_file_name) && dev_str.starts_with("/dev/"));

        if is_target_or_partition {
            let mp_str = entry.mountpoint.to_string_lossy();
            if mp_str == "/"
                || mp_str == "/boot"
                || mp_str == "/home"
                || mp_str == "/etc"
                || mp_str == "/usr"
                || mp_str == "/var"
            {
                return Err(WoeError::SafetyViolation(format!(
                    "FATAL SAFETY VIOLATION: Target {:?} has partition {:?} mounted at critical system root location '{}'! Wiping this drive would destroy your operating system.",
                    target, entry.device, mp_str
                )));
            }

            mounted_partitions.push((entry.device.clone(), entry.mountpoint.clone()));
        }
    }

    if !mounted_partitions.is_empty() {
        warnings.push(format!(
            "Target device has {} mounted partition(s) which will need to be unmounted.",
            mounted_partitions.len()
        ));
    }

    Ok(SafetyReport {
        target_device: target.to_path_buf(),
        mounted_partitions,
        warnings,
    })
}
