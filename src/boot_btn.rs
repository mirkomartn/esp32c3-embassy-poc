use crate::*;
use esp_hal::{gpio::AnyInput, peripheral::Peripheral};

pub fn start(spawner: &Spawner, cb: impl Fn() + 'static) {
    let but = unsafe { AnyInput::new(esp_hal::gpio::Gpio9::steal(), esp_hal::gpio::Pull::Up) };
    spawner.spawn(button_press(but, cb)).ok();
}

#[embassy_executor::task]
async fn button_press(mut button: AnyInput<'static>, cb: impl Fn() + 'static) {
    loop {
        button.wait_for_rising_edge().await;
        cb();
    }
}
