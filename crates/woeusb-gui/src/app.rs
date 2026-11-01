use crate::window::MainWindow;
use adw::prelude::*;

pub const APP_ID: &str = "com.github.woeusb.woeusbgui";

pub struct App {
    app: adw::Application,
}

impl App {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id(APP_ID)
            .build();

        app.connect_activate(|app| {
            let window = MainWindow::new(app);
            window.present();
        });

        Self { app }
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
}
