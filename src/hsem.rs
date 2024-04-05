// STM32H7 Hardware semaphore (HSEM)
// this should go into the HAL

//use stm32h7::stm32h747cm4 as pac1;
use embassy_stm32::pac;

#[derive(Debug)]
pub enum HsemError {
    InvalidHsemIndex,
    TakeFailed,
}

/// CPU core.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
#[derive(defmt::Format)]
pub enum Core {
    /// Cortex-M7, core 1.
    Cm7 = 0x3,
    /// Cortex-M4, core 2.
    Cm4 = 0x1,
}

#[inline(always)]
pub fn get_current_cpuid() -> Core {
    let cpuid = unsafe { cortex_m::peripheral::CPUID::PTR.read_volatile().base.read() };
    if ((cpuid & 0x000000F0) >> 4) == 0x7 {
        Core::Cm7
    } else {
        Core::Cm4
    }
}

pub trait Hsem {
    fn take(&self, sem_id: u8, process_id: u8) -> Result<(), HsemError>;
    fn fast_take(&self, sem_id: u8) -> Result<(), HsemError>;
    fn release(&self, sem_id: u8, process_id: u8) -> Result<(), HsemError>;
    fn release_all(&self, key: u16, core_id: u8);
    fn is_semaphore_taken(&self, sem_id: u8) -> Result<bool, HsemError>;
    fn set_clear_key(&self, key: u16);
    fn get_clear_key(&self) -> u16;
}

pub trait HsemNotify {
    fn enable_interrupt(&self, core_id: Core, sem_x: usize, enable: bool);
    fn clear_interrupt(&self, core_id: Core, sem_x: usize);
}

impl Hsem for pac::hsem::Hsem {
    fn take(&self, sem_id: u8, process_id: u8) -> Result<(), HsemError> {
        if sem_id > 31 {
            Err(HsemError::InvalidHsemIndex)
        } else {
            self.r(sem_id as usize).write(|w| {
                w.set_procid(process_id);
                w.set_coreid(get_current_cpuid() as u8);
                w.set_lock(true);
            });

            Ok(())
        }
    }

    fn fast_take(&self, sem_id: u8) -> Result<(), HsemError> {
        if sem_id > 31 {
            Err(HsemError::InvalidHsemIndex)
        } else {
            let register = self.rlr(sem_id as usize).read();
            match (
                register.lock(),
                register.coreid() == get_current_cpuid() as u8,
                register.procid(),
            ) {
                (false, true, 0) => Ok(()),
                _ => Err(HsemError::TakeFailed),
            }
        }
    }

    fn release(&self, sem_id: u8, process_id: u8) -> Result<(), HsemError> {
        if sem_id > 31 {
            Err(HsemError::InvalidHsemIndex)
        } else {
            self.r(sem_id as usize).write(|w| {
                w.set_procid(process_id);
                w.set_coreid(get_current_cpuid() as u8);
                w.set_lock(false);
            });
            Ok(())
        }
    }

    fn release_all(&self, key: u16, core_id: u8) {
        self.cr().write(|w| {
            w.set_key(key);
            w.set_coreid(core_id);
        });
    }

    fn is_semaphore_taken(&self, sem_id: u8) -> Result<bool, HsemError> {
        if sem_id > 31 {
            return Err(HsemError::InvalidHsemIndex);
        } else {
            Ok(self.r(sem_id as usize).read().lock())
        }
    }

    fn set_clear_key(&self, key: u16) {
        self.keyr().modify(|w| w.set_key(key));
    }

    fn get_clear_key(&self) -> u16 {
        self.keyr().read().key()
    }
}

impl HsemNotify for pac::hsem::Hsem {
    fn enable_interrupt(&self, core_id: Core, sem_x: usize, enable: bool) {
        match core_id {
            Core::Cm7 => self.c1ier().modify(|w| w.set_ise(sem_x, enable)),
            Core::Cm4 => self.c2ier().modify(|w| w.set_ise(sem_x, enable)),
        }
    }

    fn clear_interrupt(&self, core_id: Core, sem_x: usize) {
        match core_id {
            Core::Cm7 => self.c1icr().write(|w| w.set_isc(sem_x, false)),
            Core::Cm4 => self.c2icr().write(|w| w.set_isc(sem_x, false)),
        }
    }
}
