#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(Rust_OS::test_runner)]
#![reexport_test_harness_main = "test_main"]



use Rust_OS::{QemuExitCode, exit_qemu, serial_println};
use Rust_OS::serial_print;
use core::panic::PanicInfo;
use Rust_OS::{println, test_panic_handler};

#[no_mangle] // "unsafe" here is incorrect; just use no_mangle
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

// This will be used when not linking the full Rust_OS crate's test runner
// You can delete this if you're using `Rust_OS::test_runner` correctly
/*
fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}
*/

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}


#[test_case]
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(1, 1);
}