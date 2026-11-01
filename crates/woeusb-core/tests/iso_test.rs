use std::path::Path;
use woeusb_core::assets::get_uefi_ntfs_image;
use woeusb_core::iso::{parse_7z_slt_output, FAT32_MAX_FILE_SIZE};

#[test]
fn test_embedded_uefi_ntfs_image() {
    let img = get_uefi_ntfs_image();
    assert!(!img.is_empty());
    // Rufus uefi-ntfs.img has OEM label RUFUS_BOOT in FAT boot sector
    let s = String::from_utf8_lossy(img);
    assert!(s.contains("RUFUS_BOOT") || s.contains("FAT"));
}

#[test]
fn test_parse_7z_slt_output_detects_wim_and_uefi() {
    let sample = r#"Path = boot/boot.sdi
Folder = -
Size = 3176448

Path = sources/install.wim
Folder = -
Size = 5120000000

Path = efi/boot/bootx64.efi
Folder = -
Size = 1536000

"#;

    let meta = parse_7z_slt_output(Path::new("dummy.iso"), sample).expect("Parse metadata");
    assert!(meta.has_install_wim);
    assert!(!meta.has_install_esd);
    assert!(meta.is_uefi_capable);
    assert!(meta.max_file_size_bytes > FAT32_MAX_FILE_SIZE);
    assert!(meta.requires_ntfs);
    assert_eq!(meta.max_file_name, "sources/install.wim");
}

#[test]
fn test_parse_7z_slt_output_fat32_compatible() {
    let sample = r#"Path = sources/install.esd
Folder = -
Size = 3500000000

Path = efi/boot/bootx64.efi
Folder = -
Size = 1536000

"#;

    let meta = parse_7z_slt_output(Path::new("win10_small.iso"), sample).expect("Parse metadata");
    assert!(!meta.has_install_wim);
    assert!(meta.has_install_esd);
    assert!(meta.is_uefi_capable);
    assert!(!meta.requires_ntfs);
}
