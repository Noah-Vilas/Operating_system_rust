#![no_std]
#![no_main]



use Rust_OS::{get_free_memory_regions, print_memory_layout, init_mapper, println,
    interrupts::KEYBOARD_BUFFER,
    memory::BootFrameAlloc,
};

use core::panic::PanicInfo;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{PageTableFlags, PhysFrame, Page, Mapper};
use x86_64::VirtAddr;
use x86_64::structures::paging::FrameAllocator;
use bootloader::{BootInfo, entry_point};
use Rust_OS::memory::init_heap;

extern crate alloc;

use alloc::boxed::Box;


entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    Rust_OS::init();


    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_mapper(phys_mem_offset) };

    let mut frame_allocator = unsafe {
        BootFrameAlloc::init(&boot_info.memory_map)
    };

    init_heap(&mut mapper, frame_allocator);

    let test_box = Box::new(42);
    println!("It's a Box {}", test_box);

    Rust_OS::hlt_loop();
}




/// This function is called on panic.

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    Rust_OS::hlt_loop();
}