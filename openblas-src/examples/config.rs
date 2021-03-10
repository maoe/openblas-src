use std::{borrow::Cow, ffi::CStr, os::raw::c_char};

extern crate openblas_src;

fn main() {
    println!("{}", get_config());
}

fn get_config() -> Cow<'static, str> {
    unsafe { CStr::from_ptr(openblas_get_config()).to_string_lossy() }
}

extern "C" {
    fn openblas_get_config() -> *const c_char;
}
