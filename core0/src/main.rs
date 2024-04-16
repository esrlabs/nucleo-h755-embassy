#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::pac;
use embassy_time::Timer;
use hal::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    hsem::{CoreId, HardwareSemaphore, InterruptHandler},
    peripherals, Config,
};

use {defmt_rtt as _, embassy_stm32 as hal, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        HSEM2 => InterruptHandler<peripherals::HSEM>;
    }
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core0: STM32H755 Embassy HSEM Test.");

    // Wait for Core1 to be finished with its init
    // tasks and in Stop mode
    let mut timeout = 0xFFFF;
    while pac::RCC.cr().read().d2ckrdy() == true && timeout > 0 {
        timeout -= 1;
        // cortex_m::asm::nop();
    }

    // let mut cp = cortex_m::Peripherals::take().unwrap();
    // cp.SCB.enable_icache();

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.enable_debug_during_sleep = false;
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

    // Link SRAM3 power state to CPU1
    // pac::RCC.ahb2enr().modify(|w| w.set_sram3en(true));

    //enable HSEM clock
    pac::RCC.ahb4enr().modify(|w| w.set_hsemen(true));
    info!("HSEM clock enabled");

    let mut led_green = Output::new(p.PB0, Level::Low, Speed::Low);
    let mut led_red = Output::new(p.PB14, Level::Low, Speed::Low);

    let mut hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    // Take the semaphore for waking Core1 (CM4)
    if let Err(_err) = hsem.one_step_lock(0) {
        info!("Error taking semaphore 0");
    } else {
        info!("Semaphore 0 taken");
    }
    // Wake Core1 (CM4)
    hsem.unlock(0, 0);

    if hsem.is_interrupt_enabled(CoreId::Core1, 0) {
        info!("HSEM2 interrupt enabled");
    } else {
        info!("HSEM2 interrupt not enabled");
    }

    loop {
        // if let Err(_err) = hsem.two_step_lock(0, 0) {
        //     info!("Error taking semaphore for process 0");
        //     Timer::after_millis(1000).await;
        // } else {
        //     info!("Semaphore taken for process 0");
        // }

        led_green.set_high();
        Timer::after_millis(500).await;
        led_green.set_low();

        led_red.set_high();
        Timer::after_millis(500).await;
        led_red.set_low();
    }
}
