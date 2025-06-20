use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};



pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
const ATA_PRIMARY_IRQ: u8 = 14;




#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.general_protection_fault.set_handler_fn(general_protection_handler);
            idt.page_fault.set_handler_fn(page_fault_handler);
            idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
            idt.overflow.set_handler_fn(overflow_handler);
            idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);

            idt.divide_error.set_handler_fn(divide_by_zero_handler);
            idt.debug.set_handler_fn(debug_handler);
            idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
            idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
            idt.invalid_tss.set_handler_fn(invalid_tss_handler);
            idt.segment_not_present.set_handler_fn(segment_not_present_handler);
            idt.alignment_check.set_handler_fn(alignment_check_handler);
            idt.machine_check.set_handler_fn(machine_check_handler);
            idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[(PIC_1_OFFSET + ATA_PRIMARY_IRQ) as usize ].set_handler_fn(ata_primary_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}



extern "x86-interrupt" fn ata_primary_interrupt_handler(_stack_frame: InterruptStackFrame) {
    println!("ATA Primary Interrupt fired!");

    // Notify PIC that interrupt handling is done
    unsafe {
        PICS.lock().notify_end_of_interrupt(PIC_1_OFFSET + ATA_PRIMARY_IRQ);
    }
}


extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Divide By Zero\n{:#?}", stack_frame);
    panic!("Divide By Zero Exception");
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Debug\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Non Maskable Interrupt (NMI)\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Bound Range Exceeded\n{:#?}", stack_frame);
    panic!("Bound Range Exceeded Exception");
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: Invalid TSS\nError Code: {:#x}\n{:#?}", error_code, stack_frame);
    panic!("Invalid TSS Exception");
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: Segment Not Present\nError Code: {:#x}\n{:#?}", error_code, stack_frame);
    panic!("Segment Not Present Exception");
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: Alignment Check\nError Code: {:#x}\n{:#?}", error_code, stack_frame);
    panic!("Alignment Check Exception");
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    println!("EXCEPTION: Machine Check\n{:#?}", stack_frame);
    panic!("Machine Check Exception");
}

extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: SIMD Floating-Point\n{:#?}", stack_frame);
}



extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: Stack Segment Fault\nError Code: {:#x}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, pagefaultcode: PageFaultErrorCode) {
    println!("EXCEPTION: Page Fault Error: \n{:#?}", stack_frame);
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Overflow\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: Invalid Opcode\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    println!("EXCEPTION: General Protection \n{:#?}. Error Code: {}", stack_frame, error_code);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}