use bootloader::bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};
use x86_64::structures::paging::Mapper;
use x86_64::structures::paging::{PageTableFlags, Page};
use crate::println;



//////////////////////

//     BASIC FUNCITONS TO PROCEED

//////////////////////
pub fn get_free_memory_regions(mem_map: &MemoryMap) -> impl Iterator<Item = &MemoryRegion> {
    mem_map.iter().filter(|region| region.region_type == MemoryRegionType::Usable)
}

pub fn print_memory_layout(mem_map: &MemoryMap){
    for region in mem_map.iter() {
        let start = region.range.start_frame_number * 4096;
        let end = region.range.end_frame_number * 4096;
        let size_kib = (end - start) / 1024;

        println!(
            "Region: {:?}, start: {:#x}, end: {:#x}, size: {} KiB",
            region.region_type,
            start,
            end,
            size_kib
        );
    }
}





//////////////////////////////

//  MORE IMPORTANT STUFF

//////////////////////////////


use x86_64::{
    VirtAddr,
    structures::paging::{OffsetPageTable, PageTable, PhysFrame, FrameAllocator}
};

pub unsafe fn init_mapper(physical_mem_offser: VirtAddr) -> OffsetPageTable<'static>{
    let level_4_page_table = active_level_4_page_table(physical_mem_offser);
    OffsetPageTable::new(level_4_page_table,physical_mem_offser)
}

pub unsafe fn active_level_4_page_table(phys_mem_off: VirtAddr) -> &'static mut PageTable{
    use x86_64::registers::control::Cr3;

    let (leve_4_frame, _) = Cr3::read();
    let phys_frame = leve_4_frame.start_address();
    let virtadd = phys_mem_off + phys_frame.as_u64();
    let pag_tab: *mut PageTable = virtadd.as_mut_ptr();
    &mut *pag_tab
}

pub struct BootFrameAlloc {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootFrameAlloc {

    pub unsafe fn init(memory_map: &'static MemoryMap)-> Self {
        BootFrameAlloc{memory_map, next: 0}
    }

    fn get_use_frame(&self) -> impl Iterator<Item = PhysFrame>{
        
        let free_mem_reg = get_free_memory_regions(self.memory_map);
        let addr_rang = free_mem_reg.map(|r| r.range.start_frame_number ..= r.range.end_frame_number);
        addr_rang.flat_map(|r| r.map(|frame_num| PhysFrame::containing_address(x86_64::PhysAddr::new(frame_num*4096))))
    }
}

unsafe impl FrameAllocator<x86_64::structures::paging::Size4KiB> for BootFrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.get_use_frame().nth(self.next);
        self.next += 1;
        frame
    }
}


////////////////////////////



////////    ALLOCATOR

/////////////////////////////

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();











pub const HEAP_START: usize = 0x4444_0000_0000;
pub const HEAP_SIZE: usize = 128 * 4096;


pub fn init_heap(mapper: &mut OffsetPageTable<'static>, mut allocator: BootFrameAlloc ){
    let start = VirtAddr::new((HEAP_START) as u64);
    let end = VirtAddr::new((HEAP_START+HEAP_SIZE-1) as u64);
    let range = Page::range(Page::containing_address(start), Page::containing_address(end));
    for page in range{
        let frame = allocator.allocate_frame().expect("NO MORE PAGES");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe{mapper.map_to(page, frame, flags,&mut allocator).expect("error").flush();}
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}