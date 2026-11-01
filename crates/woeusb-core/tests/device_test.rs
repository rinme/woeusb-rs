use woeusb_core::device::{list_dvd_drives, list_usb_drives};

#[test]
fn test_list_usb_drives_runs_without_panic() {
    let drives = list_usb_drives(true).expect("Listing all drives should succeed");
    // Should not crash and return a valid vector of Drives
    for drive in drives {
        assert!(!drive.name.is_empty());
        assert!(drive.path.to_str().unwrap().starts_with("/dev/"));
    }
}

#[test]
fn test_list_dvd_drives_runs_without_panic() {
    let dvds = list_dvd_drives().expect("Listing DVD drives should succeed");
    for dvd in dvds {
        assert!(dvd.name.starts_with("sr"));
    }
}
