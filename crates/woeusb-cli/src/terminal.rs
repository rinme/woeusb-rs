use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::path::Path;
use woeusb_core::engine::Engine;
use woeusb_core::iso::IsoMetadata;
use woeusb_core::models::{format_bytes, Drive, FlashConfig, ProgressEvent};

pub fn run_interactive(config: FlashConfig, assume_yes: bool) -> Result<()> {
    if !assume_yes && !config.dry_run {
        prompt_user_confirmation(&config.source, &config.target, config.install_mode.to_string())?;
    }

    let pb_step = ProgressBar::new(8);
    pb_step.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:20.cyan/blue}] Step {pos}/{len}: {msg}")
            .expect("Valid template"),
    );
    pb_step.set_prefix("WoeUSB");

    let pb_copy = ProgressBar::new(100);
    pb_copy.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:30.green/black}] {percent}% ({bytes_per_sec}) {msg}")
            .expect("Valid template")
            .progress_chars("#>-"),
    );

    let engine = Engine::new(config);

    engine.run(|event| match event {
        ProgressEvent::StepStarted {
            step,
            total_steps,
            message,
        } => {
            pb_step.set_length(total_steps as u64);
            pb_step.set_position(step as u64);
            pb_step.set_message(message);
        }
        ProgressEvent::StepFinished { step: _, message } => {
            pb_step.println(format!("✓ {}", message));
        }
        ProgressEvent::ProgressUpdated {
            bytes_copied,
            total_bytes,
            percentage,
            speed_bytes_per_sec,
            current_file,
        } => {
            pb_copy.set_length(total_bytes);
            pb_copy.set_position(bytes_copied);
            let speed_str = format!("{}/s", format_bytes(speed_bytes_per_sec));
            let short_file = if current_file.len() > 30 {
                format!("...{}", &current_file[current_file.len() - 27..])
            } else {
                current_file
            };
            pb_copy.set_message(format!("{} | {:.1}% ({})", short_file, percentage, speed_str));
        }
        ProgressEvent::LogMessage { level, message } => {
            pb_step.println(format!("[{}] {}", level, message));
        }
        ProgressEvent::Completed { message } => {
            pb_copy.finish_and_clear();
            pb_step.finish_with_message("Done!");
            println!("\n\x1b[1;32m✓ {}\x1b[0m", message);
        }
        ProgressEvent::Error { message } => {
            pb_copy.finish_and_clear();
            eprintln!("\n\x1b[1;31m✗ Error: {}\x1b[0m", message);
        }
    })?;

    Ok(())
}

fn prompt_user_confirmation(source: &Path, target: &Path, mode: String) -> Result<()> {
    println!("\x1b[1;33mWARNING: All data on target {:?} will be completely WIPED!\x1b[0m", target);
    println!("Source: {:?}", source);
    println!("Target: {:?}", target);
    println!("Mode:   {}", mode);
    print!("\nAre you sure you want to proceed? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if trimmed != "y" && trimmed != "yes" {
        println!("Aborted by user.");
        std::process::exit(0);
    }
    Ok(())
}

pub fn print_drives_table(drives: &[Drive]) {
    if drives.is_empty() {
        println!("No eligible storage devices found.");
        return;
    }

    println!("\x1b[1m{:<12} {:<24} {:<12} {:<12} {:<16}\x1b[0m", "Device", "Model", "Capacity", "Removable", "Mountpoints");
    println!("{:-<76}", "");

    for d in drives {
        let mps = if d.mountpoints.is_empty() {
            "-".to_string()
        } else {
            d.mountpoints
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "{:<12} {:<24} {:<12} {:<12} {:<16}",
            d.path.display(),
            if d.model.len() > 22 { &d.model[..22] } else { &d.model },
            d.size_formatted,
            if d.is_removable { "Yes" } else { "No (Internal)" },
            mps
        );
    }
}

pub fn print_iso_info(meta: &IsoMetadata) {
    println!("\x1b[1mWindows Image Information:\x1b[0m");
    println!("  Path:                  {}", meta.path.display());
    println!("  Total Size:            {}", format_bytes(meta.total_size_bytes));
    println!("  Largest File:          {} ({})", meta.max_file_name, format_bytes(meta.max_file_size_bytes));
    println!("  Has install.wim:       {}", if meta.has_install_wim { "Yes" } else { "No" });
    println!("  Has install.esd:       {}", if meta.has_install_esd { "Yes" } else { "No" });
    println!("  UEFI Boot Capable:     {}", if meta.is_uefi_capable { "Yes" } else { "No" });
    println!("  Requires NTFS:         {}", if meta.requires_ntfs { "Yes (exceeds 4GiB FAT32 limit)" } else { "No" });
    println!("  Windows 7 Legacy EFI:  {}", if meta.is_windows_7 { "Yes (workaround required)" } else { "No" });
}
