use x86_64::instructions::port::Port;
use core::ptr::write_volatile;

pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 200;

// Colores clasicos VGA de 16 colores (funcionan igual en la paleta de modo 13h)
pub const COLOR_BLACK: u8 = 0;
pub const COLOR_BLUE: u8 = 1;
pub const COLOR_GREEN: u8 = 2;
pub const COLOR_CYAN: u8 = 3;
pub const COLOR_RED: u8 = 4;
pub const COLOR_LIGHT_GRAY: u8 = 7;
pub const COLOR_DARK_GRAY: u8 = 8;
pub const COLOR_YELLOW: u8 = 14;
pub const COLOR_WHITE: u8 = 15;

// Tabla de registros estandar para VGA modo 13h (320x200, 256 colores)
// MISC(1) + SEQ(5) + CRTC(25) + GC(9) + AC(21) = 61 bytes
const MODE_13H_REGS: [u8; 61] = [
    // MISC
    0x63,
    // SEQ
    0x03, 0x01, 0x0F, 0x00, 0x0E,
    // CRTC
    0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F,
    0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x9C, 0x0E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3,
    0xFF,
    // GC
    0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F,
    0xFF,
    // AC
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x41, 0x00, 0x0F, 0x00, 0x00,
];

static mut FRAMEBUFFER_ADDR: usize = 0xA0000;

/// Debe llamarse una vez al inicio, pasando el offset de memoria fisica
/// que usa el bootloader (necesario porque la memoria fisica esta
/// mapeada a partir de ese offset en la tabla de paginas del kernel).
pub unsafe fn set_physical_memory_offset(offset: u64) {
    FRAMEBUFFER_ADDR = offset as usize + 0xA0000;
}

pub unsafe fn init_mode_13h() {
    let mut misc_port: Port<u8> = Port::new(0x3C2);
    misc_port.write(MODE_13H_REGS[0]);

    let mut seq_index: Port<u8> = Port::new(0x3C4);
    let mut seq_data: Port<u8> = Port::new(0x3C5);
    for i in 0..5u16 {
        seq_index.write(i as u8);
        seq_data.write(MODE_13H_REGS[1 + i as usize]);
    }

    let mut crtc_index: Port<u8> = Port::new(0x3D4);
    let mut crtc_data: Port<u8> = Port::new(0x3D5);

    // Desbloquear registros CRTC 0-7
    crtc_index.write(0x03);
    let mut tmp = crtc_data.read();
    crtc_data.write(tmp | 0x80);
    crtc_index.write(0x11);
    tmp = crtc_data.read();
    crtc_data.write(tmp & !0x80);

    let crtc_regs = &MODE_13H_REGS[6..6 + 25];
    for i in 0..25u16 {
        crtc_index.write(i as u8);
        crtc_data.write(crtc_regs[i as usize]);
    }

    let gc_regs = &MODE_13H_REGS[31..31 + 9];
    let mut gc_index: Port<u8> = Port::new(0x3CE);
    let mut gc_data: Port<u8> = Port::new(0x3CF);
    for i in 0..9u16 {
        gc_index.write(i as u8);
        gc_data.write(gc_regs[i as usize]);
    }

    let ac_regs = &MODE_13H_REGS[40..40 + 21];
    let mut ac_port: Port<u8> = Port::new(0x3C0);
    let mut input_status: Port<u8> = Port::new(0x3DA);
    let _ = input_status.read(); // resetea el flip-flop de direccion/dato
    for i in 0..21u16 {
        ac_port.write(i as u8);
        ac_port.write(ac_regs[i as usize]);
    }
    let _ = input_status.read();
    ac_port.write(0x20); // habilita salida de video
}

#[inline]
pub fn set_pixel(x: usize, y: usize, color: u8) {
    if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
        return;
    }
    unsafe {
        let addr = FRAMEBUFFER_ADDR + (y * SCREEN_WIDTH + x);
        write_volatile(addr as *mut u8, color);
    }
}

pub fn fill_screen(color: u8) {
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            set_pixel(x, y, color);
        }
    }
}

pub fn draw_rect(x: usize, y: usize, w: usize, h: usize, color: u8) {
    for row in y..(y + h).min(SCREEN_HEIGHT) {
        for col in x..(x + w).min(SCREEN_WIDTH) {
            set_pixel(col, row, color);
        }
    }
}

pub fn draw_rect_border(x: usize, y: usize, w: usize, h: usize, color: u8) {
    for col in x..(x + w).min(SCREEN_WIDTH) {
        set_pixel(col, y, color);
        set_pixel(col, (y + h).min(SCREEN_HEIGHT - 1), color);
    }
    for row in y..(y + h).min(SCREEN_HEIGHT) {
        set_pixel(x, row, color);
        set_pixel((x + w).min(SCREEN_WIDTH - 1), row, color);
    }
}

/// Dibuja el escritorio base de NeriOS: fondo, barra de tareas, boton e iconos.
pub fn draw_desktop() {
    fill_screen(COLOR_BLUE);

    // Barra de tareas
    draw_rect(0, 184, SCREEN_WIDTH, 16, COLOR_LIGHT_GRAY);

    // Boton "Start"
    draw_rect(2, 186, 40, 12, COLOR_GREEN);
    draw_rect_border(2, 186, 40, 12, COLOR_BLACK);

    // Un par de "iconos" en el escritorio (cuadros de color)
    draw_rect(10, 10, 24, 24, COLOR_YELLOW);
    draw_rect_border(10, 10, 24, 24, COLOR_BLACK);

    draw_rect(10, 44, 24, 24, COLOR_CYAN);
    draw_rect_border(10, 44, 24, 24, COLOR_BLACK);

    draw_rect(10, 78, 24, 24, COLOR_WHITE);
    draw_rect_border(10, 78, 24, 24, COLOR_BLACK);
}