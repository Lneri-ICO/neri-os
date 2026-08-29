use x86_64::instructions::port::Port;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::vga_graphics::{self, SCREEN_WIDTH, SCREEN_HEIGHT, COLOR_WHITE, COLOR_BLACK};

struct MouseState {
    x: i32,
    y: i32,
    packet: [u8; 3],
    packet_index: usize,
}

lazy_static! {
    static ref MOUSE: Mutex<MouseState> = Mutex::new(MouseState {
        x: (SCREEN_WIDTH / 2) as i32,
        y: (SCREEN_HEIGHT / 2) as i32,
        packet: [0; 3],
        packet_index: 0,
    });
}

unsafe fn wait_input() {
    let mut status_port: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        if status_port.read() & 0x02 == 0 {
            return;
        }
    }
}

unsafe fn wait_output() {
    let mut status_port: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        if status_port.read() & 0x01 != 0 {
            return;
        }
    }
}

unsafe fn mouse_write(value: u8) {
    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);
    wait_input();
    command_port.write(0xD4);
    wait_input();
    data_port.write(value);
}

unsafe fn mouse_read() -> u8 {
    let mut data_port: Port<u8> = Port::new(0x60);
    wait_output();
    data_port.read()
}

pub unsafe fn init() {
    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    // Habilita el puerto auxiliar (mouse)
    wait_input();
    command_port.write(0xA8);

    // Habilita interrupciones del mouse (IRQ12) en el byte de configuracion
    wait_input();
    command_port.write(0x20);
    wait_output();
    let mut status = data_port.read();
    status |= 0b0000_0010;
    status &= !0b0010_0000;
    wait_input();
    command_port.write(0x60);
    wait_input();
    data_port.write(status);

    // Restaura valores por defecto del mouse
    mouse_write(0xF6);
    let _ = mouse_read(); // ACK

    // Habilita el reporte de movimiento
    mouse_write(0xF4);
    let _ = mouse_read(); // ACK

    // Dibuja el cursor inicial
    draw_cursor();
}

fn draw_cursor() {
    let m = MOUSE.lock();
    let x = m.x.max(0) as usize;
    let y = m.y.max(0) as usize;
    // Cursor simple en forma de "L" (como una flecha basica)
    for i in 0..8 {
        vga_graphics::set_pixel(x, y + i, COLOR_WHITE);
    }
    for i in 0..6 {
        vga_graphics::set_pixel(x + i, y + i, COLOR_WHITE);
    }
    vga_graphics::set_pixel(x, y, COLOR_BLACK);
}

pub fn handle_byte(byte: u8) {
    let mut m = MOUSE.lock();
    let idx = m.packet_index;
    m.packet[idx] = byte;
    m.packet_index += 1;

    if m.packet_index >= 3 {
        m.packet_index = 0;

        let flags = m.packet[0];
        let mut dx = m.packet[1] as i32;
        let mut dy = m.packet[2] as i32;

        // Bits de signo del primer byte del paquete
        if flags & 0x10 != 0 {
            dx -= 256;
        }
        if flags & 0x20 != 0 {
            dy -= 256;
        }

        m.x = (m.x + dx).clamp(0, (SCREEN_WIDTH - 8) as i32);
        // El eje Y del mouse PS/2 esta invertido respecto a la pantalla
        m.y = (m.y - dy).clamp(0, (SCREEN_HEIGHT - 8) as i32);

        drop(m);

        // Redibuja el escritorio completo y el cursor en la nueva posicion
        // (simple pero suficiente para 320x200 a la velocidad de un mouse)
        vga_graphics::draw_desktop();
        draw_cursor();
    }
}