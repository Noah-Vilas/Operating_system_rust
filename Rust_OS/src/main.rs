#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(Rust_OS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use Rust_OS::println;
use core::panic::PanicInfo;
use Rust_OS::interrupts::KEYBOARD_BUFFER;
use x86_64::instructions::interrupts;
use bootloader::{BootInfo, entry_point};



entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    Rust_OS::init();


    let memory_map = &boot_info.memory_map;

    for region in memory_map.iter() {
        println!("{:?}", region);
    }


    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop{
        interrupts::without_interrupts(|| {
            if let Some(key) = KEYBOARD_BUFFER.lock().read_key() {
                println!("Key: {}", key as char);
            }
        });
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    Rust_OS::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Rust_OS::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}