use crate::error::{Result, WoeError};
use crate::models::{format_bytes, Drive, Partition};
use crate::safety::get_system_mounts;
use std::fs;
use std::path::{Path, PathBuf};

pub fn list_usb_drives(show_all: bool) -> Result<Vec<Drive>> {
    let mut drives = Vec::new();
    let sys_block = Path::new("/sys/block");
    if !sys_block.exists() {
        return Ok(drives);
    }

    let mounts = get_system_mounts().unwrap_or_default();

    for entry in fs::read_dir(sys_block)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip non-disk devices
        if name.starts_with("loop")
            || name.starts_with("ram")
            || name.starts_with("sr")
            || name.starts_with("cdrom")
            || name.starts_with("zram")
        {
            continue;
        }

        let dev_path = sys_block.join(&name);

        let removable = fs::read_to_string(dev_path.join("removable"))
            .unwrap_or_default()
            .trim()
            == "1";

        let ro = fs::read_to_string(dev_path.join("ro"))
            .unwrap_or_default()
            .trim()
            == "1";

        if !show_all && (!removable || ro) {
            continue;
        }

        let sectors: u64 = fs::read_to_string(dev_path.join("size"))
            .unwrap_or_default()
            .trim()
            .parse()
            .unwrap_or(0);
        let size_bytes = sectors * 512;

        let model = fs::read_to_string(dev_path.join("device/model"))
            .or_else(|_| fs::read_to_string(dev_path.join("device/name")))
            .unwrap_or_else(|_| "Generic Storage Device".to_string())
            .trim()
            .to_string();

        let vendor = fs::read_to_string(dev_path.join("device/vendor"))
            .unwrap_or_default()
            .trim()
            .to_string();

        let full_model = if !vendor.is_empty() && !model.starts_with(&vendor) {
            format!("{} {}", vendor, model)
        } else {
            model
        };

        let block_dev = PathBuf::from(format!("/dev/{}", name));

        // Enumerate partitions
        let mut partitions = Vec::new();
        let mut drive_mountpoints = Vec::new();

        if let Ok(dir_entries) = fs::read_dir(&dev_path) {
            for sub_entry in dir_entries.flatten() {
                let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                if sub_name.starts_with(&name) && sub_name != name {
                    let part_dev = PathBuf::from(format!("/dev/{}", sub_name));
                    let part_sectors: u64 = fs::read_to_string(sub_entry.path().join("size"))
                        .unwrap_or_default()
                        .trim()
                        .parse()
                        .unwrap_or(0);
                    let part_size_bytes = part_sectors * 512;

                    // Match with mountpoints
                    let part_mountpoint = mounts
                        .iter()
                        .find(|m| m.device == part_dev)
                        .map(|m| m.mountpoint.clone());

                    if let Some(ref mp) = part_mountpoint {
                        drive_mountpoints.push(mp.clone());
                    }

                    partitions.push(Partition {
                        name: sub_name,
                        path: part_dev,
                        size_bytes: part_size_bytes,
                        filesystem: None,
                        mountpoint: part_mountpoint,
                        label: None,
                    });
                }
            }
        }

        // Also check if the whole drive is mounted directly
        if let Some(m) = mounts.iter().find(|m| m.device == block_dev) {
            drive_mountpoints.push(m.mountpoint.clone());
        }

        drives.push(Drive {
            name,
            path: block_dev,
            model: full_model,
            size_bytes,
            size_formatted: format_bytes(size_bytes),
            is_removable: removable,
            is_read_only: ro,
            partitions,
            mountpoints: drive_mountpoints,
        });
    }

    Ok(drives)
}

pub fn list_dvd_drives() -> Result<Vec<Drive>> {
    let mut drives = Vec::new();
    let sys_block = Path::new("/sys/block");
    if !sys_block.exists() {
        return Ok(drives);
    }

    for entry in fs::read_dir(sys_block)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with("sr") {
            let dev_path = sys_block.join(&name);
            let model = fs::read_to_string(dev_path.join("device/model"))
                .unwrap_or_else(|_| "Optical Drive".to_string())
                .trim()
                .to_string();

            let block_dev = PathBuf::from(format!("/dev/{}", name));

            drives.push(Drive {
                name,
                path: block_dev,
                model,
                size_bytes: 0,
                size_formatted: "Optical Disc".to_string(),
                is_removable: true,
                is_read_only: true,
                partitions: vec![],
                mountpoints: vec![],
            });
        }
    }

    Ok(drives)
}

pub fn inspect_device(target: &Path) -> Result<Drive> {
    let drives = list_usb_drives(true)?;
    let target_canonical = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());

    drives
        .into_iter()
        .find(|d| d.path == target || d.path == target_canonical)
        .ok_or_else(|| WoeError::DeviceNotFound(target.to_path_buf()))
}
