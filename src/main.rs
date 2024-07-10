#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, peripherals::Peripherals, prelude::*, system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println::println;

struct Tsens {
    tsens_reg: esp32c3::APB_SARADC,
    sys_reg: esp32c3::SYSTEM,
}

impl Tsens {
    fn new() -> Self {
        let tsens = Self {
            tsens_reg: unsafe { esp32c3::APB_SARADC::steal() },
            sys_reg: unsafe { esp32c3::SYSTEM::steal() },
        };
        tsens.tsens_reg.tsens_ctrl().write(|w| w.pu().set_bit());
        tsens
            .sys_reg
            .perip_clk_en1()
            .write(|w| w.tsens_clk_en().set_bit());
        // Select XTAL_CLK (default as per C idf source)
        tsens
            .tsens_reg
            .tsens_ctrl2()
            .write(|w| w.clk_sel().set_bit());
        tsens
    }
}

impl Drop for Tsens {
    fn drop(&mut self) {
        self.tsens_reg.tsens_ctrl().write(|w| w.pu().clear_bit());
        self.sys_reg
            .perip_clk_en1()
            .write(|w| w.tsens_clk_en().clear_bit());
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    spawner.spawn(tsens()).ok();

    loop {
        println!("Hello World");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[embassy_executor::task]
async fn tsens() {
    let tsens = Tsens::new();
    Timer::after(Duration::from_micros(300)).await;

    loop {
        println!("Temp: {}", tsens.tsens_reg.tsens_ctrl().read().out().bits());
        Timer::after(Duration::from_millis(2_000)).await;
    }
}
