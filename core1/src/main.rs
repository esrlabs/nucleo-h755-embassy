#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]
use core::cell::SyncUnsafeCell;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    hsem::{HardwareSemaphore, InterruptHandler},
    peripherals::{self, HSEM},
};
use {
    defmt_rtt as _, embassy_stm32 as hal, panic_probe as _, shared as _, stm32h7hal_ext as hal_ext,
};

bind_interrupts!(
    struct Irqs {
        HSEM2 => InterruptHandler<peripherals::HSEM>;
    }
);

// SAFETY: This is safe because all access to the HSEM registers are atomic
static HSEM_INSTANCE: SyncUnsafeCell<Option<HardwareSemaphore<'static, HSEM>>> =
    SyncUnsafeCell::new(None);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core1: STM32H755 Embassy HSEM Test.");

    hal_ext::core1_startup();

    let p = embassy_stm32::init_core1(200_000_000);

    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);
    // initialize global HSEM instance
    unsafe { *HSEM_INSTANCE.get() = Some(hsem) };

    // Enable the Cortex-M4 ART Clock
    embassy_stm32::pac::RCC
        .ahb1enr()
        .modify(|w| w.set_arten(true));

    let mut led_yellow = Output::new(p.PE1, Level::Low, Speed::Low);
    let _ = get_global_hsem().one_step_lock(1);
    let _ = get_global_hsem().one_step_lock(2);

    led_yellow.set_high();
    Timer::after_millis(2000).await;
    get_global_hsem().unlock(1, 0);
    led_yellow.set_low();

    Timer::after_millis(1000).await;

    led_yellow.set_high();
    Timer::after_millis(2000).await;
    get_global_hsem().unlock(2, 0);
    led_yellow.set_low();
    Timer::after_millis(2000).await;
    loop {
        let _ = get_core1_blink_delay().await;
        let delay_time = get_core1_blink_delay().await;
        led_yellow.set_high();
        Timer::after_millis(delay_time as u64).await;

        led_yellow.set_low();
        Timer::after_millis(delay_time as u64).await;
    }
}

async fn get_core1_blink_delay() -> u32 {
    let mut retry = 10;
    let mut delay = 250;
    while !get_global_hsem().lock(5).await && retry > 0 {
        Timer::after_micros(50).await;
        retry -= 1;
    }
    if retry > 0 {
        unsafe {
            delay = shared::MAILBOX[0];
        }
        get_global_hsem().unlock(5, 0);
    } else {
        // Core1 has asquired the semaphore and is
        // not releasing it - crashed?
        defmt::panic!("Failed to asquire semaphore 1");
    }
    delay
}

fn get_global_hsem() -> &'static mut HardwareSemaphore<'static, HSEM> {
    unsafe {
        match *HSEM_INSTANCE.get() {
            Some(ref mut obj) => obj,
            None => defmt::panic!("HardwareSemaphore was not initialized"),
        }
    }
}
