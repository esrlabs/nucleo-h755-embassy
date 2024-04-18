# Exploring multi-core usage on Nucleo-H755 with `embassy`

This project contains various attempts at using both cores of the [Nucleo-H755 board][nucleo]
together with [`embassy`][embassy].

## Getting started

- [Install probe-rs][probe-rs-install]
- [Setup the probe (Linux)][probe-rs-setup]
- Add the matching target for `rustc`: `rustup target add thumbv7em-none-eabihf`

Optionally:

- Install the "Debugger for probe-rs" extention in VS-Code

## Notes

At the moment, there is an example checked in which shows coordination between the cores.

- [core0](./core0/) contains the binary to be run on the M7.
- [core1](./core1/) contains the binary to be run on the M4.
- On power-up, both cores start executing their binary.
  - core1 quickly hits a semaphore and goes to sleep waiting on this semaphore.
  - core0 does most of the setup and wakes up core1 by unlocking one semaphore while waiting for
    another semaphore.
  - core1 wakes up, initializes just its own clock and then unlocks the other semaphore.
  - Now, both cores run in parallel, coordinating via semaphores.

## Todos

1. Project structure for dual core? (M)
   - Main project
   - Core 0 project
   - Core 1 project
   - Shared project
2. How to use a customized memory.x file? (S)
3. How to upload a binary to the second core? Which address? Where to configure? (S)
   1. How do we need to configure embassy-stm32 for the second core
      (i.e. remove "rt" feature - the rt feature populates the NVIC table with the (default) interrupt handlers,
      which might not be what we want for the second core).
      This also raises the question how do we configure the NVIC with interrupt handlers for both cores.
4. How do we use `defmt` for logging on the second core? What needs to be done to receive the output in probe-rs? (S)
   1. Could we write our own [global `Logger`][logger-trait]
      which waits for a hardware semaphore in [`Logger::acquire`][logger-acquire]?
5. Which build system do we use? Can we do everything with cargo or do we need something like cargo-make?
   Cargo only is preferred. (M)
6. Define and implement start-up sequence for dual core.
   Startingpoint is the STM Cube SDK example.
   If we follow this approach,
   we should implement the required functions in a `stm32h7xx_hal_dual_core` supplement
   (assuming that the functionality required is mostly net new and not changes to existing HAL functionality)
   crate for the time being and decide later if we want to upstream it.

   ![Start-up sequence as provided in the STM Cube SDK example](/assets/startup_seq.png)

7. Integrate with hamoc
   1. Setup FreeRTOS config for the two cores
   2. Integrate into the hamoc workspace
8. Define a concept for peripherals initialization and ownership
9. Improve debugging
   1. Could we debug both cores at the same time using e.g. [`openocd`][openocd]?
   2. Could we debug both cores at the same time using VS-Code?
      (see e.g. [multi-core debugging discussion in `cortex-debug`][mc-debug-discussion])

[nucleo]: https://www.st.com/en/evaluation-tools/nucleo-h755zi-q.html
[embassy]: https://github.com/embassy-rs/embassy
[probe-rs-install]: https://probe.rs/docs/getting-started/installation/
[probe-rs-setup]: https://probe.rs/docs/getting-started/probe-setup/#linux%3A-udev-rules
[logger-trait]: https://github.com/knurling-rs/defmt/blob/main/defmt/src/traits.rs#L90
[logger-acquire]: https://github.com/knurling-rs/defmt/blob/main/firmware/defmt-rtt/src/lib.rs#L52
[openocd]: https://openocd.org/
[mc-debug-discussion]: https://github.com/Marus/cortex-debug/issues/152#issuecomment-492815247