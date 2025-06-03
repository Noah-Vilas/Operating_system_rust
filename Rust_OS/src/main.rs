#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(Rust_OS::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use Rust_OS::println;
use x86_64::{structures::paging::Translate, VirtAddr,structures::paging::Page};
use Rust_OS::memory;
use Rust_OS::memory::BootInfoFrameAllocator;

use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    Rust_OS::init();

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };





    // as before
    #[cfg(test)]
    test_main();
    Rust_OS::hlt_loop();  
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