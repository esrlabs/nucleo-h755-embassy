#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    Config,
};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core1: STM32H755 Embassy HSEM Test.");

    let p = embassy_stm32::init(Config::default());
    info!("Config set");

    let mut led_yellow = Output::new(p.PE1, Level::Low, Speed::Low);

    loop {
        led_yellow.set_high();
        Timer::after_millis(500).await;
        led_yellow.set_low();
        Timer::after_millis(500).await;
    }
}
