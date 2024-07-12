pub struct Tsens {
    tsens_reg: esp32c3::APB_SARADC,
    sys_reg: esp32c3::SYSTEM,
}

impl Tsens {
    pub fn new() -> Self {
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

        tsens
    }

    pub fn get_temp(&self) -> u8 {
        // get the reading of the sensor
        self.tsens_reg.tsens_ctrl().read().out().bits()
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
