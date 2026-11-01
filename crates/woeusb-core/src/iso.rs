use crate::error::{Result, WoeError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const FAT32_MAX_FILE_SIZE: u64 = 4 * 1024 * 1024 * 1024 - 1; // 4 GiB - 1 byte

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoMetadata {
    pub path: PathBuf,
    pub total_size_bytes: u64,
    pub max_file_size_bytes: u64,
    pub max_file_name: String,
    pub has_install_wim: bool,
    pub has_install_esd: bool,
    pub is_uefi_capable: bool,
    pub is_windows_7: bool,
    pub requires_ntfs: bool,
}

pub fn inspect_iso(path: &Path) -> Result<IsoMetadata> {
    if !path.exists() {
        return Err(WoeError::InvalidSource(format!(
            "Source ISO path does not exist: {:?}",
            path
        )));
    }

    // If source is a mounted directory or directory, inspect directly
    if path.is_dir() {
        return inspect_mounted_dir(path);
    }

    // Try inspecting via 7z
    if let Ok(meta) = inspect_iso_via_7z(path) {
        return Ok(meta);
    }

    // Fallback: estimate from file metadata
    let file_size = fs::metadata(path)?.len();
    Ok(IsoMetadata {
        path: path.to_path_buf(),
        total_size_bytes: file_size,
        max_file_size_bytes: file_size,
        max_file_name: "unknown".to_string(),
        has_install_wim: true,
        has_install_esd: false,
        is_uefi_capable: true,
        is_windows_7: false,
        requires_ntfs: file_size >= FAT32_MAX_FILE_SIZE,
    })
}

pub fn inspect_mounted_dir(mountpoint: &Path) -> Result<IsoMetadata> {
    let mut total_size = 0u64;
    let mut max_file_size = 0u64;
    let mut max_file_name = String::new();
    let mut has_install_wim = false;
    let mut has_install_esd = false;
    let mut is_uefi_capable = false;
    let mut is_windows_7 = false;

    for entry in walkdir(mountpoint)? {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = fs::metadata(&path) {
                let len = meta.len();
                total_size += len;
                if len > max_file_size {
                    max_file_size = len;
                    max_file_name = path
                        .strip_prefix(mountpoint)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                }

                let lower = path.to_string_lossy().to_lowercase();
                if lower.ends_with("install.wim") {
                    has_install_wim = true;
                } else if lower.ends_with("install.esd") {
                    has_install_esd = true;
                } else if lower.ends_with("bootx64.efi") || lower.ends_with("bootia32.efi") {
                    is_uefi_capable = true;
                } else if lower.ends_with("cversion.ini") {
                    if let Ok(content) = fs::read_to_string(path) {
                        if content.contains("MinServer=7") {
                            is_windows_7 = true;
                        }
                    }
                }
            }
        }
    }

    Ok(IsoMetadata {
        path: mountpoint.to_path_buf(),
        total_size_bytes: total_size,
        max_file_size_bytes: max_file_size,
        max_file_name,
        has_install_wim,
        has_install_esd,
        is_uefi_capable,
        is_windows_7,
        requires_ntfs: max_file_size >= FAT32_MAX_FILE_SIZE,
    })
}

fn inspect_iso_via_7z(path: &Path) -> Result<IsoMetadata> {
    let output = Command::new("7z")
        .args(["l", "-ba", "-slt"])
        .arg(path)
        .output()
        .map_err(|e| WoeError::Other(format!("Failed to execute 7z: {}", e)))?;

    if !output.status.success() {
        return Err(WoeError::Other(format!(
            "7z returned exit code {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_7z_slt_output(path, &stdout)
}

pub fn parse_7z_slt_output(path: &Path, content: &str) -> Result<IsoMetadata> {
    let mut total_size = 0u64;
    let mut max_file_size = 0u64;
    let mut max_file_name = String::new();
    let mut has_install_wim = false;
    let mut has_install_esd = false;
    let mut is_uefi_capable = false;
    let mut is_windows_7 = false;

    let mut current_path = String::new();
    let mut current_size = 0u64;
    let mut is_folder = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !is_folder && !current_path.is_empty() {
                total_size += current_size;
                if current_size > max_file_size {
                    max_file_size = current_size;
                    max_file_name = current_path.clone();
                }

                let lower = current_path.to_lowercase();
                if lower.ends_with("install.wim") {
                    has_install_wim = true;
                } else if lower.ends_with("install.esd") {
                    has_install_esd = true;
                } else if lower.ends_with("bootx64.efi") || lower.ends_with("bootia32.efi") {
                    is_uefi_capable = true;
                } else if lower.contains("cversion.ini") {
                    is_windows_7 = true;
                }
            }
            current_path.clear();
            current_size = 0;
            is_folder = false;
            continue;
        }

        if let Some(val) = line.strip_prefix("Path = ") {
            current_path = val.to_string();
        } else if let Some(val) = line.strip_prefix("Size = ") {
            current_size = val.parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("Folder = ") {
            is_folder = val == "+";
        }
    }

    Ok(IsoMetadata {
        path: path.to_path_buf(),
        total_size_bytes: total_size,
        max_file_size_bytes: max_file_size,
        max_file_name,
        has_install_wim,
        has_install_esd,
        is_uefi_capable,
        is_windows_7,
        requires_ntfs: max_file_size >= FAT32_MAX_FILE_SIZE,
    })
}

fn walkdir(dir: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut results = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path)?);
            } else {
                results.push(entry);
            }
        }
    }
    Ok(results)
}
