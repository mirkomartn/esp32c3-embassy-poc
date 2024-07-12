use esp_hal::gpio::AnyInput;

pub struct Button {
    button: AnyInput<'static>,
}

impl Button {
    pub fn new(button: AnyInput<'static>) -> Self {
        Self { button }
    }

    //
    pub async fn wait(&mut self) {
        self.button.wait_for_rising_edge().await;
    }
}
