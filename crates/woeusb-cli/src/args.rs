use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "woeusb",
    version,
    about = "A Linux program to create bootable Windows USB installers from ISO images or DVDs",
    after_help = "Examples:\n  woeusb flash win10.iso /dev/sdb\n  woeusb --device win10.iso /dev/sdb\n  woeusb list\n  woeusb info win11.iso"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Source Windows ISO image or DVD device (Legacy mode)
    #[arg(value_name = "SOURCE")]
    pub source: Option<PathBuf>,

    /// Target USB storage device or partition (Legacy mode)
    #[arg(value_name = "TARGET")]
    pub target: Option<PathBuf>,

    /// Completely WIPE the entire USB storage device, then build a bootable Windows USB device from scratch
    #[arg(short = 'd', long = "device")]
    pub device: bool,

    /// Copy Windows files to an existing partition of a USB storage device and make it bootable
    #[arg(short = 'p', long = "partition")]
    pub partition: bool,

    /// Specify filesystem (FAT or NTFS)
    #[arg(long = "target-filesystem", alias = "tgt-fs", default_value = "FAT")]
    pub target_filesystem: String,

    /// Specify label for the newly created file system
    #[arg(short = 'l', long = "label", default_value = "Windows USB")]
    pub label: String,

    /// Workaround BIOS bug that won't include disk if no boot flag is toggled
    #[arg(long = "workaround-bios-boot-flag")]
    pub workaround_bios_boot_flag: bool,

    /// Skip legacy GRUB bootloader creation step
    #[arg(long = "workaround-skip-grub", alias = "skip-grub")]
    pub skip_grub: bool,

    /// Dry-run simulation mode (does not write to disks)
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Debug mode
    #[arg(long = "debug")]
    pub debug: bool,

    /// Internal worker mode for GUI (outputs JSON lines over stdout)
    #[arg(long = "worker-internal", hide = true)]
    pub worker_internal: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Flash a Windows ISO image to a USB device
    Flash {
        /// Source Windows ISO image or DVD block device
        #[arg(value_name = "SOURCE")]
        source: PathBuf,

        /// Target USB storage device (e.g. /dev/sdb) or partition (/dev/sdb1)
        #[arg(value_name = "TARGET")]
        target: PathBuf,

        /// Flash to an existing partition instead of wiping whole device
        #[arg(short = 'p', long = "partition")]
        partition: bool,

        /// Target filesystem: FAT or NTFS
        #[arg(long = "target-filesystem", alias = "tgt-fs", default_value = "FAT")]
        target_filesystem: String,

        /// Volume label
        #[arg(short = 'l', long = "label", default_value = "Windows USB")]
        label: String,

        /// Workaround BIOS bug that requires a boot flag
        #[arg(long = "workaround-bios-boot-flag")]
        workaround_bios_boot_flag: bool,

        /// Skip legacy GRUB bootloader installation
        #[arg(long = "skip-grub")]
        skip_grub: bool,

        /// Dry-run simulation mode (does not write to disks)
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Skip confirmation prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Internal worker mode (outputs JSON lines over stdout)
        #[arg(long = "worker", hide = true)]
        worker: bool,
    },

    /// List connected USB drives and optical disc devices
    List {
        /// Show all block devices, even non-removable ones
        #[arg(short = 'a', long = "all")]
        all: bool,
    },

    /// Inspect a Windows ISO image or DVD
    Info {
        /// Path to Windows ISO image or optical drive
        #[arg(value_name = "SOURCE")]
        source: PathBuf,
    },
}
