use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use woeusb_core::models::{FlashConfig, ProgressEvent};

pub struct WorkerClient;

impl WorkerClient {
    pub fn spawn_worker(config: FlashConfig, tx: Sender<ProgressEvent>) {
        thread::spawn(move || {
            let mut cmd = if config.dry_run {
                Command::new("woeusb")
            } else {
                let mut pk = Command::new("pkexec");
                pk.arg("woeusb");
                pk
            };

            cmd.arg("--worker-internal");
            cmd.arg("flash");
            cmd.arg(&config.source);
            cmd.arg(&config.target);

            cmd.arg("--target-filesystem");
            cmd.arg(config.target_filesystem.to_string());

            cmd.arg("--label");
            cmd.arg(&config.filesystem_label);

            if config.workaround_bios_boot_flag {
                cmd.arg("--workaround-bios-boot-flag");
            }

            if config.skip_grub {
                cmd.arg("--skip-grub");
            }

            if config.dry_run {
                cmd.arg("--dry-run");
            }

            cmd.arg("-y");

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(err) => {
                    let _ = tx.send(ProgressEvent::Error {
                        message: format!("Failed to launch worker process: {}", err),
                    });
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if let Ok(event) = serde_json::from_str::<ProgressEvent>(&line) {
                        let _ = tx.send(event);
                    }
                }
            }

            let status = child.wait();
            match status {
                Ok(exit) if !exit.success() => {
                    let _ = tx.send(ProgressEvent::Error {
                        message: format!("Worker process exited with code {:?}", exit.code()),
                    });
                }
                Err(e) => {
                    let _ = tx.send(ProgressEvent::Error {
                        message: format!("Worker wait error: {}", e),
                    });
                }
                _ => {}
            }
        });
    }
}
