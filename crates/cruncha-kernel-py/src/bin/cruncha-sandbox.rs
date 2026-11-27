// SPDX-License-Identifier: GPL-3.0-or-later
//! `cruncha-sandbox <python> <args…>`: run the Python worker in its Windows sandbox
//! (ADR-0030; `sandbox::windows`). The app does the same as `cruncha-app --py-sandbox`; this
//! binary is what the crate's tests use.

fn main() {
    #[cfg(windows)]
    std::process::exit(cruncha_kernel_py::sandbox::windows::launch(std::env::args_os().skip(1).collect()));
    #[cfg(not(windows))]
    {
        eprintln!("cruncha-sandbox: Windows only (macOS uses sandbox-exec)");
        std::process::exit(2);
    }
}
