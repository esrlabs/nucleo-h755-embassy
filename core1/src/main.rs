#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    hsem::{HardwareSemaphore, InterruptHandler},
    peripherals,
};

use {defmt_rtt as _, embassy_stm32 as hal, panic_probe as _, stm32h7hal_ext as hal_ext};

bind_interrupts!(
    struct Irqs {
        HSEM2 => InterruptHandler<peripherals::HSEM>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core1: STM32H755 Embassy HSEM Test.");

    hal_ext::core1_startup();

    let p = embassy_stm32::init_core1(200_000_000);

    let mut hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    // Enable the Cortex-M4 ART Clock
    embassy_stm32::pac::RCC
        .ahb1enr()
        .modify(|w| w.set_arten(true));

    let mut led_yellow = Output::new(p.PE1, Level::Low, Speed::Low);
    let _ = hsem.one_step_lock(1);
    let _ = hsem.one_step_lock(2);

    led_yellow.set_high();
    Timer::after_millis(2000).await;
    hsem.unlock(1, 0);
    led_yellow.set_low();

    Timer::after_millis(1000).await;

    led_yellow.set_high();
    Timer::after_millis(2000).await;
    hsem.unlock(2, 0);
    led_yellow.set_low();
    Timer::after_millis(2000).await;
    loop {
        led_yellow.set_high();
        Timer::after_millis(250).await;

        led_yellow.set_low();
        Timer::after_millis(250).await;
    }
}
