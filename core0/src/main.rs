#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]
use core::cell::SyncUnsafeCell;
use cortex_m::peripheral::NVIC;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    hsem::{HardwareSemaphore, InterruptHandler},
    peripherals::{self, HSEM},
    Config,
};
use {
    defmt_rtt as _, embassy_stm32 as hal, panic_probe as _, shared as _, stm32h7hal_ext as hal_ext,
};

bind_interrupts!(
    struct Irqs {
        HSEM1 => InterruptHandler<peripherals::HSEM>;
    }
);

// SAFETY: This is safe because all access to the HSEM registers are atomic
static HSEM_INSTANCE: SyncUnsafeCell<Option<HardwareSemaphore<'static, HSEM>>> =
    SyncUnsafeCell::new(None);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core0: STM32H755 Embassy HSEM Test.");
    // let mut mailbox = unsafe { *shared::MAILBOX.get() as [u32; 10] };
    // for i in 0..10 {
    //     mailbox[i] = i as u32;
    // }
    // let m = unsafe { *shared::MAILBOX.get() as [u32; 10] };
    // info!("Mailbox = {:?}", m);
    unsafe { info!("Mailbox = {:?}", shared::MAILBOX) };
    unsafe {
        for i in 0..10 {
            shared::MAILBOX[i] = 0;
            info!("Mailbox = 0x{:02x}", shared::MAILBOX[i]);
        }
    }
    // Wait for Core1 to be finished with its init
    // tasks and in Stop mode
    hal_ext::wait_for_core1();

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

    hal_ext::enable_hsem_clock();
    info!("HSEM clock enabled");

    let mut led_green = Output::new(p.PB0, Level::Low, Speed::Low);
    let mut led_red = Output::new(p.PB14, Level::Low, Speed::Low);

    let hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    // initialize global HSEM instance
    unsafe { *HSEM_INSTANCE.get() = Some(hsem) };

    unsafe { NVIC::unmask(embassy_stm32::pac::Interrupt::HSEM1) };
    // Take the semaphore for waking Core1 (CM4)
    if !get_global_hsem().one_step_lock(0) {
        info!("Error taking semaphore 0");
    } else {
        info!("Semaphore 0 taken");
    }

    let mut core_1_blink_delay = 500;
    set_core1_blink_delay(core_1_blink_delay).await;

    // Wake Core1 (CM4)
    get_global_hsem().unlock(0, 0);
    info!("Core1 (CM4) woken");
    led_green.set_high();
    Timer::after_millis(250).await;
    led_green.set_low();

    info!("Waiting for Sem 1");
    let _ = get_global_hsem().wait_unlocked(1).await;
    led_green.set_high();
    led_red.set_high();
    info!("Waiting for Sem 2");
    let _ = get_global_hsem().wait_unlocked(2).await;
    led_red.set_low();
    led_green.set_low();

    loop {
        led_green.set_high();
        Timer::after_millis(500).await;
        led_green.set_low();

        led_red.set_high();
        Timer::after_millis(500).await;
        led_red.set_low();
        info!("Set Core 1 blink delay {}", core_1_blink_delay);
        set_core1_blink_delay(core_1_blink_delay).await;
        if core_1_blink_delay < 100 {
            core_1_blink_delay = 500;
        }
        core_1_blink_delay -= 50;
    }
}

async fn set_core1_blink_delay(freq: u32) {
    let mut retry = 10;
    while !get_global_hsem().lock(5).await && retry > 0 {
        Timer::after_micros(50).await;
        retry -= 1;
    }
    if retry > 0 {
        unsafe {
            //let mut mailbox = *shared::MAILBOX.get() as [u32; 10];
            unsafe {
                shared::MAILBOX[0] = freq;
            };
        }
        get_global_hsem().unlock(5, 0);
    } else {
        // Core1 has asquired the semaphore and is
        // not releasing it - crashed?
        defmt::panic!("Failed to asquire semaphore 1");
    }
}

fn get_global_hsem() -> &'static mut HardwareSemaphore<'static, HSEM> {
    unsafe {
        match *HSEM_INSTANCE.get() {
            Some(ref mut obj) => obj,
            None => defmt::panic!("HardwareSemaphore was not initialized"),
        }
    }
}
