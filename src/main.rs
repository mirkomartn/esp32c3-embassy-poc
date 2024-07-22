#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_net::{tcp::TcpSocket, Config, Ipv4Address, Stack, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
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
use heapless::String;

mod boot_btn;
mod mqtt;
mod netstack;
mod tsens;
mod wifi;

#[derive(Debug, Clone)]
enum SigSource {
    BootBtn,
    Timer,
    MQTT,
    None,
}

static SIGNAL: Signal<CriticalSectionRawMutex, SigSource> = Signal::new();

#[main]
async fn main(spawner: Spawner) {
    // General setup/default configuration of the board
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);

    esp_hal_embassy::init(&clocks, timg0);
    let _ = esp_hal::gpio::Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // setup boot button handler
    boot_btn::start(&spawner, || SIGNAL.signal(SigSource::BootBtn));

    // setup timer that will raise signal every 5 seconds
    spawner
        .spawn(timer_trigger(5, || SIGNAL.signal(SigSource::Timer)))
        .ok();

    // poll two futures concurrently
    let (tsens, mut wifi_link) = join(
        tsens::Tsens::new(),
        wifi::WifiLink::new(
            &spawner,
            peripherals.SYSTIMER,
            peripherals.RNG,
            peripherals.RADIO_CLK,
            &clocks,
            peripherals.WIFI,
        ),
    )
    .await;

    let netstack = netstack::NetStack::new(&spawner, wifi_link.take().unwrap()).await;

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut socket = netstack.get_tcp_socket(&mut rx_buffer, &mut tx_buffer);

    if let Some(_) = netstack
        .connect_socket(&mut socket, "broker.hivemq.com", 1883)
        .await
    {
        let mut rbuf = [0; 512];
        let mut wbuf = [0; 512];
        let rlen = rbuf.len();
        let wlen = wbuf.len();

        let mut mqtt = mqtt::MqttConnection::new(socket, &mut rbuf, rlen, &mut wbuf, wlen).await;

        mqtt.subscribe("get-temp/1").await;

        let mut msg: String<32> = String::new();

        loop {
            let mut src = SigSource::None;

            SIGNAL.reset();

            match select(SIGNAL.wait(), mqtt.recv_msg()).await {
                Either::First(s) => {
                    src = s;
                }
                Either::Second(Some(m)) => {
                    if m.0 == "get-temp/1" && m.1 == b"it's me" {
                        src = SigSource::MQTT;
                    }
                }
                _ => {}
            }

            match src {
                SigSource::BootBtn => {
                    write!(msg, "BootBtn: {:.2}", tsens.get_temp()).ok();
                }
                SigSource::Timer => {
                    write!(msg, "Timer: {:.2}", tsens.get_temp()).ok();
                }
                SigSource::MQTT => {
                    write!(msg, "MQTT: {:.2}", tsens.get_temp()).ok();
                }
                _ => {
                    continue;
                }
            }

            mqtt.send_temp(&msg).await;
            msg.clear();
        }
    }
}

#[embassy_executor::task]
async fn timer_trigger(sec: u64, cb: impl Fn() + 'static) {
    loop {
        Timer::after(Duration::from_secs(sec)).await;
        cb();
    }
}
