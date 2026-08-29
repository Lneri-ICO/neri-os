#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod vga_graphics;
mod mouse;
mod gdt;
mod interrupts;
mod memory;
mod allocator;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Fallo la inicializacion del heap");

    unsafe {
        vga_graphics::init_mode_13h();
    }

    vga_graphics::draw_desktop();

    unsafe {
        mouse::init();
    }

    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_graphics::fill_screen(vga_graphics::COLOR_RED);
    loop {
        x86_64::instructions::hlt();
    }
}