use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{print, println};

lazy_static! {
    static ref INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());
}

pub fn handle_byte(byte: u8) {
    match byte {
        b'\r' | b'\n' => {
            let mut buffer = INPUT_BUFFER.lock();
            println!();
            execute_command(&buffer);
            buffer.clear();
            print!("NeriOS> ");
        }
        0x7f | 0x08 => {
            // backspace/delete
            let mut buffer = INPUT_BUFFER.lock();
            buffer.pop();
            print!("\u{8} \u{8}");
        }
        c if c.is_ascii_graphic() || c == b' ' => {
            let mut buffer = INPUT_BUFFER.lock();
            buffer.push(c as char);
            print!("{}", c as char);
        }
        _ => {}
    }
}

fn execute_command(cmd: &str) {
    match cmd.trim() {
        "help" => {
            println!("Comandos disponibles:");
            println!("  help  - muestra esta ayuda");
            println!("  about - informacion de NeriOS");
        }
        "about" => {
            println!("NeriOS - Mini sistema operativo bare-metal");
            println!("Desarrollado por NeriSoft Dev (Ing. Eduardo Neri)");
            println!("Escrito en Rust para x86_64");
        }
        "" => {}
        other => {
            println!("Comando desconocido: '{}'. Escribe 'help' para ver comandos.", other);
        }
    }
}

pub fn init() {
    print!("NeriOS> ");
}