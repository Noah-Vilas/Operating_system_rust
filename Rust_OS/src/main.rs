#![no_std]
#![no_main]



use Rust_OS::{get_free_memory_regions, print_memory_layout, init_mapper, println, print,
    memory::BootFrameAlloc,
    task::{keyboard,
         CLI,
         read_drive::{ata_read_sector,ata_write_sector},
        }
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

    let mut buffer = [0u8; 512];
    unsafe {
        ata_read_sector(0x00000000, &mut buffer);
    }
    for (i, byte) in buffer.iter().enumerate() {
        print!("{:02X} ", byte);
    }
    let mut executor = Executor::new();
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
