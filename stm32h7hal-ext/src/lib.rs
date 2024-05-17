#![no_std]

use cortex_m::peripheral::NVIC;
use {embassy_stm32 as hal, embassy_stm32::pac};

#[allow(dead_code)]
pub enum PwrDomain {
    D1,
    D2,
    D3,
}

/// Specifies the regulator state in STOP mode.
#[allow(dead_code)]
pub enum PwrRegulator {
    /// STOP mode with regulator ON.
    MainRegulator,
    /// STOP mode with regulator in low power mode.
    LowPowerRegulator,
}

/// Specifies if STOP mode in entered with WFI or WFE
/// intrinsic instruction.
#[allow(dead_code)]
pub enum StopMode {
    /// Enter STOP mode with WFI instruction.
    StopEntryWfi,
    /// Enter STOP mode with WFE instruction.
    StopEntryWfe,
}

/// Enter a Domain to DSTOP mode.
/// This API gives flexibility to manage independently each domain STOP
/// mode. For dual core lines, this API should be executed with the
/// corresponding Cortex-Mx to enter domain to DSTOP mode. When it is
/// executed by all available Cortex-Mx, the system enter to STOP mode.
/// For single core lines, calling this API with domain parameter set to
/// PWR_D1_DOMAIN (D1/CD), the whole system will enter in STOP mode
/// independently of PWR_CPUCR_PDDS_Dx bits values if RUN_D3 bit in the
/// CPUCR_RUN_D3 is cleared.
/// In DStop mode the domain bus matrix clock is stopped.
/// The system D3/SRD domain enter Stop mode only when the CPU subsystem
/// is in CStop mode, the EXTI wakeup sources are inactive and at least
/// one PDDS_Dn bit in PWR CPU control register (PWR_CPUCR) for
/// any domain request Stop.
/// Before entering DSTOP mode it is recommended to call SCB_CleanDCache
/// function in order to clean the D-Cache and guarantee the data
/// integrity for the SRAM memories.
/// In System Stop mode, the domain peripherals that use the LSI or LSE
/// clock, and the peripherals that have a kernel clock request to
/// select HSI or CSI as source, are still able to operate.
///
/// This assumes that it runs on a dual core SoC
pub fn enter_stop_mode(stop_mode: StopMode, domain: PwrDomain) {
    // Enable the Stop mode
    pac::PWR.cr1().modify(|w| w.set_lpds(true));

    let mut scb = unsafe { cortex_m::Peripherals::steal().SCB };
    // FIXME: CPU2CR is not available in the PAC
    let cpu2cr = (0x40000000u32 + 0x18020000u32 + 0x4800u32) as *mut u32;

    match domain {
        PwrDomain::D1 => {
            if hal::hsem::get_current_coreid() == hal::hsem::CoreId::Core0 {
                // When the domain selected and the cortex-mx don't match, entering stop
                // mode will not be performed
                return;
            }
            // Keep DSTOP mode when D1/CD domain enters Deepsleep
            pac::PWR.cpucr().modify(|w| w.set_pdds_d1(false));

            // Set SLEEPDEEP bit of Cortex System Control Register
            scb.set_sleepdeep();

            // Ensure that all instructions are done before entering STOP mode
            cortex_m::asm::dsb();
            cortex_m::asm::isb();

            match stop_mode {
                StopMode::StopEntryWfi => {
                    // Request Wait For Interrupt
                    cortex_m::asm::wfi();
                }
                StopMode::StopEntryWfe => {
                    // Request Wait For Event
                    cortex_m::asm::wfe();
                }
            }

            // Clear SLEEPDEEP bit of Cortex-Mx in the System Control Register
            scb.clear_sleepdeep();
        }
        PwrDomain::D2 => {
            if hal::hsem::get_current_coreid() == hal::hsem::CoreId::Core0 {
                // When the domain selected and the cortex-mx don't match, entering stop
                // mode will not be performed
                return;
            }

            // Keep DSTOP mode when D2 domain enters Deepsleep
            // ((((PWR_TypeDef *) (((0x40000000UL) + 0x18020000UL) + 0x4800UL))->CPU2CR) &= ~((0x1UL << (1U))))
            // pac::PWR::cpu2cr().modify(|w| w.set_pdds_d2(false));
            // FIXME: CPU2CR is not available in the PAC
            unsafe {
                cpu2cr.write_volatile(cpu2cr.read_volatile() & !(0x1 << 1));
            }

            // Set SLEEPDEEP bit of Cortex System Control Register
            scb.set_sleepdeep();

            // Ensure that all instructions are done before entering STOP mode
            cortex_m::asm::dsb();
            cortex_m::asm::isb();

            match stop_mode {
                StopMode::StopEntryWfi => {
                    // Request Wait For Interrupt
                    cortex_m::asm::wfi();
                }
                StopMode::StopEntryWfe => {
                    // Request Wait For Event
                    cortex_m::asm::wfe();
                }
            }

            // Clear SLEEPDEEP bit of Cortex-Mx in the System Control Register
            scb.clear_sleepdeep();
        }
        PwrDomain::D3 => {
            if hal::hsem::get_current_coreid() == hal::hsem::CoreId::Core0 {
                // Keep DSTOP mode when D3 domain enters Deepsleep
                pac::PWR.cpucr().modify(|w| w.set_pdds_d3(false));
            } else {
                // Keep DSTOP mode when D3 domain enters Deepsleep
                // ((((PWR_TypeDef *) (((0x40000000UL) + 0x18020000UL) + 0x4800UL))->CPU2CR) &= ~((0x1UL << (2U))))
                // pac::PWR::cpu2cr().modify(|w| w.set_pdds_d3(false));
                // FIXME: CPU2CR is not available in the PAC
                unsafe {
                    cpu2cr.write_volatile(cpu2cr.read_volatile() & !(0x1 << 2));
                }
            }
        }
    }
}

fn hsem_activate_notification(sem_id: usize) {
    if hal::hsem::get_current_coreid() == hal::hsem::CoreId::Core0 {
        pac::HSEM.ier(0).modify(|w| w.set_ise(sem_id, true));
    } else {
        pac::HSEM.ier(1).modify(|w| w.set_ise(sem_id, true));
    }

    // TODO: remove once working
    let ier = (0x58026400u32 + 0x100u32 + 0x010u32) as *mut u32;
    unsafe {
        ier.write_volatile(ier.read_volatile() | 0x1);
    }
}

pub fn enable_hsem_clock() {
    pac::RCC.ahb4enr().modify(|w| w.set_hsemen(true));
}

pub fn clear_pending_events() {
    cortex_m::asm::sev();
    cortex_m::asm::wfe();
}

/// To be called from core0 at startup. This function waits
/// for core1 to be in Stop mode.
pub fn wait_for_core1() -> bool {
    // Wait for Core1 to be finished with its init
    // tasks and in Stop mode
    let mut timeout = 0xFFFF;
    while pac::RCC.cr().read().d2ckrdy() == true && timeout > 0 {
        timeout -= 1;
        // cortex_m::asm::nop();
    }
    if timeout > 0 {
        true
    } else {
        false
    }
}

/// To be called from core1 at startup. This function initializes
/// the HSEM peripheral and enters Stop mode. It returns from Stop
/// mode when core0 releases the semaphore 0.
pub fn core1_startup() {
    enable_hsem_clock();

    // pac::HSEM.icr(1).write(|w| w.set_isc(0, true));
    // pac::HSEM.icr(1).write(|w| w.set_isc(1, true));
    hsem_activate_notification(1); // FIXME: it should be 0 but needs to be the leased significant bit set

    clear_pending_events();

    unsafe {
        // NVIC::unpend(pac::Interrupt::HSEM2);
        NVIC::unmask(pac::Interrupt::HSEM2);
    };

    enter_stop_mode(StopMode::StopEntryWfe, PwrDomain::D2);
}
