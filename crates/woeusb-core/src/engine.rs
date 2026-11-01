use crate::assets::get_uefi_ntfs_image;
use crate::copier::Copier;
use crate::error::{Result, WoeError};
use crate::iso::{inspect_iso, FAT32_MAX_FILE_SIZE};
use crate::models::{FlashConfig, InstallMode, ProgressEvent, TargetFilesystem};
use crate::safety::{get_system_mounts, get_system_swaps, verify_device_safety};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::info;

pub struct Engine {
    config: FlashConfig,
    cancel_flag: Arc<AtomicBool>,
}

impl Engine {
    pub fn new(config: FlashConfig) -> Self {
        Self {
            config,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn run<F>(&self, on_progress: F) -> Result<()>
    where
        F: Fn(ProgressEvent),
    {
        let total_steps = 8;
        let mut current_step = 1;

        // Step 1: Pre-flight checks and safety verification
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Performing safety and environment checks...".to_string(),
        });

        if !self.config.source.exists() {
            return Err(WoeError::InvalidSource(format!(
                "Source image does not exist: {:?}",
                self.config.source
            )));
        }

        // Verify target safety against mounted system filesystems
        let mounts = get_system_mounts().unwrap_or_default();
        let swaps = get_system_swaps().unwrap_or_default();
        let safety_report = verify_device_safety(&self.config.target, &mounts, &swaps)?;

        for warning in &safety_report.warnings {
            on_progress(ProgressEvent::LogMessage {
                level: "WARN".to_string(),
                message: warning.clone(),
            });
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "Safety checks passed.".to_string(),
        });
        current_step += 1;

        // Step 2: Inspect source image / DVD
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Inspecting Windows installation image...".to_string(),
        });

        let iso_meta = inspect_iso(&self.config.source)?;
        let mut target_fs = self.config.target_filesystem;

        if target_fs == TargetFilesystem::Fat && iso_meta.max_file_size_bytes >= FAT32_MAX_FILE_SIZE {
            on_progress(ProgressEvent::LogMessage {
                level: "INFO".to_string(),
                message: format!(
                    "File '{}' ({:.2} GiB) exceeds FAT32 4GiB limit. Auto-switching to NTFS with UEFI:NTFS support.",
                    iso_meta.max_file_name,
                    iso_meta.max_file_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
            });
            target_fs = TargetFilesystem::Ntfs;
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: format!("Image verified. Target filesystem: {}", target_fs),
        });
        current_step += 1;

        // Determine target partitions
        let (target_part1, target_part2) = self.get_target_partitions(&self.config.target)?;

        // Step 3: Unmount any busy partitions on target
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Unmounting busy partitions on target...".to_string(),
        });

        if !self.config.dry_run {
            for (part_dev, mountpoint) in &safety_report.mounted_partitions {
                on_progress(ProgressEvent::LogMessage {
                    level: "INFO".to_string(),
                    message: format!("Unmounting {:?} at {:?}", part_dev, mountpoint),
                });
                let _ = self.run_command("umount", &[mountpoint.to_string_lossy().as_ref()]);
            }
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "Target unmounted and ready.".to_string(),
        });
        current_step += 1;

        // Step 4: Partitioning & Formatting
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Partitioning and formatting target storage device...".to_string(),
        });

        if self.config.install_mode == InstallMode::Device {
            self.wipe_and_partition_device(&target_part1, &target_part2, target_fs, &on_progress)?;
        }

        self.format_partition(&target_part1, target_fs, &on_progress)?;

        if target_fs == TargetFilesystem::Ntfs && self.config.install_mode == InstallMode::Device {
            self.install_uefi_ntfs_partition(&target_part2, &on_progress)?;
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "Partitioning and formatting complete.".to_string(),
        });
        current_step += 1;

        // Step 5: Mount filesystems
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Mounting source image and target filesystem...".to_string(),
        });

        let source_temp = tempfile::Builder::new()
            .prefix("woeusb_src_")
            .tempdir()?;
        let target_temp = tempfile::Builder::new()
            .prefix("woeusb_tgt_")
            .tempdir()?;

        let source_mnt = source_temp.path();
        let target_mnt = target_temp.path();

        if !self.config.dry_run {
            self.mount_source(&self.config.source, source_mnt)?;
            self.mount_target(&target_part1, target_mnt)?;
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "Filesystems mounted successfully.".to_string(),
        });
        current_step += 1;

        // Step 6: Copy files with progress
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Copying Windows installation files to USB...".to_string(),
        });

        if self.config.dry_run {
            // In dry run, simulate progress
            on_progress(ProgressEvent::ProgressUpdated {
                bytes_copied: iso_meta.total_size_bytes,
                total_bytes: iso_meta.total_size_bytes,
                percentage: 100.0,
                speed_bytes_per_sec: 25 * 1024 * 1024,
                current_file: "[DRY-RUN Simulation: All files transferred]".to_string(),
            });
        } else {
            let copier = Copier::new(self.cancel_flag.clone());
            copier.copy_tree(source_mnt, target_mnt, &on_progress)?;
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "All installation files successfully copied.".to_string(),
        });
        current_step += 1;

        // Step 7: Bootloader & Workarounds
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Installing bootloader and compatibility configurations...".to_string(),
        });

        if !self.config.dry_run {
            // Windows 7 UEFI workaround if needed
            if iso_meta.is_windows_7 {
                self.apply_win7_uefi_workaround(source_mnt, target_mnt, &on_progress);
            }

            // Legacy GRUB bootloader
            if !self.config.skip_grub {
                self.install_grub(target_mnt, &self.config.target, &on_progress)?;
            }

            // BIOS boot flag workaround
            if self.config.workaround_bios_boot_flag {
                self.set_boot_flag(&self.config.target, &on_progress)?;
            }
        } else {
            on_progress(ProgressEvent::LogMessage {
                level: "INFO".to_string(),
                message: "[DRY-RUN] Skipped bootloader installation commands.".to_string(),
            });
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "Bootloader installation finished.".to_string(),
        });
        current_step += 1;

        // Step 8: Sync and unmount
        on_progress(ProgressEvent::StepStarted {
            step: current_step,
            total_steps,
            message: "Flushing filesystem buffers and finalizing...".to_string(),
        });

        if !self.config.dry_run {
            info!("Syncing disks...");
            let _ = self.run_command("sync", &[]);

            let _ = self.run_command("umount", &[target_mnt.to_string_lossy().as_ref()]);
            let _ = self.run_command("umount", &[source_mnt.to_string_lossy().as_ref()]);
        }

        on_progress(ProgressEvent::StepFinished {
            step: current_step,
            message: "USB storage device safely synced and ready.".to_string(),
        });

        on_progress(ProgressEvent::Completed {
            message: "Success! Bootable Windows USB installer created successfully.".to_string(),
        });

        Ok(())
    }

    fn get_target_partitions(&self, target: &Path) -> Result<(PathBuf, PathBuf)> {
        let target_str = target.to_string_lossy();
        if self.config.install_mode == InstallMode::Partition {
            return Ok((target.to_path_buf(), target.to_path_buf()));
        }

        // For NVMe/eMMC: /dev/nvme0n1 -> /dev/nvme0n1p1, /dev/nvme0n1p2
        // For SCSI/SATA: /dev/sdb -> /dev/sdb1, /dev/sdb2
        let last_char = target_str.chars().last().unwrap_or(' ');
        if last_char.is_ascii_digit() {
            Ok((
                PathBuf::from(format!("{}p1", target_str)),
                PathBuf::from(format!("{}p2", target_str)),
            ))
        } else {
            Ok((
                PathBuf::from(format!("{}1", target_str)),
                PathBuf::from(format!("{}2", target_str)),
            ))
        }
    }

    fn wipe_and_partition_device(
        &self,
        part1: &Path,
        _part2: &Path,
        fs_type: TargetFilesystem,
        on_progress: &impl Fn(ProgressEvent),
    ) -> Result<()> {
        let dev_str = self.config.target.to_string_lossy();
        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: format!("Wiping partition table on {}", dev_str),
        });

        if self.config.dry_run {
            return Ok(());
        }

        self.run_command("wipefs", &["--all", "--force", dev_str.as_ref()])?;
        self.run_command("parted", &["--script", dev_str.as_ref(), "mklabel", "msdos"])?;

        match fs_type {
            TargetFilesystem::Fat => {
                self.run_command(
                    "parted",
                    &[
                        "--script",
                        dev_str.as_ref(),
                        "mkpart",
                        "primary",
                        "fat32",
                        "4MiB",
                        "100%",
                    ],
                )?;
            }
            TargetFilesystem::Ntfs => {
                // Partition 1 for NTFS
                self.run_command(
                    "parted",
                    &[
                        "--script",
                        dev_str.as_ref(),
                        "mkpart",
                        "primary",
                        "ntfs",
                        "4MiB",
                        "--",
                        "-2049s",
                    ],
                )?;
                // Partition 2 for UEFI:NTFS boot image (FAT16/FAT12)
                self.run_command(
                    "parted",
                    &[
                        "--align",
                        "none",
                        "--script",
                        dev_str.as_ref(),
                        "mkpart",
                        "primary",
                        "fat16",
                        "--",
                        "-2048s",
                        "-1s",
                    ],
                )?;
            }
        }

        // Reread partition table
        let _ = self.run_command("blockdev", &["--rereadpt", dev_str.as_ref()]);
        thread::sleep(Duration::from_millis(1500));

        // Ensure partition node exists
        let mut waited = 0;
        while !part1.exists() && waited < 10 {
            thread::sleep(Duration::from_millis(300));
            waited += 1;
        }

        Ok(())
    }

    fn format_partition(
        &self,
        part1: &Path,
        fs_type: TargetFilesystem,
        on_progress: &impl Fn(ProgressEvent),
    ) -> Result<()> {
        let part_str = part1.to_string_lossy();
        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: format!("Formatting {} as {}", part_str, fs_type),
        });

        if self.config.dry_run {
            return Ok(());
        }

        match fs_type {
            TargetFilesystem::Fat => {
                let mkfs_fat = if self.has_command("mkfs.vfat") {
                    "mkfs.vfat"
                } else {
                    "mkdosfs"
                };
                self.run_command(
                    mkfs_fat,
                    &[
                        "-F",
                        "32",
                        "-n",
                        &self.config.filesystem_label,
                        part_str.as_ref(),
                    ],
                )?;
            }
            TargetFilesystem::Ntfs => {
                self.run_command(
                    "mkntfs",
                    &[
                        "--quick",
                        "--label",
                        &self.config.filesystem_label,
                        part_str.as_ref(),
                    ],
                )?;
            }
        }
        Ok(())
    }

    fn install_uefi_ntfs_partition(
        &self,
        part2: &Path,
        on_progress: &impl Fn(ProgressEvent),
    ) -> Result<()> {
        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: format!("Writing embedded UEFI:NTFS boot image to {:?}", part2),
        });

        if self.config.dry_run {
            return Ok(());
        }

        let uefi_img = get_uefi_ntfs_image();
        let mut file = File::create(part2)?;
        file.write_all(uefi_img)?;
        file.flush()?;

        Ok(())
    }

    fn mount_source(&self, source: &Path, mnt: &Path) -> Result<()> {
        let src_str = source.to_string_lossy();
        let mnt_str = mnt.to_string_lossy();

        if source.is_file() {
            self.run_command(
                "mount",
                &[
                    "-o",
                    "loop,ro",
                    "-t",
                    "udf,iso9660",
                    src_str.as_ref(),
                    mnt_str.as_ref(),
                ],
            )?;
        } else {
            self.run_command("mount", &["-o", "ro", src_str.as_ref(), mnt_str.as_ref()])?;
        }
        Ok(())
    }

    fn mount_target(&self, target_part: &Path, mnt: &Path) -> Result<()> {
        let part_str = target_part.to_string_lossy();
        let mnt_str = mnt.to_string_lossy();
        self.run_command("mount", &[part_str.as_ref(), mnt_str.as_ref()])?;
        Ok(())
    }

    fn install_grub(
        &self,
        target_mnt: &Path,
        target_dev: &Path,
        on_progress: &impl Fn(ProgressEvent),
    ) -> Result<()> {
        let grub_cmd = if self.has_command("grub-install") {
            "grub-install"
        } else if self.has_command("grub2-install") {
            "grub2-install"
        } else {
            on_progress(ProgressEvent::LogMessage {
                level: "WARN".to_string(),
                message: "grub-install not found. Skipping legacy GRUB installation.".to_string(),
            });
            return Ok(());
        };

        let boot_dir = format!("--boot-directory={}", target_mnt.display());
        let dev_str = target_dev.to_string_lossy();

        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: format!("Installing GRUB for Legacy BIOS boot on {}", dev_str),
        });

        let _ = self.run_command(
            grub_cmd,
            &[
                "--target=i386-pc",
                &boot_dir,
                "--force",
                dev_str.as_ref(),
            ],
        );

        // Write custom chainload grub.cfg
        let grub_dir_name = if grub_cmd == "grub2-install" {
            "grub2"
        } else {
            "grub"
        };
        let cfg_dir = target_mnt.join(grub_dir_name);
        fs::create_dir_all(&cfg_dir)?;
        let cfg_path = cfg_dir.join("grub.cfg");
        fs::write(cfg_path, "ntldr /bootmgr\nboot\n")?;

        Ok(())
    }

    fn apply_win7_uefi_workaround(
        &self,
        _src_mnt: &Path,
        tgt_mnt: &Path,
        on_progress: &impl Fn(ProgressEvent),
    ) {
        let efi_boot = tgt_mnt.join("efi/boot");
        let bootx64 = efi_boot.join("bootx64.efi");

        if bootx64.exists() {
            return;
        }

        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: "Applying Windows 7 UEFI bootloader workaround...".to_string(),
        });

        let _ = fs::create_dir_all(&efi_boot);
        // Attempt extraction from sources/boot.wim via 7z if available
        let boot_wim = tgt_mnt.join("sources/boot.wim");
        if boot_wim.exists() {
            let out_arg = format!("-o{}", efi_boot.display());
            let _ = Command::new("7z")
                .args(["e", "-y", &boot_wim.to_string_lossy(), "1/Windows/Boot/EFI/bootmgfw.efi", &out_arg])
                .output();
            let extracted = efi_boot.join("bootmgfw.efi");
            if extracted.exists() {
                let _ = fs::rename(extracted, bootx64);
            }
        }
    }

    fn set_boot_flag(&self, target_dev: &Path, on_progress: &impl Fn(ProgressEvent)) -> Result<()> {
        let dev_str = target_dev.to_string_lossy();
        on_progress(ProgressEvent::LogMessage {
            level: "INFO".to_string(),
            message: format!("Setting boot flag on partition 1 of {}", dev_str),
        });
        self.run_command("parted", &["--script", dev_str.as_ref(), "set", "1", "boot", "on"])?;
        Ok(())
    }

    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| WoeError::CommandFailed {
                command: format!("{} {}", cmd, args.join(" ")),
                code: None,
                stderr: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(WoeError::CommandFailed {
                command: format!("{} {}", cmd, args.join(" ")),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn has_command(&self, cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
