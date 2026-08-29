use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{print, println, vga_buffer::WRITER};

lazy_static! {
    static ref INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());
}

pub fn handle_char(character: char) {
    match character {
        '\n' => {
            let mut buffer = INPUT_BUFFER.lock();
            println!();
            execute_command(&buffer);
            buffer.clear();
            print!("NeriOS> ");
        }
        '\u{8}' => {
            // backspace
            let mut buffer = INPUT_BUFFER.lock();
            if buffer.pop().is_some() {
                print!("{}", 0x08 as char);
            }
        }
        c => {
            let mut buffer = INPUT_BUFFER.lock();
            buffer.push(c);
            print!("{}", c);
        }
    }
}

fn execute_command(cmd: &str) {
    match cmd.trim() {
        "help" => {
            println!("Comandos disponibles:");
            println!("  help  - muestra esta ayuda");
            println!("  about - informacion de NeriOS");
            println!("  clear - limpia la pantalla");
        }
        "about" => {
            println!("NeriOS - Mini sistema operativo bare-metal");
            println!("Desarrollado por NeriSoft Dev (Ing. Eduardo Neri)");
            println!("Escrito en Rust para x86_64");
        }
        "clear" => {
            WRITER.lock().clear_screen();
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