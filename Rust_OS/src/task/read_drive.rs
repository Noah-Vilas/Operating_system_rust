
use crate::{print, println};

use alloc::vec::Vec;

const Max_lba: u32 = 100;
const ATA_DATA: u16 = 0x1F0;
const ATA_ERROR: u16 = 0x1F1;
const ATA_SECCOUNT0: u16 = 0x1F2;
const ATA_LBA0: u16 = 0x1F3;
const ATA_LBA1: u16 = 0x1F4;
const ATA_LBA2: u16 = 0x1F5;
const ATA_DRIVE: u16 = 0x1F6;
const ATA_COMMAND: u16 = 0x1F7;
const ATA_STATUS: u16 = 0x1F7;

const ATA_CMD_READ_PIO: u8 = 0x20;
const ATA_CMD_WRITE_PIO: u8 = 0x30;
const ATA_CMD_IDENTIFY:  u8 = 0xEC;

unsafe fn port_inb(port: u16) -> u8 {
    let mut data: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") data);
    data
}

unsafe fn port_outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value);
}

pub unsafe fn port_outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value);
}

unsafe fn port_inw(port: u16) -> u16 {
    let mut data: u16;
    core::arch::asm!("in ax, dx", in("dx") port, out("ax") data);
    data
}

unsafe fn ata_wait() {
    // Wait for BSY to clear
    while port_inb(ATA_STATUS) & 0x80 != 0 {}
    // Wait for DRQ to be set
    while port_inb(ATA_STATUS) & 0x08 == 0 {}
}
unsafe fn ata_wait_after_write() {
    while port_inb(ATA_STATUS) & 0x80 != 0 {} // Wait only for BSY to clear
}
pub unsafe fn ata_read_sector(lba: u32, buffer: &mut [u8; 512]) {
    // Send drive/head + LBA high 4 bits
    port_outb(ATA_DRIVE, 0xF0 | ((lba >> 24) & 0x0F) as u8);

    // Send sector count (1)
    port_outb(ATA_SECCOUNT0, 1);

    // Send LBA low/mid/high bytes
    port_outb(ATA_LBA0, (lba & 0xFF) as u8);
    port_outb(ATA_LBA1, ((lba >> 8) & 0xFF) as u8);
    port_outb(ATA_LBA2, ((lba >> 16) & 0xFF) as u8);

    // Send READ PIO command
    port_outb(ATA_COMMAND, ATA_CMD_READ_PIO);

    ata_wait();
    let ptr = buffer.as_mut_ptr();
    for i in 0..256 {
        let word = port_inw(ATA_DATA);
        *ptr.add(i * 2) = (word & 0xFF) as u8;
        *ptr.add(i * 2 + 1) = (word >> 8) as u8;
    }
}

pub unsafe fn ata_identify(buffer: &mut [u16; 256]) {
    // Select drive
    port_outb(ATA_DRIVE, 0xA0); // 0xA0 = master, CHS mode is fine for identify
    port_outb(ATA_SECCOUNT0, 0);
    port_outb(ATA_LBA0, 0);
    port_outb(ATA_LBA1, 0);
    port_outb(ATA_LBA2, 0);
    port_outb(ATA_COMMAND, ATA_CMD_IDENTIFY);

    // Wait for drive to respond
    if port_inb(ATA_STATUS) == 0 {
        return; // No drive
    }

    while (port_inb(ATA_STATUS) & 0x80) != 0 {} // Wait while BSY
    while (port_inb(ATA_STATUS) & 0x08) == 0 {} // Wait until DRQ

    // Read 256 words (512 bytes)
    for i in 0..256 {
        buffer[i] = port_inw(ATA_DATA);
    }
}

pub unsafe fn ata_write_sector(lba: u32, buffer: &mut [u8; 512]) {
    // Send drive/head + LBA high 4 bits
    port_outb(ATA_DRIVE, 0xF0 | ((lba >> 24) & 0x0F) as u8);

    // Send sector count (1)
    port_outb(ATA_SECCOUNT0, 1);

    // Send LBA low/mid/high bytes
    port_outb(ATA_LBA0, (lba & 0xFF) as u8);
    port_outb(ATA_LBA1, ((lba >> 8) & 0xFF) as u8);
    port_outb(ATA_LBA2, ((lba >> 16) & 0xFF) as u8);

    // Send READ PIO command
    port_outb(ATA_COMMAND, ATA_CMD_WRITE_PIO);

    ata_wait();
    let ptr = buffer.as_mut_ptr();
    for i in 0..256 {
        let lo = *ptr.add(i * 2) as u16;
        let hi = *ptr.add(i * 2 + 1) as u16;
        let word = lo | (hi << 8);
        port_outw(0x1F0, word);
    }
    ata_wait_after_write();
}


pub unsafe fn read_directory(dir_pointer: u32) {
    let mut buffer = [0u8; 512];
    ata_read_sector(dir_pointer, &mut buffer);

    let mut index = 0;
    let mut field_index = 0;

    println!("Reading directory at sector: {:#X}", dir_pointer);

    while index < 512 {
        // If double null (0x00 0x00), stop — end of directory
        if index + 1 < 512 && buffer[index] == 0 && buffer[index + 1] == 0 {
            println!("End of directory");
            break;
        }

        // Found a 0 delimiter — process the field between delimiters
        if buffer[index] == 0 {
            let mut field_data = Vec::new();
            let mut read_index = index + 1;

            // Read until next 0 or end of buffer
            while read_index < 512 && buffer[read_index] != 0 {
                field_data.push(buffer[read_index]);
                read_index += 1;
            }

            // Convert & print based on field position
            match field_index % 3 {
                0 => {
                    print!("File type: ");
                    for b in &field_data {
                        print!("{}", *b as char);
                    }
                    println!();
                }
                1 => {
                    print!("File name: ");
                    for b in &field_data {
                        print!("{}", *b as char);
                    }
                    println!();
                }
                2 => {
                    print!("File pointer: 0x");
                    for b in &field_data {
                        print!("{:02X}", b);
                    }
                    println!();
                }
                _ => {}
            }

            field_index += 1;
            index = read_index;
        } else {
            index += 1;
        }
    }
}
pub unsafe fn create_file(name: &str, pointer: u32) {
    let mut buffer = [0u8; 512];
    ata_read_sector(pointer, &mut buffer);

    // Find end of current directory entries (look for 00 terminator)
    let mut read_index = 0;
    while read_index < 511 && !(buffer[read_index] == 0 && buffer[read_index + 1] == 0) {
        read_index += 1;
    }

    println!("Appending at byte index: {}", read_index);

    // Split name into [type, filename]
    let parts: Vec<_> = name.split('.').collect();
    if parts.len() != 2 {
        println!("Invalid name format, must be type.name");
        return;
    }

    let file_type = parts[0].as_bytes();
    let file_name = parts[1].as_bytes();


    // Write name
    buffer[read_index] = 0;
    read_index += 1;
    for &b in file_name {
        buffer[read_index] = b;
        read_index += 1;
    }

    // Write type
    buffer[read_index] = 0;
    read_index += 1;
    for &b in file_type {
        buffer[read_index] = b;
        read_index += 1;
    }
    read_index += 1;
    // Write fake pointer for now (just use 0x00 0x00 0x00)
    let size: u32 = find_free_sector(0, Max_lba).unwrap_or(0);
    buffer[read_index]     = (size & 0xFF) as u8;
    buffer[read_index + 1] = ((size >> 8) & 0xFF) as u8;
    buffer[read_index + 2] = ((size >> 16) & 0xFF) as u8;
    buffer[read_index + 3] = ((size >> 24) & 0xFF) as u8;
    read_index += 4;
    // Write final 00 terminator after new entry
    buffer[read_index] = 0;
    read_index += 1;
    buffer[read_index] = 0;

    ata_write_sector(pointer, &mut buffer);
}



pub unsafe fn is_block_empty(buffer: &[u8; 512]) -> bool {
    // Consider block unused only if it does NOT start with 0x11
    !(buffer[0] == 0x11 && buffer[1] == 0x11)
}



pub unsafe fn find_free_sector(start_lba: u32, max_lba: u32) -> Option<u32> {
    let mut buffer = [0u8; 512];

    for lba in start_lba..max_lba {
        ata_read_sector(lba, &mut buffer);
        if is_block_empty(&buffer) {
            buffer[0] = 0x11;
            buffer[1] = 0x11;
            ata_write_sector(lba, &mut buffer);
            return Some(lba);
        }
    }
    None
}

pub unsafe fn zero_sector(lba: u32) {
    let mut buffer = [0u8; 512];
    ata_write_sector(lba, &mut buffer);
}

pub unsafe fn create_root_directory() {
    let mut buffer = [0u8; 512];

    // Tag the block as used (0x11 0x11)
    buffer[0] = 0x11;
    buffer[1] = 0x11;

    // Add 00 00 to mark end of directory (empty root)
    buffer[2] = 0x00;
    buffer[3] = 0x00;

    // Fill rest with zeroes (already is by default)

    ata_write_sector(0, &mut buffer);
    println!("Root directory created");
}