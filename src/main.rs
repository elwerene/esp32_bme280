#![forbid(clippy::unwrap_used)]

mod fs;
mod hardware;
mod httpd;
mod ntp;
mod sessions;

use anyhow::Result;
use std::time::Duration;

const READ_TEMPERATURE_WAIT: Duration = Duration::from_secs(10);

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(err) = start() {
        panic!("Error: {err:?}");
    }
}

fn start() -> Result<()> {
    fs::init()?;
    let mut hardware = hardware::Hardware::setup()?;
    ntp::sync(Duration::from_secs(10))?;
    sessions::init()?;
    httpd::Httpd::setup()?;

    log::info!("Starting main loop");
    loop {
        let temperature = hardware.read_temperature()?;
        sessions::add_temp(temperature)?;

        std::thread::sleep(READ_TEMPERATURE_WAIT);
    }
}
