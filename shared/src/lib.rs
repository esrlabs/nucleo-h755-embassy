#![no_std]
#[allow(unused_imports)]
use core::mem::MaybeUninit;

pub mod rtt;
pub mod rtt_log;

#[cfg(feature = "core0")]
#[link_section = ".shared"]
#[export_name = "MAILBOX"]
#[used]
pub static mut MAILBOX: MaybeUninit<[u32; 10]> = MaybeUninit::uninit();

#[cfg(not(feature = "core0"))]
extern "C" {
    pub static mut MAILBOX: [u32; 10];
}

pub fn init_shared_memory() {
    // Start and end of shared memory
    extern "C" {
        static mut _sshared: u8;
        static mut _eshared: u8;
    }

    unsafe {
        use core::ptr::addr_of_mut;
        let count = addr_of_mut!(_eshared) as *const u8 as usize
            - addr_of_mut!(_sshared) as *const u8 as usize;
        core::ptr::write_bytes(addr_of_mut!(_sshared) as *mut u8, 0, count);
    }
}
