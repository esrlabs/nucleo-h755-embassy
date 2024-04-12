#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::{
    gpio::{Level, Output, Speed},
    Config,
};

use {defmt_rtt as _, embassy_stm32 as hal, embassy_stm32::pac, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //info!("Core1: STM32H755 Embassy HSEM Test.");

    let p = embassy_stm32::init(Config::default());
    //info!("Config set");

    // Enable the Cortex-M4 ART Clock
    pac::RCC.ahb1enr().modify(|w| w.set_arten(true));

    let mut led_yellow = Output::new(p.PE1, Level::Low, Speed::Low);
    // let mut led_green = Output::new(p.PB0, Level::Low, Speed::Low);
    // let mut led_red = Output::new(p.PB14, Level::Low, Speed::Low);

    // let cpu = embassy_stm32::hsem::get_current_coreid();
    // if cpu == embassy_stm32::hsem::CoreId::Core1 {
    //     led_green.set_high();
    // } else {
    //     led_red.set_high();
    // }
    loop {
        led_yellow.set_high();
        Timer::after_millis(500).await;
        led_yellow.set_low();
        Timer::after_millis(500).await;
    }
}
