use crate::worker_client::WorkerClient;
use adw::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, DropDown, FileChooserAction, FileChooserNative, FileFilter,
    Label, Orientation, ProgressBar, ResponseType, StringList, Switch,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use woeusb_core::device::list_usb_drives;
use woeusb_core::models::{Drive, FlashConfig, InstallMode, ProgressEvent, TargetFilesystem};

pub struct MainWindow {
    pub window: adw::ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("WoeUSB")
            .default_width(620)
            .default_height(580)
            .build();

        let header = adw::HeaderBar::new();

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);

        let content_box = GtkBox::new(Orientation::Vertical, 16);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(16);
        content_box.set_margin_bottom(24);

        // State
        let selected_source = Rc::new(RefCell::new(Option::<PathBuf>::None));
        let selected_drive = Rc::new(RefCell::new(Option::<Drive>::None));
        let drives_list = Rc::new(RefCell::new(Vec::<Drive>::new()));

        // Group 1: Source
        let source_group = adw::PreferencesGroup::builder()
            .title("1. Source Media")
            .description("Select a Windows ISO image or DVD disc")
            .build();

        let source_row = adw::ActionRow::builder()
            .title("Windows ISO Image")
            .subtitle("No file selected")
            .build();

        let choose_file_btn = Button::with_label("Choose ISO...");
        choose_file_btn.set_valign(Align::Center);

        let s_row = source_row.clone();
        let s_source = selected_source.clone();
        let win_clone = window.clone();

        choose_file_btn.connect_clicked(move |_| {
            let chooser = FileChooserNative::new(
                Some("Select Windows ISO"),
                Some(&win_clone),
                FileChooserAction::Open,
                Some("Open"),
                Some("Cancel"),
            );

            let filter = FileFilter::new();
            filter.add_pattern("*.iso");
            filter.add_pattern("*.ISO");
            filter.set_name(Some("Windows Disk Images (*.iso)"));
            chooser.add_filter(&filter);

            let s_row_c = s_row.clone();
            let s_src_c = s_source.clone();

            chooser.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            s_row_c.set_subtitle(&path.to_string_lossy());
                            *s_src_c.borrow_mut() = Some(path);
                        }
                    }
                }
            });

            chooser.show();
        });

        source_row.add_suffix(&choose_file_btn);
        source_group.add(&source_row);
        content_box.append(&source_group);

        // Group 2: Target
        let target_group = adw::PreferencesGroup::builder()
            .title("2. Target USB Storage Device")
            .description("Select the destination USB flash drive to install Windows on")
            .build();

        let target_row = adw::ActionRow::builder()
            .title("Target Drive")
            .build();

        let drive_string_list = StringList::new(&[]);
        let drive_dropdown = DropDown::new(Some(drive_string_list.clone()), gtk4::Expression::NONE);
        drive_dropdown.set_valign(Align::Center);

        let refresh_btn = Button::with_label("Refresh");
        refresh_btn.set_valign(Align::Center);

        // Function to populate drives
        let populate_drives = {
            let d_list = drives_list.clone();
            let d_str_list = drive_string_list;
            let s_drive = selected_drive.clone();

            move || {
                let drives = list_usb_drives(false).unwrap_or_default();
                d_str_list.splice(0, d_str_list.n_items(), &[]);

                if drives.is_empty() {
                    d_str_list.append("No USB drives detected");
                    *s_drive.borrow_mut() = None;
                } else {
                    for d in &drives {
                        d_str_list.append(&format!(
                            "{} - {} ({})",
                            d.path.display(),
                            d.model,
                            d.size_formatted
                        ));
                    }
                    *s_drive.borrow_mut() = Some(drives[0].clone());
                }
                *d_list.borrow_mut() = drives;
            }
        };

        populate_drives();

        let p_drives = populate_drives.clone();
        refresh_btn.connect_clicked(move |_| {
            p_drives();
        });

        let d_list_c = drives_list;
        let s_drive_c = selected_drive.clone();
        drive_dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            let list = d_list_c.borrow();
            if idx < list.len() {
                *s_drive_c.borrow_mut() = Some(list[idx].clone());
            }
        });

        target_row.add_suffix(&drive_dropdown);
        target_row.add_suffix(&refresh_btn);
        target_group.add(&target_row);
        content_box.append(&target_group);

        // Group 3: Options
        let options_group = adw::PreferencesGroup::builder()
            .title("3. Installation Options")
            .build();

        let fs_row = adw::ComboRow::builder()
            .title("Target Filesystem")
            .subtitle("FAT32 is recommended for UEFI; NTFS will be chosen if install.wim > 4GB")
            .model(&StringList::new(&["FAT32", "NTFS"]))
            .build();

        let boot_flag_row = adw::ActionRow::builder()
            .title("Workaround BIOS Boot Flag")
            .subtitle("For older motherboards that ignore drives with no active boot flag")
            .build();
        let boot_flag_switch = Switch::new();
        boot_flag_switch.set_valign(Align::Center);
        boot_flag_row.add_suffix(&boot_flag_switch);

        let skip_grub_row = adw::ActionRow::builder()
            .title("Skip Legacy GRUB Bootloader")
            .subtitle("Only bootable on UEFI systems; skips GRUB installation")
            .build();
        let skip_grub_switch = Switch::new();
        skip_grub_switch.set_valign(Align::Center);
        skip_grub_row.add_suffix(&skip_grub_switch);

        let dry_run_row = adw::ActionRow::builder()
            .title("Dry-Run Simulation")
            .subtitle("Simulate burning without writing to physical USB device")
            .build();
        let dry_run_switch = Switch::new();
        dry_run_switch.set_valign(Align::Center);
        dry_run_row.add_suffix(&dry_run_switch);

        options_group.add(&fs_row);
        options_group.add(&boot_flag_row);
        options_group.add(&skip_grub_row);
        options_group.add(&dry_run_row);
        content_box.append(&options_group);

        // Group 4: Progress & Action
        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(true);
        progress_bar.set_fraction(0.0);
        progress_bar.set_visible(false);

        let status_label = Label::new(Some("Ready to create bootable Windows USB."));
        status_label.set_wrap(true);

        let flash_btn = Button::with_label("Install Windows to USB");
        flash_btn.add_css_class("suggested-action");
        flash_btn.add_css_class("pill");
        flash_btn.set_height_request(44);

        content_box.append(&progress_bar);
        content_box.append(&status_label);
        content_box.append(&flash_btn);

        main_box.append(&content_box);
        window.set_content(Some(&main_box));

        // Flash action handler
        let pb = progress_bar;
        let lbl = status_label;
        let f_btn = flash_btn.clone();
        let win = window.clone();

        flash_btn.connect_clicked(move |_| {
            let src = selected_source.borrow().clone();
            let drv = selected_drive.borrow().clone();

            let source_path = match src {
                Some(p) => p,
                None => {
                    lbl.set_text("Please choose a Windows ISO image first.");
                    return;
                }
            };

            let target_path = match drv {
                Some(d) => d.path,
                None => {
                    lbl.set_text("No target USB storage device selected.");
                    return;
                }
            };

            let target_fs = if fs_row.selected() == 1 {
                TargetFilesystem::Ntfs
            } else {
                TargetFilesystem::Fat
            };

            let config = FlashConfig {
                source: source_path,
                target: target_path,
                install_mode: InstallMode::Device,
                target_filesystem: target_fs,
                filesystem_label: "Windows USB".to_string(),
                workaround_bios_boot_flag: boot_flag_switch.is_active(),
                skip_grub: skip_grub_switch.is_active(),
                dry_run: dry_run_switch.is_active(),
            };

            f_btn.set_sensitive(false);
            pb.set_visible(true);
            pb.set_fraction(0.0);
            lbl.set_text("Starting Windows installation process...");

            let (tx, rx) = mpsc::channel::<ProgressEvent>();

            let pb_clone = pb.clone();
            let lbl_clone = lbl.clone();
            let f_btn_clone = f_btn.clone();
            let win_clone = win.clone();

            glib::timeout_add_local(Duration::from_millis(50), move || {
                while let Ok(event) = rx.try_recv() {
                    match event {
                        ProgressEvent::StepStarted { step, total_steps, message } => {
                            pb_clone.set_fraction((step as f64 - 1.0) / total_steps as f64);
                            lbl_clone.set_text(&format!("Step {}/{}: {}", step, total_steps, message));
                        }
                        ProgressEvent::ProgressUpdated { percentage, speed_bytes_per_sec, current_file, .. } => {
                            pb_clone.set_fraction(percentage / 100.0);
                            let speed_mb = speed_bytes_per_sec as f64 / (1024.0 * 1024.0);
                            lbl_clone.set_text(&format!(
                                "Copying: {} ({:.1}%, {:.1} MB/s)",
                                current_file, percentage, speed_mb
                            ));
                        }
                        ProgressEvent::StepFinished { message, .. } => {
                            lbl_clone.set_text(&message);
                        }
                        ProgressEvent::LogMessage { message, .. } => {
                            lbl_clone.set_text(&message);
                        }
                        ProgressEvent::Completed { message } => {
                            pb_clone.set_fraction(1.0);
                            lbl_clone.set_text(&format!("✓ {}", message));
                            f_btn_clone.set_sensitive(true);

                            let dialog = adw::MessageDialog::builder()
                                .transient_for(&win_clone)
                                .heading("Installation Complete")
                                .body("Your bootable Windows USB storage device has been created successfully!")
                                .build();
                            dialog.add_response("ok", "OK");
                            dialog.present();
                            return glib::ControlFlow::Break;
                        }
                        ProgressEvent::Error { message } => {
                            pb_clone.set_fraction(0.0);
                            lbl_clone.set_text(&format!("✗ Error: {}", message));
                            f_btn_clone.set_sensitive(true);

                            let dialog = adw::MessageDialog::builder()
                                .transient_for(&win_clone)
                                .heading("Installation Failed")
                                .body(&message)
                                .build();
                            dialog.add_response("ok", "Close");
                            dialog.present();
                            return glib::ControlFlow::Break;
                        }
                    }
                }
                glib::ControlFlow::Continue
            });

            WorkerClient::spawn_worker(config, tx);
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
