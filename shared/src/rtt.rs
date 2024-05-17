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
                  name: "Terminal"
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
macro_rules! rtt_config_shared {
    () => {
        rtt_init_multi_core_shared! {
          ".shared",
          up: {
              0: {
                  size: 512,
                  mode: ChannelMode::NoBlockTrim,
                  name: "Terminal"
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
            #[export_name = concat!("_RTT_CHANNEL_BUFFER_",stringify!($number))]
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

#[macro_export]
macro_rules! rtt_init_multi_core_shared {
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

        extern "C" {
            static mut _SEGGER_RTT: RttControlBlock;

        }


        unsafe {
            use core::ptr::addr_of_mut;
            //let cb = &mut *addr_of_mut!(_SEGGER_RTT);

            // FIXME: make this dynamic
            pub struct Channels {
                // $( pub up: rtt_target::rtt_init_repeat!({ UpChannel, } {}; $($up)*), )?
                // $( pub down: rtt_target::rtt_init_repeat!({ DownChannel, } {}; $($down)*), )?
                pub up: (UpChannel, UpChannel),
            }

            // FIXME: make this dynamic
            Channels {
                // $( up: rtt_target::rtt_init_wrappers!(cb.up_channels; UpChannel::new; {}; $($up)*), )?
                // $( down: rtt_target::rtt_init_wrappers!(cb.down_channels; DownChannel::new; {}; $($down)*), )?
               up: ( UpChannel::new(&mut _SEGGER_RTT.up_channels[0] as *mut _),
                     UpChannel::new(&mut _SEGGER_RTT.up_channels[1] as *mut _),
            ),
            }
        }
    }};
}
