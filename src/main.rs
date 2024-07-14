#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Config, Ipv4Address, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_println::println;

mod boot_btn;
mod tsens;
mod wifi;

// recast a reference to T to a reference to static T
#[inline]
unsafe fn make_static<T>(t: &T) -> &'static T {
    core::mem::transmute(t)
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.init(($val)); // uninit().write(($val));
        x
    }};
}

// make mk_static! macro available to all modules of the crate
pub(crate) use mk_static;

#[main]
async fn main(spawner: Spawner) {
    // General setup/default configuration of the board
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);

    esp_hal_embassy::init(&clocks, timg0);

    // setup boot button handler
    // TODO: pass a Fn to be called on event
    boot_btn::start(&spawner);

    let tsens = tsens::new();
    let _wifi_link = wifi::WifiLink::new(
        &spawner,
        peripherals.SYSTIMER,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        unsafe { make_static(&clocks) },
        peripherals.WIFI,
    )
    .await;

    loop {
        println!("Temperature == {}", tsens.get_temp());
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
