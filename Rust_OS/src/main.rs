#![no_std]
#![no_main]



use Rust_OS::{get_free_memory_regions, print_memory_layout, init_mapper, println,
    memory::BootFrameAlloc,
    task::{keyboard, CLI, read_drive}
};
use Rust_OS::task::{Task, executor::Executor};
use core::panic::PanicInfo;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{PageTableFlags, PhysFrame, Page, Mapper};
use x86_64::VirtAddr;
use bootloader::{BootInfo, entry_point};

extern crate alloc;

use alloc::boxed::Box;


entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    Rust_OS::init(boot_info);


    let mut executor = Executor::new();
    executor.spawn(Task::new(read_drive::Test_drive()));
    executor.spawn(Task::new(CLI::CLI_START()));
    executor.spawn(Task::new(keyboard::handle_keypresses()));
    executor.run();

    println!("hello");


    Rust_OS::hlt_loop();
}




/// This function is called on panic.

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    Rust_OS::hlt_loop();
}
