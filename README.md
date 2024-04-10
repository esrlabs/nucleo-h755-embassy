# Exploring multi-core

## Todos

1. Project structure for dual core? (M)
   - Main project
   - Core 0 project
   - Core 1 project
   - Shared project
2. How to use a customized memory.x file? (S)
3. How to upload a binary to the second core? Which address? Where to configure? (S)
   a.) How do we need to configure embassy-stm32 for the second core (i.e. remove "rt" feature - the rt feature populates the NVIC table with the (default) interrupt handlers, which might not be what we want for the second core). This also raises the question how do we configure the NVIC with interrupt handlers for both cores.
4. How do we use dfmt for logging the second core? What needs to be done to receive the output in probe-rs? (S)
5. Which build system do we use? Can we do everything with cargo or do we need something like cargo-make? Cargo only is preferred. (M)
6. Define and implement start-up sequence for dual core. Startingpoint is the STM Cube SDK example. If we follow this approach, we should implement the required functions in a `stm32h7xx_hal_dual_core` supplement (assuming that the functionality required is mostly net new and not changes to existing HAL functionality) crate for the time being and decide later if we want to upstream it.

   ![Start-up sequence as provided in the STM Cube SDK example](/assets/startup_seq.png)

7. Integrate with hamoc
   a.) Setup FreeRTOS config for the two cores
   b.) Integrate into the hamoc workspace
8. Define a concept for peripherals initialization and ownership
