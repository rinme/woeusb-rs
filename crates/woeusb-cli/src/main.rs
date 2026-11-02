mod args;
mod terminal;
mod worker;

use anyhow::Result;
use clap::Parser;
use woeusb_core::device::{list_dvd_drives, list_usb_drives};
use woeusb_core::iso::inspect_iso;
use woeusb_core::models::{FlashConfig, InstallMode, TargetFilesystem};

fn main() -> Result<()> {
    let cli = args::Cli::parse();

    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    if let Some(cmd) = cli.command {
        match cmd {
            args::Commands::Flash {
                source,
                target,
                partition,
                target_filesystem,
                label,
                workaround_bios_boot_flag,
                skip_grub,
                dry_run,
                yes,
                worker,
            } => {
                let fs = parse_filesystem(&target_filesystem);
                let mode = if partition {
                    InstallMode::Partition
                } else {
                    InstallMode::Device
                };

                let config = FlashConfig {
                    source,
                    target,
                    install_mode: mode,
                    target_filesystem: fs,
                    filesystem_label: label,
                    workaround_bios_boot_flag,
                    skip_grub,
                    dry_run,
                };

                if worker {
                    worker::run_worker(config)?;
                } else {
                    terminal::run_interactive(config, yes)?;
                }
            }
            args::Commands::List { all } => {
                let mut drives = list_usb_drives(all)?;
                let mut dvds = list_dvd_drives().unwrap_or_default();
                drives.append(&mut dvds);
                terminal::print_drives_table(&drives);
            }
            args::Commands::Info { source } => {
                let meta = inspect_iso(&source)?;
                terminal::print_iso_info(&meta);
            }
            args::Commands::Completions { shell, bin } => {
                use clap::CommandFactory;
                let mut cmd = args::Cli::command();
                clap_complete::generate(shell, &mut cmd, bin, &mut std::io::stdout());
            }
        }
    } else if let (Some(source), Some(target)) = (cli.source, cli.target) {
        // Legacy CLI compatibility mode
        let mode = if cli.partition {
            InstallMode::Partition
        } else {
            InstallMode::Device
        };
        let fs = parse_filesystem(&cli.target_filesystem);

        let config = FlashConfig {
            source,
            target,
            install_mode: mode,
            target_filesystem: fs,
            filesystem_label: cli.label,
            workaround_bios_boot_flag: cli.workaround_bios_boot_flag,
            skip_grub: cli.skip_grub,
            dry_run: cli.dry_run,
        };

        if cli.worker_internal {
            worker::run_worker(config)?;
        } else {
            terminal::run_interactive(config, cli.yes)?;
        }
    } else {
        // No command or parameters provided, print help
        use clap::CommandFactory;
        args::Cli::command().print_help()?;
        println!();
    }

    Ok(())
}

fn parse_filesystem(fs_str: &str) -> TargetFilesystem {
    match fs_str.to_uppercase().as_str() {
        "NTFS" => TargetFilesystem::Ntfs,
        _ => TargetFilesystem::Fat,
    }
}
