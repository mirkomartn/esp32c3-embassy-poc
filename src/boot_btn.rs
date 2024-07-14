use crate::*;
use esp_hal::{gpio::AnyInput, peripheral::Peripheral};

pub fn start(spawner: &Spawner) {
    let but = unsafe { AnyInput::new(esp_hal::gpio::Gpio9::steal(), esp_hal::gpio::Pull::Up) };
    spawner.spawn(button_press(but)).ok();
}

#[embassy_executor::task]
async fn button_press(mut button: AnyInput<'static>) {
    loop {
        button.wait_for_rising_edge().await;
        println!("Zdravo Gasper!");
    }
}
