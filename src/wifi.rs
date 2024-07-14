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
    stack: Option<&'static Stack<WifiDevice<'static, WifiStaDevice>>>,
}

// TODO: use a typestate pattern
impl WifiLink {
    pub async fn new(
        spawner: &Spawner,
        systimer: esp_hal::peripherals::SYSTIMER,
        rng: esp_hal::peripherals::RNG,
        rclock: esp_hal::peripherals::RADIO_CLK,
        clocks: &'static esp_hal::clock::Clocks<'static>,
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

        let (wifi_interface, controller) =
            esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

        let config = Config::dhcpv4(Default::default());
        let seed = 8888; // TODO

        let stack = &*mk_static!(
            Stack<WifiDevice<'_, WifiStaDevice>>,
            Stack::new(
                wifi_interface,
                config,
                mk_static!(StackResources<3>, StackResources::<3>::new()),
                seed
            )
        );

        spawner.spawn(connection(controller)).ok();
        spawner.spawn(net_task(&stack)).ok();

        //wait until wifi is connected
        while !stack.is_link_up() {
            Timer::after(Duration::from_millis(500)).await;
        }

        loop {
            if let Some(config) = stack.config_v4() {
                println!("Got IP: {}", config.address); //dhcp IP address
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        Self { stack: Some(stack) }
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    // println!("Device capabilities: {:?}", controller.get_capabilities());
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected and attempt to reconnect after 5 sec
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
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
