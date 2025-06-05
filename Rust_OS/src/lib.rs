#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

use core::panic::PanicInfo;
use bootloader::BootInfo;
use crate::memory::{BootFrameAlloc, init_heap};

pub mod memory;
pub mod task;
pub use memory::{get_free_memory_regions,print_memory_layout, init_mapper};
use x86_64::VirtAddr;
pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

pub fn init(boot_info: &'static BootInfo) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { init_mapper(phys_mem_offset) };

    let mut frame_allocator = unsafe {
        BootFrameAlloc::init(&boot_info.memory_map)
    };

    init_heap(&mut mapper, frame_allocator);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

extern crate alloc;

pub mod allocator;

