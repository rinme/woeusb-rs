use std::path::PathBuf;
use woeusb_core::models::{format_bytes, Drive, InstallMode, ProgressEvent, TargetFilesystem};

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1024), "1.00 KiB");
    assert_eq!(format_bytes(1024 * 1024 * 15), "15.00 MiB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 32), "32.00 GiB");
}

#[test]
fn test_drive_system_detection() {
    let non_system = Drive {
        name: "sdb".to_string(),
        path: PathBuf::from("/dev/sdb"),
        model: "USB Flash".to_string(),
        size_bytes: 16_000_000_000,
        size_formatted: "14.9 GiB".to_string(),
        is_removable: true,
        is_read_only: false,
        partitions: vec![],
        mountpoints: vec![PathBuf::from("/run/media/user/MYUSB")],
    };
    assert!(!non_system.is_system_drive());

    let system_drive = Drive {
        name: "nvme0n1".to_string(),
        path: PathBuf::from("/dev/nvme0n1"),
        model: "Samsung SSD".to_string(),
        size_bytes: 512_000_000_000,
        size_formatted: "476 GiB".to_string(),
        is_removable: false,
        is_read_only: false,
        partitions: vec![],
        mountpoints: vec![PathBuf::from("/"), PathBuf::from("/boot")],
    };
    assert!(system_drive.is_system_drive());
}

#[test]
fn test_progress_event_serialization() {
    let event = ProgressEvent::ProgressUpdated {
        bytes_copied: 1024,
        total_bytes: 2048,
        percentage: 50.0,
        speed_bytes_per_sec: 512,
        current_file: "boot.wim".to_string(),
    };

    let json = serde_json::to_string(&event).expect("Serialize to json");
    assert!(json.contains("boot.wim"));
    assert!(json.contains("ProgressUpdated"));

    let deserialized: ProgressEvent = serde_json::from_str(&json).expect("Deserialize from json");
    match deserialized {
        ProgressEvent::ProgressUpdated { current_file, percentage, .. } => {
            assert_eq!(current_file, "boot.wim");
            assert_eq!(percentage, 50.0);
        }
        _ => panic!("Unexpected event variant"),
    }
}

#[test]
fn test_enums_display() {
    assert_eq!(TargetFilesystem::Fat.to_string(), "FAT32");
    assert_eq!(TargetFilesystem::Ntfs.to_string(), "NTFS");
    assert_eq!(InstallMode::Device.to_string(), "device");
    assert_eq!(InstallMode::Partition.to_string(), "partition");
}
