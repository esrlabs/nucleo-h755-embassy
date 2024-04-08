#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    hsem::HardwareSemaphore,
    Config,
};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("STM32H755 Embassy HSEM Test.");
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        // Set SMPS power config otherwise MCU will not powered after next power-off
        config.rcc.supply_config = SupplyConfig::DirectSMPS;
    }

    let p = embassy_stm32::init(config);
    info!("Config set");
    //enable HSEM clock
    embassy_stm32::pac::RCC
        .ahb4enr()
        .modify(|w| w.set_hsemen(true));
    info!("HSEM clock enabled");

    let mut led_green = Output::new(p.PB0, Level::Low, Speed::Low);
    let mut led_yellow = Output::new(p.PE1, Level::Low, Speed::Low);
    let mut led_red = Output::new(p.PB14, Level::Low, Speed::Low);

    let mut hsem = HardwareSemaphore::new(p.HSEM);

    loop {
        loop {
            if let Err(_err) = hsem.two_step_lock(0, 0) {
                info!("Error taking semaphore for process 0");
                Timer::after_millis(1000).await;
            } else {
                info!("Semaphore taken for process 0");
            }

            led_green.set_high();
            Timer::after_millis(500).await;

            led_green.set_low();

            led_yellow.set_high();
            Timer::after_millis(500).await;

            led_yellow.set_low();

            led_red.set_high();
            Timer::after_millis(500).await;
            led_red.set_low();
        }
    }
}
