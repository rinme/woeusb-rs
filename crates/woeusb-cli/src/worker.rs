use anyhow::Result;
use std::io::Write;
use woeusb_core::engine::Engine;
use woeusb_core::models::{FlashConfig, ProgressEvent};

pub fn run_worker(config: FlashConfig) -> Result<()> {
    let engine = Engine::new(config);

    let res = engine.run(|event| {
        if let Ok(json) = serde_json::to_string(&event) {
            println!("{}", json);
            let _ = std::io::stdout().flush();
        }
    });

    if let Err(e) = res {
        let err_event = ProgressEvent::Error {
            message: e.to_string(),
        };
        if let Ok(json) = serde_json::to_string(&err_event) {
            println!("{}", json);
            let _ = std::io::stdout().flush();
        }
        std::process::exit(1);
    }

    Ok(())
}
