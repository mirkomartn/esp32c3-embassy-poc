use crate::*;

use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use static_cell::StaticCell;

pub struct NetStack {
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
}

impl NetStack {
    pub async fn new(spawner: &Spawner, wifi_iface: WifiDevice<'static, WifiStaDevice>) -> Self {
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
            wifi_iface,
            config,
            RESOURCES.init(StackResources::new()),
            seed,
        ));

        // `net_task` runs a network-loop, processing all networking
        // related-events. It's diverging and never returns.
        spawner.spawn(net_task(&stack)).ok();

        // wait for the stack to obtain a valid IP configuration
        // TODO: wrap this into select! together with a timeout
        // and handle failure
        stack.wait_config_up().await;

        Self { stack }
    }
}

// This is a diverging function, so it must be run on a separate task in
// the background. It runs the network stack, processing network events.
#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
