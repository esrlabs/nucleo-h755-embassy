#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;
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

// ************************************************************
// The following code will go into the HAL once finished
// ************************************************************

pub trait HsemSemaphore {
    async fn lock(&self) -> bool;
    fn release(&self);
}

/// Shared Hardware Semaphore (HSEM) device.
pub struct SharedHSEM<'a, M: RawMutex, SEM> {
    sem_dev: &'a Mutex<M, SEM>,
    sem_id: u8,
}

impl<'a, M: RawMutex, SEM> SharedHSEM<'a, M, SEM> {
    /// Create a new `SingleSem`.
    pub fn new(sem_dev: &'a Mutex<M, SEM>, sem_id: u8) -> Self {
        Self { sem_dev, sem_id }
    }
}

impl<M, SEM> HsemSemaphore for SharedHSEM<'_, M, SEM>
where
    M: RawMutex + 'static,
    SEM: HsemSemaphore + 'static,
{
    async fn lock(&self) -> bool {
        let mut sem_dev = self.sem_dev.lock().await;
        sem_dev.lock().await
    }

    fn release(&self) {
        let mut sem_dev = self.sem_dev.lock();
        sem_dev.unlock();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Core0: STM32H755 Embassy HSEM Test.");

    unsafe { info!("Mailbox = {:?}", shared::MAILBOX) };

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

    let mut hsem = HardwareSemaphore::new(p.HSEM, Irqs);

    unsafe { NVIC::unmask(embassy_stm32::pac::Interrupt::HSEM1) };
    // Take the semaphore for waking Core1 (CM4)
    if !hsem.one_step_lock(0) {
        info!("Error taking semaphore 0");
    } else {
        info!("Semaphore 0 taken");
    }

    // Wake Core1 (CM4)
    hsem.unlock(0, 0);
    info!("Core1 (CM4) woken");
    led_green.set_high();
    Timer::after_millis(250).await;
    led_green.set_low();

    info!("Waiting for Sem 1");
    let _ = hsem.wait_unlocked(1).await;
    led_green.set_high();
    led_red.set_high();
    info!("Waiting for Sem 2");
    let _ = hsem.wait_unlocked(2).await;
    led_red.set_low();
    led_green.set_low();

    loop {
        led_green.set_high();
        Timer::after_millis(500).await;
        led_green.set_low();

        led_red.set_high();
        Timer::after_millis(500).await;
        led_red.set_low();
    }
}

async fn set_core1_blink_frq(freq: u32, hsem: &mut HardwareSemaphore<'static, HSEM>) {
    let mut retry = 10;
    while !hsem.lock(1).await && retry > 0 {
        Timer::after_micros(50).await;
        retry -= 1;
    }
    if retry > 0 {
        unsafe {
            shared::MAILBOX[0] = freq;
        }
        hsem.unlock(1, 0);
    } else {
        // Core1 has asquired the semaphore and is
        // not releasing it - crashed?
        defmt::panic!("Failed to asquire semaphore 1");
    }
}
