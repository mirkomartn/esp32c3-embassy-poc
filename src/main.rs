#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::{AnyInput, Io, Pull},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_println::println;

mod button;
mod tsens;

#[main]
async fn main(spawner: Spawner) {
    // General setup/default configuration of the board
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0);

    // Use GPIO9 (BOOT button) for user input
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let but = AnyInput::new(io.pins.gpio9, Pull::Up);
    let but = button::Button::new(but);

    // Periodically measure temperature
    spawner.spawn(tsens()).ok();
    // Wait for user to press the button
    spawner.spawn(button_press(but)).ok();

    // Main is an embassy task as well, might as well use it
    loop {
        println!("Hello World");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

#[embassy_executor::task]
async fn tsens() {
    // Create new Tsens struct which will initialize the sensor
    let tsens = tsens::Tsens::new();
    // Recommended time for the sensor to settle (as per C idf source)
    Timer::after(Duration::from_micros(300)).await;

    loop {
        println!("Temp: {}", tsens.get_temp());
        Timer::after(Duration::from_millis(2_000)).await;
    }
}

#[embassy_executor::task]
async fn button_press(mut button: button::Button) {
    loop {
        button.wait().await;
        println!("Zdravo Gasper!");
    }
}
