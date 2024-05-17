//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Generate linker input for shared memory symbols defined in the core0 elf file
    Command::new("arm-none-eabi-ld")
        .args(&[
            "-r",
            format!(
                "--just-symbols=../core0/target/thumbv7em-none-eabihf/{}/core0",
                env::var("PROFILE").unwrap()
            )
            .as_str(),
            "-o",
            "./target/shared.elf",
            "--retain-symbols-file",
            "../shared/to-export.txt",
        ])
        .output()
        .expect("Failed to execute arm-none-eabi-ld");

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    //println!("cargo:rustc-link-search=/usr/lib/arm-none-eabi/newlib/thumb/v6-m/nofp");
    //println!("cargo:rustc-link-lib=static:+whole-archive=c_nano");

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
    println!(
        "cargo:rerun-if-changed=../core0/target/thumbv7em-none-eabihf/{}/core0",
        env::var("PROFILE").unwrap()
    );
}
