use bindgen::Builder;

use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This call will make make config entries available in the code for every device tree node, to
    // allow conditional compilation based on whether it is present in the device tree.
    // For example, it will be possible to have:
    // ```rust
    // #[cfg(dt = "aliases::led0")]
    // ```
    zephyr_build::dt_cfgs();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let zephyr_base = env::var("ZEPHYR_BASE")?;
    let target = env::var("TARGET")?;
    let target_arg = format!("--target={}", target);

    let bindings = Builder::default()
        .header(
            Path::new("src/cffi.h")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .use_core()
        .clang_arg(&target_arg)
        .clang_arg(format!("-I{}/lib/libc/minimal/include", zephyr_base))
        .derive_copy(false)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}