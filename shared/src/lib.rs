#![no_std]
#![feature(sync_unsafe_cell)]

#[link_section = ".shared"]
#[export_name = "MAILBOX"]
// The initial value is not written into memory, but needs to be
// done to make the compiler happy.
// FIXME: Replace with MaybeUninit
#[used]
pub static mut MAILBOX: [u32; 10] = [0; 10];

// Define RTT channel setup for both cores
#[macro_export]
macro_rules! rtt_config {
    () => {
        rtt_init_multi_core! {
          ".shared",
          up: {
              0: {
                  size: 512,
                  mode: ChannelMode::NoBlockTrim,
                  name: "Up core 0"
              }
              1: {
                  size: 512,
                  mode: ChannelMode::NoBlockTrim,
                  name: "Up core 1"
              }
          }
        }
    };
}

#[macro_export]
macro_rules! rtt_init_channels_multi_core {
    (
        $link_section:literal,
        $field:expr;
        $number:literal: {
            size: $size:expr
            $(, mode: $mode:path )?
            $(, name: $name:literal )?
            $(,)?
        }
        $($tail:tt)*
    ) => {
        let mut name: *const u8 = core::ptr::null();
        $( name = concat!($name, "\0").as_bytes().as_ptr(); )?

        let mut mode = rtt_target::ChannelMode::NoBlockSkip;
        $( mode = $mode; )?

        $field[$number].init(name, mode, {
            #[link_section = $link_section]
            static mut _RTT_CHANNEL_BUFFER: MaybeUninit<[u8; $size]> = MaybeUninit::uninit();
            _RTT_CHANNEL_BUFFER.as_mut_ptr()
        });

        $crate::rtt_init_channels_multi_core!($link_section, $field; $($tail)*);
    };
    ($link_section:literal, $field:expr;) => { };
}

#[macro_export]
macro_rules! rtt_init_multi_core {
    {
        $link_section:literal,
        $(up: { $($up:tt)* } )?
        $(down: { $($down:tt)* } )?
    } => {{
        use core::mem::MaybeUninit;
        use core::ptr;
        use rtt_target::{UpChannel, DownChannel, rtt::*};

        #[repr(C)]
        pub struct RttControlBlock {
            header: RttHeader,
            up_channels: [RttChannel; rtt_target::rtt_init_repeat!({ 1 + } { 0 }; $($($up)*)?)],
            down_channels: [RttChannel; rtt_target::rtt_init_repeat!({ 1 + } { 0 }; $($($down)*)?)],
        }

        #[used]
        #[no_mangle]
        #[export_name = "_SEGGER_RTT"]
        #[link_section = $link_section]
        pub static mut CONTROL_BLOCK: MaybeUninit<RttControlBlock> = MaybeUninit::uninit();

        unsafe {
            ptr::write_bytes(CONTROL_BLOCK.as_mut_ptr(), 0, 1);

            let cb = &mut *CONTROL_BLOCK.as_mut_ptr();

            $( $crate::rtt_init_channels_multi_core!($link_section, cb.up_channels; $($up)*); )?
            $( $crate::rtt_init_channels_multi_core!($link_section, cb.down_channels; $($down)*); )?

            // The header is initialized last to make it less likely an unfinished control block is
            // detected by the host.

            cb.header.init(cb.up_channels.len(), cb.down_channels.len());

            pub struct Channels {
                $( pub up: rtt_target::rtt_init_repeat!({ UpChannel, } {}; $($up)*), )?
                $( pub down: rtt_target::rtt_init_repeat!({ DownChannel, } {}; $($down)*), )?
            }

            Channels {
                $( up: rtt_target::rtt_init_wrappers!(cb.up_channels; UpChannel::new; {}; $($up)*), )?
                $( down: rtt_target::rtt_init_wrappers!(cb.down_channels; DownChannel::new; {}; $($down)*), )?
            }
        }
    }};
}
