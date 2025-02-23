use crate::*;

pub struct Tsens {
    tsens_reg: esp32c3::APB_SARADC,
    sys_reg: esp32c3::SYSTEM,
}

impl Tsens {
    pub async fn new() -> Self {
        let tsens = Self {
            tsens_reg: unsafe { esp32c3::APB_SARADC::steal() },
            sys_reg: unsafe { esp32c3::SYSTEM::steal() },
        };

        // power on the sensor
        tsens.tsens_reg.tsens_ctrl().write(|w| w.pu().set_bit());

        // Select XTAL_CLK (default as per C idf source)
        tsens
            .tsens_reg
            .tsens_ctrl2()
            .write(|w| w.clk_sel().set_bit());

        // enable sensor clock
        tsens
            .sys_reg
            .perip_clk_en1()
            .write(|w| w.tsens_clk_en().set_bit());

        // As per esp-idf implementation it's recommended to wait for
        // 300ms for the sensor to settle, before reading it
        Timer::after_millis(300).await;

        tsens
    }

    pub fn get_temp(&self) -> f32 {
        // get the reading of the sensor
        let t = self.tsens_reg.tsens_ctrl().read().out().bits();
        ((t as f32) * 0.4386) - 20.52
    }
}

impl Drop for Tsens {
    fn drop(&mut self) {
        // power off the sensor and disable the clock
        self.tsens_reg.tsens_ctrl().write(|w| w.pu().clear_bit());
        self.sys_reg
            .perip_clk_en1()
            .write(|w| w.tsens_clk_en().clear_bit());
    }
}
