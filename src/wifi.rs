use crate::*;

use esp_wifi::{
    initialize,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitFor,
};

const SSID: &str = core::env!("SSID");
const PASSWORD: &str = core::env!("PASSWORD");

pub struct WifiLink {
    interface: Option<WifiDevice<'static, WifiStaDevice>>,
}

// TODO: use a typestate pattern
impl WifiLink {
    pub async fn new(
        spawner: &Spawner,
        systimer: esp_hal::peripherals::SYSTIMER,
        rng: esp_hal::peripherals::RNG,
        rclock: esp_hal::peripherals::RADIO_CLK,
        clocks: &esp_hal::clock::Clocks<'_>,
        wifi: esp_hal::peripherals::WIFI,
    ) -> Self {
        let init = initialize(
            EspWifiInitFor::Wifi,
            SystemTimer::new(systimer).alarm0,
            Rng::new(rng),
            rclock,
            clocks,
        )
        .expect("Failed to initialize Wifi");

        let (interface, controller) =
            esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

        // `connection` establishes an L2 link and upon link break-down
        // waits a bit and then tries to re-establish it
        spawner.spawn(connection(controller)).ok();

        Self {
            interface: Some(interface),
        }
    }

    pub fn take(&mut self) -> Option<WifiDevice<'static, WifiStaDevice>> {
        self.interface.take()
    }
}

// Establish a persistent L2 connection. Upon failure/break-down, wait and retry.
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected and attempt to reconnect after 5 sec
                // as long as we're connected, this task will be parked at this await point
                // and will not consume any resources
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller.start().await.unwrap();
        }

        match controller.connect().await {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}
