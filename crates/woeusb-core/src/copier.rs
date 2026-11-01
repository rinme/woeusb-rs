use crate::error::{Result, WoeError};
use crate::models::ProgressEvent;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB buffer
pub const LARGE_FILE_THRESHOLD: u64 = 5 * 1024 * 1024; // 5 MiB

pub struct Copier {
    cancel_flag: Arc<AtomicBool>,
}

impl Copier {
    pub fn new(cancel_flag: Arc<AtomicBool>) -> Self {
        Self { cancel_flag }
    }

    pub fn copy_tree<F>(
        &self,
        source_dir: &Path,
        target_dir: &Path,
        on_progress: F,
    ) -> Result<u64>
    where
        F: Fn(ProgressEvent),
    {
        // 1. Calculate total bytes
        let mut total_bytes = 0u64;
        let mut file_list = Vec::new();

        self.collect_files(source_dir, &mut file_list, &mut total_bytes)?;

        let mut copied_bytes = 0u64;
        let start_time = Instant::now();
        let mut last_progress_time = Instant::now();

        for (rel_path, size) in file_list {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(WoeError::Cancelled);
            }

            let src_file = source_dir.join(&rel_path);
            let dst_file = target_dir.join(&rel_path);

            if let Some(parent) = dst_file.parent() {
                fs::create_dir_all(parent)?;
            }

            let rel_str = rel_path.to_string_lossy().to_string();

            if size < LARGE_FILE_THRESHOLD {
                fs::copy(&src_file, &dst_file)?;
                copied_bytes += size;

                let now = Instant::now();
                if now.duration_since(last_progress_time).as_millis() >= 100 {
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let speed = (copied_bytes as f64 / elapsed) as u64;
                    let percentage = if total_bytes > 0 {
                        (copied_bytes as f64 / total_bytes as f64) * 100.0
                    } else {
                        100.0
                    };

                    on_progress(ProgressEvent::ProgressUpdated {
                        bytes_copied: copied_bytes,
                        total_bytes,
                        percentage,
                        speed_bytes_per_sec: speed,
                        current_file: rel_str.clone(),
                    });
                    last_progress_time = now;
                }
            } else {
                // Stream large file in chunks
                let mut reader = File::open(&src_file)?;
                let mut writer = File::create(&dst_file)?;
                let mut buffer = vec![0u8; CHUNK_SIZE];

                loop {
                    if self.cancel_flag.load(Ordering::Relaxed) {
                        return Err(WoeError::Cancelled);
                    }

                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }

                    writer.write_all(&buffer[..bytes_read])?;
                    copied_bytes += bytes_read as u64;

                    let now = Instant::now();
                    if now.duration_since(last_progress_time).as_millis() >= 100 {
                        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                        let speed = (copied_bytes as f64 / elapsed) as u64;
                        let percentage = if total_bytes > 0 {
                            (copied_bytes as f64 / total_bytes as f64) * 100.0
                        } else {
                            100.0
                        };

                        on_progress(ProgressEvent::ProgressUpdated {
                            bytes_copied: copied_bytes,
                            total_bytes,
                            percentage,
                            speed_bytes_per_sec: speed,
                            current_file: rel_str.clone(),
                        });
                        last_progress_time = now;
                    }
                }
                writer.flush()?;
            }
        }

        // Final 100% progress update
        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        let speed = (copied_bytes as f64 / elapsed) as u64;
        on_progress(ProgressEvent::ProgressUpdated {
            bytes_copied: copied_bytes,
            total_bytes,
            percentage: 100.0,
            speed_bytes_per_sec: speed,
            current_file: "Finished copying files".to_string(),
        });

        Ok(copied_bytes)
    }

    fn collect_files(
        &self,
        dir: &Path,
        files: &mut Vec<(std::path::PathBuf, u64)>,
        total_size: &mut u64,
    ) -> Result<()> {
        self.collect_files_recursive(dir, dir, files, total_size)
    }

    fn collect_files_recursive(
        &self,
        base: &Path,
        current: &Path,
        files: &mut Vec<(std::path::PathBuf, u64)>,
        total_size: &mut u64,
    ) -> Result<()> {
        if current.is_dir() {
            for entry in fs::read_dir(current)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.collect_files_recursive(base, &path, files, total_size)?;
                } else if path.is_file() {
                    let size = entry.metadata()?.len();
                    let rel = path
                        .strip_prefix(base)
                        .unwrap_or(&path)
                        .to_path_buf();
                    *total_size += size;
                    files.push((rel, size));
                }
            }
        }
        Ok(())
    }
}
