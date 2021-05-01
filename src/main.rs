extern crate sdl2;

use sdl2::event::Event;

use crate::cpu::CPU;
use crate::display::Display;


mod cpu;
mod display;
mod keypad;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let display = Display::new(sdl_context.to_owned());

    let mut cpu = CPU::new(display);
    
    cpu.load_game("games/UFO".to_string());

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => cpu.keypad.press(key, true),
                Event::KeyUp {
                    keycode: Some(key), ..
                } => cpu.keypad.press(key, false),
                _ => {}
            }
        }

        cpu.emulate_cycle();
        cpu.display.draw_screen();
    }

}
