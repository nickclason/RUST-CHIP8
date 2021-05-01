use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::display::Display;
use crate::keypad::Keypad;

extern crate rand;


pub struct CPU {
    pc: usize,           // Program counter
    i: usize,            // Index Register
    vx: [u8; 16],        // V Registers (V0-VF)
    memory: [u8; 4096],  // RAM
    stack: [u16; 16],    // Stack
    opcode: u16,         // Current opcode
    sp: usize,           // Stack pointer
    delay_timer: u8,     // Delay timer
    sound_timer: u8,     // Sound timer
    pub keypad: Keypad,  // Keypad
    pub display: Display // Display
}

impl CPU {
    pub fn new(display: Display) -> CPU {
        let mut cpu = CPU {
            pc: 0x200, // Program start
            i: 0x200,
            vx: [0; 16],
            memory: [0; 4096],
            stack: [0; 16],
            opcode: 0,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: Keypad::new(),
            display: display
        };

        for i in 0..80 {
            cpu.memory[i] = FONT_SET[i];
        }

        cpu
    }

    pub fn load_game(&mut self, game: String) {
        println!("Loading Game...\n");
       
        let path = Path::new(&game);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut buffer = [0; 3584];
        match file.read(&mut buffer) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => {  } // file read successfully
            }

        for op in &buffer
        {   
            if self.pc < 4096 {
                self.memory[self.pc] = *op;
                self.pc += 1;
            }
            else {
                self.pc = 0x200;
                break;
            }
        }
        self.pc = 0x200; // reset to beginning of memory (512)
    }

    fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
        // println!("DEBUG OPCODE: {}", self.opcode);
    }

    fn execute_opcode(&mut self) {
        match self.opcode & 0xf000 {
            0x0000 => self.op_0xxx(),
            0x1000 => self.op_1xxx(),
            0x2000 => self.op_2xxx(),
            0x3000 => self.op_3xxx(),
            0x4000 => self.op_4xxx(),
            0x5000 => self.op_5xxx(),
            0x6000 => self.op_6xxx(),
            0x7000 => self.op_7xxx(),
            0x8000 => self.op_8xxx(),
            0x9000 => self.op_9xxx(),
            0xA000 => self.op_Axxx(),
            0xB000 => self.op_Bxxx(),
            0xC000 => self.op_Cxxx(),
            0xD000 => self.op_Dxxx(),
            0xE000 => self.op_Exxx(),
            0xF000 => self.op_Fxxx(),
            _      => println!("Unknown opcode, op:{} pc:{}", self.opcode as usize, self.pc)
        }
    }

    fn op_0xxx(&mut self) {
        match self.opcode & 0x000F {
            0x0000 => { self.display.clear() }
            0x000E => {
                self.sp -=1;
                self.pc = self.stack[self.sp] as usize;
            }
            _      => { println!("Unknown opcode, op:{} pc:{}", self.opcode as usize, self.pc) }
        }
        
        self.pc += 2;
    }

    fn op_1xxx(&mut self) {
        self.pc = self.op_nnn() as usize;
    }

    fn op_2xxx(&mut self) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc =  self.op_nnn() as usize;
    }

    fn op_3xxx(&mut self) {
        if self.vx[self.op_x()] == self.op_nn()
        {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_4xxx(&mut self) {
        if self.vx[self.op_x()] != self.op_nn()
        {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_5xxx(&mut self) {
        if self.vx[self.op_x()] == self.vx[self.op_y()] {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_6xxx(&mut self) {
        self.vx[self.op_x()] = self.op_nn();
        self.pc += 2;
    }

    fn op_7xxx(&mut self) {
        let v: u8 = self.vx[self.op_x()];
        self.vx[self.op_x()] = v.wrapping_add(self.op_nn());
        self.pc += 2;
    }

    fn op_8xxx(&mut self) {
        let v: u8 = self.vx[self.op_x()];
        let vy: u8 = self.vx[self.op_y()];
        match self.opcode & 0x000F {
            0 => {
                self.vx[self.op_x()] = vy;
            }
            1 => {
                self.vx[self.op_x()] |= vy;
            }
            2 => {
                self.vx[self.op_x()] &= vy;
            }
            3 => {
                self.vx[self.op_x()] ^= vy;
            }
            4 => {
                // self.v[self.op_x()] += self.v[self.op_y()];
                self.vx[15] = if (v as u16 + vy as u16) > 0xFF { 1 } else { 0 };
                self.vx[self.op_x()] = v.wrapping_add(vy);
            }
            5 => {
                self.vx[15] = if vy > v { 0 } else { 1 };
                self.vx[self.op_x()] = v.wrapping_sub(vy);
            }
            6 => {
                self.vx[15] = v & 0x1;
                self.vx[self.op_x()] >>= 1;
            }
            7 => {
                self.vx[15] = if vy > v { 0 } else { 1 };
                self.vx[self.op_x()] = vy.wrapping_sub(v);
            }
            0xE => {
                self.vx[15] = self.vx[self.op_x()] >> 7;
                self.vx[self.op_x()] <<= 1;
            }
            _ => println!("Unknown opcode, op: {}", self.opcode),
        }
        self.pc += 2;
    }

    fn op_9xxx(&mut self) {
        if self.vx[self.op_x()] != self.vx[self.op_y()]
        {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn op_Axxx(&mut self) {
        self.i = self.op_nnn() as usize;
        self.pc += 2;
    }

    fn op_Bxxx(&mut self) {
        self.pc = (self.op_nnn() + (self.vx[0] as u16)) as usize;
    }

    fn op_Cxxx(&mut self) {
        self.vx[self.op_x()] = self.op_nn() & rand::random::<u8>();
        self.pc += 2;
    }

    fn op_Dxxx(&mut self) {
        let from = self.i;
        let to = from +(self.op_n() as usize);
        let x = self.vx[self.op_x()];
        let y = self.vx[self.op_y()];
        // self.vx[15] = self.display.draw(x as usize, y as usize, self.memory.slice(from, to));
        self.vx[15] = self.display.draw(x as usize, y as usize, &self.memory[from .. to]);
        self.pc += 2;
    }

    fn op_Exxx(&mut self) {
        let v = self.vx[self.op_x()] as usize;
        self.pc += match self.opcode & 0x00FF {
            0x9E => if self.keypad.pressed(v) { 4 } else { 2 },
            0xA1 => if !self.keypad.pressed(v) { 4 } else { 2 },
            _    => 2
        }
    }

    fn op_Fxxx(&mut self) {
        match self.opcode & 0x00FF {
            0x07 => { self.vx[self.op_x()] = self.delay_timer; }
            0x0A => { self.wait_keypress(); }
            0x15 => { self.delay_timer = self.vx[self.op_x()]; }
            0x18 => { self.sound_timer = self.vx[self.op_x()]; }
            0x1E => { self.i += self.vx[self.op_x()] as usize; }
            0x29 => { self.i = (self.vx[self.op_x()] as usize) * 5; }
            0x33 => {
                self.memory[self.i] = self.vx[self.op_x()] / 100;
                self.memory[self.i + 1] = (self.vx[self.op_x()] / 10) % 10;
                self.memory[self.i + 2] = (self.vx[self.op_x()] % 100) % 10;
            }
            0x55 => {
                for i in 0..(self.op_x() + 1) {
                    self.memory[self.i + i] = self.vx[i]
                }
                self.i += self.op_x() + 1;
            }
            0x65 => {
                for i in 0..(self.op_x() + 1) {
                    self.vx[i] = self.memory[self.i + i]
                }
                self.i += self.op_x() + 1;
            }
            _    => { println!("Unknown opcode, op: {}", self.opcode) }
        }
        self.pc += 2;
    }

    pub fn emulate_cycle(&mut self) {
        self.fetch_opcode();
        self.execute_opcode();
        
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // println!("BEEP!\n");
                self.sound_timer -= 1;
            }
        }

        for i in 0..10000 { };

    }

    fn op_x(&self)   -> usize { ((self.opcode & 0x0F00) >> 8) as usize }
    fn op_y(&self)   -> usize { ((self.opcode & 0x00F0) >> 4) as usize }
    fn op_n(&self)   -> u8 { (self.opcode & 0x000F) as u8 }
    fn op_nn(&self)  -> u8 { (self.opcode & 0x00FF) as u8 }
    fn op_nnn(&self) -> u16 { self.opcode & 0x0FFF }

    fn wait_keypress(&mut self) {
        for i in 0u8..16 {
            if self.keypad.pressed(i as usize) {
                self.vx[self.op_x()] = i;
                break;
            }
        }
        self.pc -= 2;
    }
}

pub const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F 
    ];