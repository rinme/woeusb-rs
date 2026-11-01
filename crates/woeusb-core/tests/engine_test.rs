use std::fs;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use woeusb_core::copier::Copier;
use woeusb_core::engine::Engine;
use woeusb_core::models::{FlashConfig, InstallMode, ProgressEvent, TargetFilesystem};

#[test]
fn test_copier_copies_files_and_reports_progress() {
    let src_dir = tempdir().expect("create src dir");
    let tgt_dir = tempdir().expect("create tgt dir");

    // Create some sample files
    fs::write(src_dir.path().join("bootmgr"), "bootmgr content").unwrap();
    fs::create_dir_all(src_dir.path().join("sources")).unwrap();
    fs::write(
        src_dir.path().join("sources/boot.wim"),
        vec![0u8; 1024 * 1024], // 1 MB
    )
    .unwrap();

    let cancel = Arc::new(AtomicBool::new(false));
    let copier = Copier::new(cancel);

    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let pe = progress_events.clone();

    let copied = copier
        .copy_tree(src_dir.path(), tgt_dir.path(), move |event| {
            pe.lock().unwrap().push(event);
        })
        .expect("copy tree");

    assert!(copied > 0);
    assert!(tgt_dir.path().join("bootmgr").exists());
    assert!(tgt_dir.path().join("sources/boot.wim").exists());

    let events = progress_events.lock().unwrap();
    assert!(!events.is_empty());
}

#[test]
fn test_engine_dry_run_workflow() {
    let src_dir = tempdir().expect("create src dir");
    let dummy_iso = src_dir.path().join("windows11.iso");
    fs::write(&dummy_iso, "dummy iso content").unwrap();

    let config = FlashConfig {
        source: dummy_iso,
        target: std::path::PathBuf::from("/dev/sdb"),
        install_mode: InstallMode::Device,
        target_filesystem: TargetFilesystem::Ntfs,
        filesystem_label: "WIN_INSTALL".to_string(),
        workaround_bios_boot_flag: true,
        skip_grub: false,
        dry_run: true,
    };

    let engine = Engine::new(config);
    let events = Arc::new(Mutex::new(Vec::new()));
    let ev = events.clone();

    let res = engine.run(move |event| {
        ev.lock().unwrap().push(event);
    });

    assert!(res.is_ok(), "Engine dry-run should succeed: {:?}", res);

    let recorded = events.lock().unwrap();
    let has_completed = recorded.iter().any(|e| matches!(e, ProgressEvent::Completed { .. }));
    assert!(has_completed, "Dry run should emit ProgressEvent::Completed");

    let step_count = recorded
        .iter()
        .filter(|e| matches!(e, ProgressEvent::StepStarted { .. }))
        .count();
    assert_eq!(step_count, 8, "Expected 8 steps in dry-run workflow");
}
