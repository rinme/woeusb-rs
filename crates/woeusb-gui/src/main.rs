mod app;
mod window;
mod worker_client;

use app::App;

fn main() -> glib::ExitCode {
    let app = App::new();
    app.run()
}
