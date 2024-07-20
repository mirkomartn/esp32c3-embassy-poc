use crate::*;

use esp_wifi::{
    initialize,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitFor,
};
use static_cell::StaticCell;

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

        let (wifi_interface, controller) =
            esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

        let config = Config::dhcpv4(Default::default());
        // the seed doesn't need to be cryptographically secure, it's used for
        // randomization of TCP port/initial sequence number, which helps prevent
        // collisions between sessions across *reboots*, which are quite unlikely
        // even if we're using a constant for a seed
        let seed = 8888;

        static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
        static STACK: StaticCell<Stack<WifiDevice<'_, WifiStaDevice>>> = StaticCell::new();
        // StaticCell::init() will return a &'static mut, which we then recast to &'static, because we only need a runtime initialization of a static variable, but don't require a mutable reference (in fact this might be problematic with a borrow checker)
        let stack = &*STACK.init(Stack::new(
            wifi_interface,
            config,
            RESOURCES.init(StackResources::new()),
            seed,
        ));

        // Both tasks execute indefinitely, so they need to run in
        // background tasks. First one (`connection`) establishes
        // an L2 link and upon link break-down waits a bit and then
        // tries to re-establish it. Second one (`net_task`) runs
        // a network-loop, processing all networking related-events.
        spawner.spawn(connection(controller)).ok();
        spawner.spawn(net_task(&stack)).ok();

        // wait for the stack to obtain a valid IP configuration
        // TODO: wrap this into select! together with a timeout
        // and handle failure
        stack.wait_config_up().await;

        Self { stack: Some(stack) }
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

// This is a diverging function, so it must be run on a separate task in
// the background. It runs the network stack, processing network events.
#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
