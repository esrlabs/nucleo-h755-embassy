#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::{
    gpio::{Level, Output, Speed},
    interrupt, Config,
};

use {
    defmt_rtt as _, embassy_stm32 as hal, embassy_stm32::pac, panic_probe as _,
    stm32h7hal_ext as hal_ext,
};

// This function handles HSEM interrupt request
#[interrupt]
fn HSEM2() {
    //let statusreg = pac::HSEM.misr(1).read();
    // FIXME: the semaphore ID is hardcoded
    pac::HSEM.ier(1).write(|w| w.set_ise(0, false));
    pac::HSEM.icr(1).write(|w| w.set_isc(0, true));
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    //info!("Core1: STM32H755 Embassy HSEM Test.");

    hal_ext::enable_hsem_clock();

    hal_ext::hsem_activate_notification(0);

    hal_ext::clear_pending_events();

    unsafe { NVIC::unmask(pac::Interrupt::HSEM2) };

    hal_ext::enter_stop_mode(
        hal_ext::PwrRegulator::MainRegulator,
        hal_ext::StopMode::StopEntryWfe,
        hal_ext::PwrDomain::D2,
    );

    let systick = unsafe { cortex_m::Peripherals::steal().SYST };
    let mut delay = cortex_m::delay::Delay::new(systick, 200_000_000);
    //delay.delay_ms(5000);
    let p = embassy_stm32::init_core1();

    //info!("Config set");

    // Enable the Cortex-M4 ART Clock
    // pac::RCC.ahb1enr().modify(|w| w.set_arten(true));

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
        //Timer::after_millis(500).await;
        delay.delay_ms(250);
        led_yellow.set_low();
        //Timer::after_millis(500).await;
        delay.delay_ms(250);
    }
}
