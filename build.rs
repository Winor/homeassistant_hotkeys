#[cfg(windows)]
extern crate windres;

#[cfg(windows)]
use windres::Build;

#[cfg(windows)]
fn main() {
    Build::new().compile("ico.rc").unwrap();
}

#[cfg(not(windows))]
fn main() {}