mod ethernet;

use anyhow::Result;
use bme280_rs::{Bme280, Configuration, Oversampling, SensorMode};
use esp_idf_hal::prelude::*;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{GpioPin, InputOutput},
    i2c::{config::MasterConfig, Master, MasterPins, I2C0},
};
use esp_idf_svc::{netif::EspNetifStack, nvs::EspDefaultNvs, sysloop::EspSysLoopStack};
use log::{error, info};
use std::sync::Arc;

pub struct Hardware {
    _netif_stack: Arc<EspNetifStack>,
    _sys_loop_stack: Arc<EspSysLoopStack>,
    _default_nvs: Arc<EspDefaultNvs>,

    _eth: ethernet::Ethernet,
    bme280: Bme280<Master<I2C0, GpioPin<InputOutput>, GpioPin<InputOutput>>, FreeRtos>,
}

impl Hardware {
    pub fn setup() -> Result<Self> {
        let peripherals = esp_idf_hal::prelude::Peripherals::take()
            .ok_or_else(|| anyhow::format_err!("Could not get Peripherals"))?;
        let pins = peripherals.pins;

        let netif_stack = Arc::new(EspNetifStack::new()?);
        let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
        let default_nvs = Arc::new(EspDefaultNvs::new()?);

        let i2c = peripherals.i2c0;
        let sda = pins.gpio13.into_input_output()?.degrade();
        let scl = pins.gpio16.into_input_output()?.degrade();
        let config = MasterConfig::new().baudrate(100.kHz().into());
        let i2c = Master::new(i2c, MasterPins { sda, scl }, config)?;
        let mut bme280 = Bme280::new(i2c, esp_idf_hal::delay::FreeRtos);
        bme280.init()?;
        bme280.set_sampling_configuration(
            Configuration::default()
                .with_temperature_oversampling(Oversampling::Oversample1)
                .with_sensor_mode(SensorMode::Normal),
        )?;

        let eth = ethernet::Ethernet::setup(
            netif_stack.clone(),
            sys_loop_stack.clone(),
            ethernet::EthernetPins {
                phy_pwr: pins.gpio12.into_output()?.degrade(),
                rmii_ref_clk: pins.gpio17,
                rmii_mdio: pins.gpio18.into_input_output()?.degrade(),
                rmii_txd0: pins.gpio19,
                rmii_tx_en: pins.gpio21,
                rmii_txd1: pins.gpio22,
                rmii_mdc: pins.gpio23.into_input_output()?.degrade(),
                rmii_rdx0: pins.gpio25,
                rmii_rdx1: pins.gpio26,
                rmii_crs_dv: pins.gpio27,
            },
        )
        .map_err(|err| {
            error!("Could not setup Ethernet: {err:?}");
            err
        })?;

        info!("Hardware successfully setup!");

        Ok(Self {
            _netif_stack: netif_stack,
            _sys_loop_stack: sys_loop_stack,
            _default_nvs: default_nvs,

            _eth: eth,
            bme280,
        })
    }

    pub fn read_temperature(&mut self) -> Result<u8> {
        Ok(self
            .bme280
            .read_temperature()?
            .ok_or_else(|| anyhow::format_err!("Could not read temperature"))?
            .round() as u8)
    }
}
