use std::path::{Path, PathBuf};
use woeusb_core::safety::{parse_mounts, verify_device_safety, MountEntry};
use woeusb_core::WoeError;

#[test]
fn test_parse_mounts() {
    let sample = r#"/dev/nvme0n1p2 / ext4 rw,relatime 0 0
/dev/nvme0n1p1 /boot/efi vfat rw,relatime 0 0
/dev/sdb1 /run/media/rinme/FLASH ntfs-3g rw 0 0
tmpfs /run tmpfs rw 0 0
"#;

    let mounts = parse_mounts(sample);
    assert_eq!(mounts.len(), 4);
    assert_eq!(mounts[0].device, PathBuf::from("/dev/nvme0n1p2"));
    assert_eq!(mounts[0].mountpoint, PathBuf::from("/"));
    assert_eq!(mounts[2].device, PathBuf::from("/dev/sdb1"));
    assert_eq!(mounts[2].mountpoint, PathBuf::from("/run/media/rinme/FLASH"));
}

#[test]
fn test_verify_device_safety_blocks_root() {
    let mounts = vec![
        MountEntry {
            device: PathBuf::from("/dev/sda2"),
            mountpoint: PathBuf::from("/"),
            fs_type: "ext4".to_string(),
        },
        MountEntry {
            device: PathBuf::from("/dev/sda1"),
            mountpoint: PathBuf::from("/boot"),
            fs_type: "vfat".to_string(),
        },
    ];
    let swaps = vec![];

    // Attempting to wipe /dev/sda should fail because /dev/sda2 is /
    let res = verify_device_safety(Path::new("/dev/sda"), &mounts, &swaps);
    assert!(res.is_err());
    match res.unwrap_err() {
        WoeError::SafetyViolation(msg) => {
            assert!(msg.contains("root"));
        }
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_verify_device_safety_allows_safe_removable_drive() {
    let mounts = vec![
        MountEntry {
            device: PathBuf::from("/dev/sda2"),
            mountpoint: PathBuf::from("/"),
            fs_type: "ext4".to_string(),
        },
        MountEntry {
            device: PathBuf::from("/dev/sdb1"),
            mountpoint: PathBuf::from("/run/media/user/USB"),
            fs_type: "vfat".to_string(),
        },
    ];
    let swaps = vec![];

    // Target is /dev/sdb (or partition /dev/sdb1)
    let res = verify_device_safety(Path::new("/dev/sdb"), &mounts, &swaps);
    assert!(res.is_ok());
    let report = res.unwrap();
    assert_eq!(report.mounted_partitions.len(), 1);
    assert_eq!(report.mounted_partitions[0].0, PathBuf::from("/dev/sdb1"));
}
