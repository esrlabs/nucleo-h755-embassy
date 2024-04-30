#![no_std]
#![feature(sync_unsafe_cell)]

use core::cell::SyncUnsafeCell;

#[link_section = ".shared"]
#[export_name = "MAILBOX"]
// The initial value is not written into memory, but needs to be
// done to make the compiler happy
pub static mut MAILBOX: [u32; 10] = [0; 10];
// SAFETY: This is safe because all access to the HSEM registers are atomic
//pub static MAILBOX: SyncUnsafeCell<[u32; 10]> = SyncUnsafeCell::new([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

pub fn dummy() {
    // do nothing
}
