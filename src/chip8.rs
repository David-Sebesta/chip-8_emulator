use rand;


const MEMORY_SIZE: usize = 4096;
const PROGRAM_START_ADDR: u16 = 0x200;
const STACK_SIZE: usize = 16;
const GENERAL_REGISTER_COUNT: usize = 16;

const DEFAULT_FONT_ADDR: usize = 0x50;
const DEFUALT_FONT: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
                                0xF0, 0x80, 0xF0, 0x80, 0x80,];  // F
                                  

#[derive(PartialEq, Debug)]
enum Instruction {
    SYS(u16),        // 0x0nnn 
    CLS,             // 0x00E0 - Clear Display
    RET,             // 0x00EE - Return 
    JP(u16),         // 0x1nnn - Jump to nnn
    CALL(u16),       // 0x2nnn - Call nnn
    SEByte(u8, u8),  // 0x3xkk - Skip next instruction if Vx == kk
    SNEByte(u8, u8), // 0x4xkk - Skip next instruction if Vx != kk
    SEReg(u8, u8),   // 0x5xy0 - Skip next instruction if Vx == Vy
    LDByte(u8, u8),  // 0x6xkk - Vx = kk
    ADDByte(u8, u8), // 0x7xkk - Vx = Vx + kk
    LDReg(u8, u8),   // 0x8xy0 - Vx = Vy
    OR(u8, u8),      // 0x8xy1 - Vx = Vx OR Vy
    AND(u8, u8),     // 0x8xy2 - Vx = Vx AND Vy
    XOR(u8, u8),     // 0x8xy3 - Vx = Vx XOR Vy
    ADDReg(u8, u8),  // 0x8xy4 - Vx = Vx + Vy, Set VF = Carry
    SUBReg(u8, u8),  // 0x8xy5 - Vx = Vx - Vy, Set VF = !borrow
    SHR(u8, u8),     // 0x8xy6 - Vx = Vx Shift Right 1, If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    SUBN(u8, u8),    // 0x8xy7 - Vx = Vy - Vx, Set VF = !borrow
    SHL(u8, u8),     // 0x8xyE - Vx = Vx Shift Left 1, If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    SNEReg(u8, u8),  // 0x9xy0 Skip next instruction if Vx != Vy
    LDI(u16),        // 0xAnnn - I = nnn
    JPV0(u16),       // 0xBnnn - PC = nnn + V0
    RND(u8, u8),     // 0xCxkk - Vx = Rand byte AND kk
    DRW(u8, u8, u8), // 0xDxyn - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    SKP(u8),         // 0xEx9E - Skip next instruction if key with the value of Vx is pressed
    SKNP(u8),        // 0xExA1 - Skip next instruction if key with the value of Vx is not pressed
    LDVxDT(u8),      // 0xFx07 - Vx = Delay timer value
    LDVxK(u8),       // 0xFx0A - Wait for key press, Vx = key press
    LDDTVx(u8),      // 0xFx15 - Delay timer = Vx
    LDSTVx(u8),      // 0xFx18 - Sound timer = Vx
    ADDI(u8),        // 0xFx1E - I = I + Vx
    LDF(u8),         // 0xFx29 - I = location of sprite for digit Vx
    LDB(u8),         // 0xFx33 - Store BCB representation of Vx in I, I+1, I+2
    LDStore(u8),     // 0xFx55 - Store registers V0..Vx in memory starting at location I
    LDLoad(u8),      // 0xFx65 - Read registers V0..Vx from memory starting at location I
    Unknown(u16),
}


pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    v: [u8; GENERAL_REGISTER_COUNT], // General registers V0-VF
    i: u16,  // Increment register
    pc: u16, // Program Counter
    stack: [u16; STACK_SIZE], 
    sp: u8,  // Stack pointer
    dt: u8, // Delay timer register
    st: u8, // Sound timer register
    pub display: [bool; 64*32], // Pixel buffer
    pub keys: [bool; 16], // Keys pressed
}

impl Chip8 {
    pub fn new() -> Self {
        let mut cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; GENERAL_REGISTER_COUNT],
            i: 0,
            pc: PROGRAM_START_ADDR,
            stack: [0; STACK_SIZE],
            sp: 0, 
            dt: 0,
            st: 0,
            display: [false; 64*32],
            keys: [false; 16],
        };

        cpu.load_font(DEFAULT_FONT_ADDR);

        cpu
    }

    pub fn load_font(&mut self, location: usize) {
        let start = location;
        let end = location + DEFUALT_FONT.len();
        self.memory[start..end].copy_from_slice(&DEFUALT_FONT);
    }

    pub fn load_rom(&mut self, data: &[u8]) -> bool {
        let start = PROGRAM_START_ADDR as usize;
        let end = start + data.len();
        // Don't load if greater than memory
        if end <= MEMORY_SIZE {
            self.memory[start..end].copy_from_slice(data);
            true
        } else {
            print!("Data size {:?} is larger than memory size {:?}", data.len(), MEMORY_SIZE);
            false
        }
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        let instruction = self.decode(op);
        self.execute(instruction);

    }

    fn fetch(&mut self) -> u16 {
        // Fetch
        let op_byte1 = self.memory[self.pc as usize] as u8;
        let op_byte2 = self.memory[(self.pc + 1) as usize] as u8;
        let op = ((op_byte1 as u16) << 8) | op_byte2 as u16;

        // Increment program counter by 2
        self.pc += 2;
        op
    }

    fn decode(&self, op: u16) -> Instruction {
        // Decode opcode value
        let n1 = ((op & 0xF000) >> 12) as u8;
        let x = ((op & 0x0F00) >> 8) as u8;    // X
        let y = ((op & 0x00F0) >> 4) as u8;    // Y
        let n4 = (op & 0x000F) as u8;

        let nnn = op & 0x0fff;
        let kk: u8 = (op & 0xff) as u8;

        match (n1, x, y, n4) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::CLS,
            (0x0, 0x0, 0xE, 0xE) => Instruction::RET,
            (0x0, _, _, _)       => Instruction::SYS(nnn),
            (0x1, _, _, _)       => Instruction::JP(nnn),
            (0x2, _, _, _)       => Instruction::CALL(nnn),
            (0x3, _, _, _)       => Instruction::SEByte(x, kk),
            (0x4, _, _, _)       => Instruction::SNEByte(x, kk),
            (0x5, _, _, 0x0)     => Instruction::SEReg(x, y),
            (0x6, _, _, _)       => Instruction::LDByte(x, kk),
            (0x7, _, _, _)       => Instruction::ADDByte(x, kk),
            (0x8, _, _, 0x0)     => Instruction::LDReg(x, y),   
            (0x8, _, _, 0x1)     => Instruction::OR(x, y),      
            (0x8, _, _, 0x2)     => Instruction::AND(x, y),     
            (0x8, _, _, 0x3)     => Instruction::XOR(x, y),     
            (0x8, _, _, 0x4)     => Instruction::ADDReg(x, y),  
            (0x8, _, _, 0x5)     => Instruction::SUBReg(x, y),  
            (0x8, _, _, 0x6)     => Instruction::SHR(x, y),     
            (0x8, _, _, 0x7)     => Instruction::SUBN(x, y),    
            (0x8, _, _, 0xE)     => Instruction::SHL(x, y),     
            (0x9, _, _, 0x0)     => Instruction::SNEReg(x, y),  
            (0xA, _, _, _)       => Instruction::LDI(nnn),        
            (0xB, _, _, _)       => Instruction::JPV0(nnn),       
            (0xC, _, _, _)       => Instruction::RND(x, kk),     
            (0xD, _, _, _)       => Instruction::DRW(x, y, n4), 
            (0xE, _, 0x9, 0xE)   => Instruction::SKP(x),         
            (0xE, _, 0xA, 0x1)   => Instruction::SKNP(x),        
            (0xF, _, 0x0, 0x7)   => Instruction::LDVxDT(x),      
            (0xF, _, 0x0, 0xA)   => Instruction::LDVxK(x),       
            (0xF, _, 0x1, 0x5)   => Instruction::LDDTVx(x),      
            (0xF, _, 0x1, 0x8)   => Instruction::LDSTVx(x),      
            (0xF, _, 0x1, 0xE)   => Instruction::ADDI(x),        
            (0xF, _, 0x2, 0x9)   => Instruction::LDF(x),         
            (0xF, _, 0x3, 0x3)   => Instruction::LDB(x),         
            (0xF, _, 0x5, 0x5)   => Instruction::LDStore(x),     
            (0xF, _, 0x6, 0x5)   => Instruction::LDLoad(x),      

            _ => Instruction::Unknown(op),
        }


    }

    fn execute(&mut self, instruction: Instruction) {
        
        match instruction {
            Instruction::CLS => {
                self.display.fill(false);
            },
            Instruction::RET => {
                if self.sp > 0 {
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
            },
            Instruction::JP(nnn) => {
                self.pc = nnn;
            },
            Instruction::CALL(nnn) => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            },
            Instruction::SEByte(x, kk) => {
                if self.v[x as usize] == kk {
                    self.pc += 2;
                }
            },
            Instruction::SNEByte(x, kk) => {
                if self.v[x as usize] != kk {
                    self.pc += 2;
                }
            },
            Instruction::SEReg(x, y) => {
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            },
            Instruction::LDByte(x, kk) => {
                self.v[x as usize] = kk;
            },
            Instruction::ADDByte(x, kk) => {
                self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
            },
            Instruction::LDReg(x, y) => {
                self.v[x as usize] = self.v[y as usize];
            },
            Instruction::OR(x, y) => {
                self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
            },
            Instruction::AND(x, y) => {
                self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
            },
            Instruction::XOR(x, y) => {
                self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
            },
            Instruction::ADDReg(x, y) => {
                let value: u16 = self.v[x as usize] as u16 + self.v[y as usize] as u16;
                self.v[0xF] = (value > u8::MAX as u16) as u8;
                self.v[x as usize] = value as u8;
            },
            Instruction::SUBReg(x, y) => {
                let vx = self.v[x as usize];
                let vy = self.v[y as usize];
                self.v[0xF] = (vx >= vy) as u8;
                self.v[x as usize] = vx.wrapping_sub(vy);
            },
            Instruction::SHR(x, _y) => {
                let vx = self.v[x as usize];
                self.v[0xF] = vx & 0x01;
                self.v[x as usize] = vx >> 1;
            },
            Instruction::SUBN(x, y) => {
                let vx = self.v[x as usize];
                let vy = self.v[y as usize];
                self.v[0xF] = (vy >= vx) as u8;
                self.v[x as usize] = vy.wrapping_sub(vx);
            },
            Instruction::SHL(x, _y) => {
                let vx = self.v[x as usize];
                self.v[0xF] = (vx >> 7) & 1;
                self.v[x as usize] = vx << 1;
            },
            Instruction::SNEReg(x, y) => {
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            },
            Instruction::LDI(nnn) => {
                self.i = nnn;
            },
            Instruction::JPV0(nnn) => {
                self.pc = nnn + self.v[0x0] as u16;
            },
            Instruction::RND(x, kk) => {
                let random_u8: u8 = rand::random();
                self.v[x as usize] = random_u8 & kk; 
            },
            Instruction::DRW(x, y, n) => {
                let x_pos = self.v[x as usize] as usize % 64;
                let y_pos = self.v[y as usize] as usize % 32;
                // Reset collision flag
                self.v[0xF] = 0;

                for row in 0..n {
                    let sprite_byte = self.memory[(self.i + row as u16) as usize];
                    for col in 0..8 {
                        let sprite_pixel = sprite_byte >> (7 - col) & 1;
                        if sprite_pixel == 1 {
                            let dx = (x_pos + col) % 64;
                            let dy = (y_pos + row as usize) % 32;
                            let index = dx + (dy * 64);

                            // Collision
                            if self.display[index] {
                                self.v[0xF] = 1;
                            }
                            self.display[index] ^= true;
                        }
                    }
                }
            },
            Instruction::SKP(x) => {
                if self.keys[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            },
            Instruction::SKNP(x) => {
                if !self.keys[self.v[x as usize] as usize] {
                    self.pc += 2;
                }
            },
            Instruction::LDVxDT(x) => {
                self.v[x as usize] = self.dt;
            },
            Instruction::LDVxK(x) => {
                let mut key_pressed = false;
                for key in 0..self.keys.len() {
                    if self.keys[key] {
                        self.v[x as usize] = key as u8;
                        key_pressed = true;
                        break;
                    }
                }

                // Go back an instruction instead of pausing
                if !key_pressed {
                    self.pc -= 2;
                }
            },
            Instruction::LDDTVx(x) => {
                self.dt = self.v[x as usize];
            },
            Instruction::LDSTVx(x) => {
                self.st = self.v[x as usize];
            },
            Instruction::ADDI(x) => {
                self.i += self.v[x as usize] as u16;
            },
            Instruction::LDF(x) => {
                self.i = DEFAULT_FONT_ADDR as u16 + (5 * self.v[x as usize]) as u16;
            },
            Instruction::LDB(x) => {
                if (self.i as usize) + 2 < MEMORY_SIZE {
                    let value = self.v[x as usize];
                    self.memory[self.i as usize] = value / 100;
                    self.memory[(self.i + 1) as usize] = (value / 10) % 10;
                    self.memory[(self.i + 2) as usize] = value % 10;
                }
            }
            Instruction::LDStore(x) => {
                let start = self.i as usize;
                let end = start + (x as usize) + 1;
                if end <= MEMORY_SIZE {
                    self.memory[start..end].copy_from_slice(&self.v[0..=(x as usize)]);
                }
            },
            Instruction::LDLoad(x) => {
                let start = self.i as usize;
                let end = start + (x as usize) + 1;
                if end <= MEMORY_SIZE {
                    self.v[0..=(x as usize)].copy_from_slice(&self.memory[start..end]);
                }
            },
            Instruction::Unknown(_op) => {
                println!("Opcode: {:?} not implemented yet", instruction);
            }
            _ => {
                println!("Opcode: {:?} not implemented yet", instruction);
            }
        }

        
    }

    pub fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn render_console(&self) {
        // Clear the terminal screen (ANSI escape code)
        print!("{}[2J", 27 as char);
        
        for y in 0..32 {
            for x in 0..64 {
                let index = x + (y * 64);
                if self.display[index] {
                    print!("█"); // Or "#"
                } else {
                    print!(" ");
                }
            }
            println!(); // New line after each row
        }
    }

    pub fn registers(&self) -> &[u8; 16] { &self.v }
    pub fn pc(&self) -> u16 { self.pc }
    pub fn i(&self) -> u16 { self.i }
    pub fn sp(&self) -> u8 { self.sp }
    pub fn dt(&self) -> u8 { self.dt }
    pub fn st(&self) -> u8 { self.st }

    pub fn reset(&mut self) {
        self.v = [0; GENERAL_REGISTER_COUNT];
        self.i = 0;
        self.pc = PROGRAM_START_ADDR;
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
        self.display.fill(false);
    }

}

#[test]
fn basic_cpu_test() {
    let mut test_chip8 = Chip8::new();
    
    let test_rom: [u8; 6] = [0x00, 0xE0, 0x00, 0xEE, 0x12, 0x00];
    test_chip8.load_rom(&test_rom);

    let op_list: [u16; 3] = [0x00E0, 0x00EE, 0x1200];
    let instruction_list = [Instruction::CLS, Instruction::RET, Instruction::JP(0x200)];
    
    for i in 0..3 {
        let op = test_chip8.fetch();
        assert_eq!(op, op_list[i]);
        
        let instruction = test_chip8.decode(op);
        assert_eq!(instruction, instruction_list[i]);
    }
}

#[test]
fn fetch_test() {
    let mut chip8 = Chip8::new();

    let rom: [u8; 8] = [0x60, 0x05, 0x70, 0x01, 0x30, 0x0F, 0x12, 0x02];
    chip8.load_rom(&rom);

    let op_list: [u16; 4] = [0x6005, 0x7001, 0x300F, 0x1202];

    for i in 0..4 {
        assert_eq!(chip8.fetch(), op_list[i]);
    }
}

#[test]
fn decode_test() {
    let mut chip8 = Chip8::new();

    let op_list: [u16; 4] = [0x6005, 0x7001, 0x300F, 0x1202];
    let instruction_list: [Instruction; 4] = [Instruction::LDByte(0x0, 0x5), Instruction::ADDByte(0x0, 0x01), Instruction::SEByte(0x0, 0x0F), Instruction::JP(0x202)];

    for i in 0..4 {
        assert_eq!(chip8.decode(op_list[i]), instruction_list[i]);
    }
}

#[test]
fn execute_test() {
    let mut chip8 = Chip8::new();

    let rom: [u8; 8] = [0x60, 0x05, 0x70, 0x01, 0x30, 0x0F, 0x12, 0x02];
    chip8.load_rom(&rom);

    let mut count = 0;
    loop {
        chip8.tick();
        count += 1;
        if chip8.fetch() == 0x0000 {
            break;
        } else {
            // Fetch goes forward so we need to go back
            chip8.pc -= 2;
        }
    }

    assert_eq!(count, 30);
}