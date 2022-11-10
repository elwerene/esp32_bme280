use anyhow::Result;
use embedded_hal::digital::v2::OutputPin;
use embedded_svc::{
    eth::{self, Eth, TransitionalState},
    ipv4::ClientConfiguration,
    sys_time::SystemTime,
};
use esp_idf_hal::gpio::{
    Gpio17, Gpio19, Gpio21, Gpio22, Gpio25, Gpio26, Gpio27, GpioPin, InputOutput, Output, Unknown,
};
use esp_idf_svc::{
    eth::{EspEth, RmiiEthChipset, RmiiEthPeripherals},
    netif::EspNetifStack,
    sysloop::EspSysLoopStack,
    systime::EspSystemTime,
};
use log::info;
use std::{sync::Arc, time::Duration};

type Phy = RmiiEthPeripherals<GpioPin<InputOutput>, GpioPin<InputOutput>, GpioPin<Output>>;

pub struct EthernetPins {
    pub phy_pwr: GpioPin<Output>,
    pub rmii_ref_clk: Gpio17<Unknown>,
    pub rmii_txd0: Gpio19<Unknown>,
    pub rmii_tx_en: Gpio21<Unknown>,
    pub rmii_txd1: Gpio22<Unknown>,
    pub rmii_rdx0: Gpio25<Unknown>,
    pub rmii_rdx1: Gpio26<Unknown>,
    pub rmii_crs_dv: Gpio27<Unknown>,
    pub rmii_mdio: GpioPin<InputOutput>,
    pub rmii_mdc: GpioPin<InputOutput>,
}

pub struct Ethernet(Box<EspEth<Phy>>);

impl Ethernet {
    pub fn setup(
        netif_stack: Arc<EspNetifStack>,
        sys_loop_stack: Arc<EspSysLoopStack>,
        mut pins: EthernetPins,
    ) -> Result<Self> {
        pins.phy_pwr.set_high()?;
        let eth = Box::new(EspEth::new_rmii(
            netif_stack,
            sys_loop_stack,
            RmiiEthPeripherals {
                rmii_rdx0: pins.rmii_rdx0,
                rmii_rdx1: pins.rmii_rdx1,
                rmii_crs_dv: pins.rmii_crs_dv,
                rmii_mdc: pins.rmii_mdc,
                rmii_txd1: pins.rmii_txd1,
                rmii_tx_en: pins.rmii_tx_en,
                rmii_txd0: pins.rmii_txd0,
                rmii_mdio: pins.rmii_mdio,
                rmii_ref_clk_config: esp_idf_svc::eth::RmiiClockConfig::OutputInvertedGpio17(
                    pins.rmii_ref_clk,
                ),
                rst: None,
            },
            RmiiEthChipset::LAN87XX,
            Some(0),
        )?);

        let mut eth = Self(eth);

        eth.configure()?;

        Ok(eth)
    }

    fn configure(&mut self) -> Result<()> {
        let mut hostname = heapless::String::<30>::new();
        hostname
            .push_str("Sauna-ESP32")
            .map_err(|_| anyhow::format_err!("Could not create hostname"))?;

        self.0
            .set_configuration(&eth::Configuration::Client(ClientConfiguration::DHCP(
                embedded_svc::ipv4::DHCPClientSettings {
                    hostname: Some(hostname),
                },
            )))?;

        info!("Eth configuration set, about to get status");

        self.0
            .wait_status_with_timeout(EspSystemTime.now() + Duration::from_secs(10), |status| {
                !status.is_transitional()
            })
            .map_err(|e| anyhow::anyhow!("Unexpected Eth status: {:?}", e))?;

        let status = self.0.get_status();

        if let eth::Status::Started(eth::ConnectionStatus::Connected(eth::IpStatus::Done(Some(
            _ip_settings,
        )))) = status
        {
            info!("Eth connected");

            Ok(())
        } else {
            Err(anyhow::format_err!("Could not connect ethernet"))
        }
    }
}
