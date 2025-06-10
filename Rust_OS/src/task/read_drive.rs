#![feature(asm)]
#![feature(abi_x86_interrupt)]



use alloc::string::String;
use alloc::vec::Vec;
use crate::alloc::string::ToString;

use crate::println;


pub static mut drive_info: [u16; 256] = [0; 256];


fn print_drive_info(drive_inf: &[u16; 256]) {
    fn extract_string(start: usize, len: usize, data: &[u16]) -> String {
        let mut chars = Vec::with_capacity(len * 2);
        for &word in &data[start..start + len] {
            let high = (word >> 8) as u8;
            let low = (word & 0xFF) as u8;
            chars.push(high);
            chars.push(low);
        }
        String::from_utf8_lossy(&chars).trim().to_string()
    }

    let serial_number = extract_string(10, 10, drive_inf);   // 10 words = 20 bytes
    let firmware_rev  = extract_string(23, 4, drive_inf);    // 4 words = 8 bytes
    let model_number  = extract_string(27, 20, drive_inf);   // 20 words = 40 bytes

    println!("Serial Number : {}", serial_number);
    println!("Firmware Rev  : {}", firmware_rev);
    println!("Model Number  : {}", model_number);
}


pub async fn Test_drive() {
    let mut status: u8 = 0;
    unsafe {

        // Select master drive
        core::arch::asm!(
            "mov al, 0xA0",
            "out dx, al",
            in("dx") 0x1F6,
        );

        // Read initial status
        core::arch::asm!(
            "in al, dx",
            in("dx") 0x1F7,
            out("al") status,
        );

        // Wait until BSY=0 and DRQ=1 - update status inside loop!
        while (status & 0x08) != 0 {
            core::arch::asm!(
                "in al, dx",
                in("dx") 0x1F7,
                out("al") status,
            );
        }
        println!("Status before IDENTIFY: {:#x}", status);
        println!("fucking die");
        // Send IDENTIFY command (0xEC)
        core::arch::asm!(
            "mov al, 0xEC",
            "out dx, al",
            in("dx") 0x1F7,
        );
        println!("hello World");

        // Wait for BSY=0 and DRQ=1
        loop {
            core::arch::asm!(
                "in al, dx",
                in("dx") 0x1F7,
                out("al") status,
            );
            if status & 0x80 == 0 && status & 0x08 != 0 {
                break;
            }
        }
        println!("Status before reading IDENTIFY data: {:#x}", status);

        // Read 256 words (512 bytes) of IDENTIFY data correctly into drive_info
        for i in 0..256 {
            let word: u16;
            core::arch::asm!(
                "in ax, dx",
                in("dx") 0x1F0,
                out("ax") word,
            );
            drive_info[i] = word;
        }

        print_drive_info(&drive_info);
    }
}