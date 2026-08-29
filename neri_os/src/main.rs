#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod serial;
mod gdt;
mod interrupts;
mod memory;
mod allocator;
mod shell;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Iniciando NeriOS...");

    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };

    // Habilita interrupciones del puerto serial (IRQ4) para recibir texto
    unsafe {
        use x86_64::instructions::port::Port;
        let mut port: Port<u8> = Port::new(0x3F9); // IER del COM1
        port.write(0x01u8); // habilita "data available" interrupt
    }

    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Fallo la inicializacion del heap");

    println!("NeriOS listo. Memoria dinamica activa.");
    shell::init();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}