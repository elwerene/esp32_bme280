use anyhow::Result;
use embedded_svc::sys_time::SystemTime;
use esp_idf_svc::{
    sntp::{EspSntp, SyncStatus},
    systime::EspSystemTime,
};
use esp_idf_sys as _;
use std::time::{Duration, Instant};

// Blocks while waiting for sntp sync
pub fn sync(timeout: Duration) -> Result<EspSntp> {
    let start_time = EspSystemTime.now();
    let sntp = EspSntp::new_default()?;

    loop {
        let sync_status = sntp.get_sync_status();
        if sync_status == SyncStatus::Completed {
            break;
        }

        let elapsed = EspSystemTime.now() - start_time;
        if elapsed > timeout {
            anyhow::bail!("Timeout getting ntp time");
        }

        log::info!("Status: {sync_status:?}. Waiting another second");
        std::thread::sleep(Duration::from_secs(1));
    }

    log::info!("Got current time: {:?}", Instant::now());

    Ok(sntp)
}
