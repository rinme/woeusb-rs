pub const UEFI_NTFS_IMAGE: &[u8] = include_bytes!("../assets/uefi-ntfs.img");

pub fn get_uefi_ntfs_image() -> &'static [u8] {
    UEFI_NTFS_IMAGE
}
