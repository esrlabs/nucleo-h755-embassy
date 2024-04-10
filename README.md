# Exploring multi-core

## Todos

1. How to use a customized memory.x file?
2. How to upload a binary to the second core? Which address? Where to configure?
3. Project structure for dual core?
   - Main project
   - Core 0 project
   - Core 1 project
   - Shared project
4. Which build system do we use? Can we do everything with cargo or do we need something like cargo-make?
5. Define and implement start-up sequence for dual core. Startingpoint is the STM Cube SDK example. If we follow this approach, we should implement the required functions in a `stm32h7xx_hal_dual_core` suplement (assuming that the functionality required is mostly net new and not changes to existing HAL functionality) crate for the time beeing and decide later if we want to upstream it.

   ![Start-up sequence as provided in the STM Cube SDK example](/assets/startup_seq.png)

6. Integrate with hamoc
   a.) Setup FreeRTOS config for the two cores
   b.) Integrate into the hamoc workspace
7. Define a concept for pheripherals initialization and ownership
