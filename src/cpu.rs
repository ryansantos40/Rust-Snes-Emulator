use crate::memory::Memory;

pub struct Cpu {

    // Registers
    pub a: u16, //Accumulator
    pub x: u16,     //  Regsiter X
    pub y: u16,     // Register Y
    pub sp: u16,    // Stack Pointer
    pub pc: u32,    // Program Counter
    pub dp: u16,    // Direct Page Register
    pub db: u8,     // Data Bank Register
    pub pb: u8,     // Program Bank Register

    // Status Register
    pub p: u8,

    // Mode flags (Emulation/Native, 0 - 16-bit, 1 - 8-bit)
    pub m_flag: bool, // Accumulator/Memory Size Flag
    pub x_flag: bool, // Index Register Size Flag
    pub e_flag: bool, // Emulation Mode Flag

    pub cycles: u64, // Cycle count
}

#[allow(dead_code)]
impl Cpu {
   pub const FLAG_CARRY: u8 = 0x01;
   pub const FLAG_ZERO: u8 = 0x02;
   pub const FLAG_IRQ: u8 = 0x04;
   pub const FLAG_DECIMAL: u8 = 0x08;
   pub const FLAG_INDEX: u8 = 0x10;
   pub const FLAG_MEMORY: u8 = 0x20;
   pub const FLAG_OVERFLOW: u8 = 0x40;
   pub const FLAG_NEGATIVE: u8 = 0x80;
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0x0000,
            x: 0x0000,
            y: 0x0000,
            sp: 0x01FF, // Stack Pointer starts at 0x1FF in Emulation Mode
            pc: 0x008000,
            dp: 0x0000,
            db: 0x00,
            pb: 0x00,
            p: 0x34, // Default status register value in Emulation Mode
            m_flag: true, // Start in Emulation Mode (8-bit accumulator)
            x_flag: true, // Start in Emulation Mode (8-bit index registers)
            e_flag: true, // Start in Emulation Mode
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0x0000;
        self.x = 0x0000;
        self.y = 0x0000;
        self.pc = 0x008000;
        self.sp = 0x01FF;
        self.dp = 0x0000;
        self.db = 0x00;
        self.pb = 0x00;
        self.p = 0x34;
        self.m_flag = true;
        self.x_flag = true;
        self.e_flag = true;
        self.cycles = 0;
    }

    pub fn step(&mut self, memory: &mut Memory) -> u8 {
        let opcode = memory.read(self.pc);
        self.pc += 1;

        let cycles = self.execute_instruction(opcode, memory);
        self.cycles += cycles as u64;
        cycles
    }

    fn execute_instruction(&mut self, opcode: u8, memory: &mut Memory) -> u8 {
        match opcode {
            // NOP - No Operation
            0xEA => 2,

            //LDA - Load Accumulator
            0xA9 => self.lda_immediate(memory),
            0xA5 => self.lda_direct_page(memory),
            0xAD => self.lda_absolute(memory),

            // lDX - Load X Register
            0xA2 => self.ldx_immediate(memory),
            0xA6 => self.ldx_direct_page(memory),
            0xAE => self.ldx_absolute(memory),

            // LDY - Load Y Register
            0xA0 => self.ldy_immediate(memory),
            0xA4 => self.ldy_direct_page(memory),
            0xAC => self.ldy_absolute(memory),

            // STA - Store Accumulator
            0x85 => self.sta_direct_page(memory),
            0x8D => self.sta_absolute(memory),

            // STX - Store X Register
            0x86 => self.stx_direct_page(memory),
            0x8E => self.stx_absolute(memory),

            // STY - Store Y Register
            0x84 => self.sty_direct_page(memory),
            0x8C => self.sty_absolute(memory),

            // Flags Operations
            0x18 => { self.clear_flag(Self::FLAG_CARRY); 2 },
            0x38 => { self.set_flag(Self::FLAG_CARRY); 2 },
            0x58 => { self.clear_flag(Self::FLAG_IRQ); 2 },
            0x78 => { self.set_flag(Self::FLAG_IRQ); 2 },
            0xB8 => { self.clear_flag(Self::FLAG_OVERFLOW); 2 },
            0xD8 => { self.clear_flag(Self::FLAG_DECIMAL); 2 },
            0xF8 => { self.set_flag(Self::FLAG_DECIMAL); 2 },

            // Jumps
            0x4C => self.jmp_absolute(memory),
            0x6C => self.jmp_indirect(memory),

            // Branch Instructions
            0x10 => self.branch(!self.get_flag(Self::FLAG_NEGATIVE), memory), // BPL
            0x30 => self.branch(self.get_flag(Self::FLAG_NEGATIVE), memory),  // BMI
            0x50 => self.branch(!self.get_flag(Self::FLAG_OVERFLOW), memory), // BVC
            0x70 => self.branch(self.get_flag(Self::FLAG_OVERFLOW), memory),  // BVS
            0x90 => self.branch(!self.get_flag(Self::FLAG_CARRY), memory),    // BCC
            0xB0 => self.branch(self.get_flag(Self::FLAG_CARRY), memory),     // BCS
            0xD0 => self.branch(!self.get_flag(Self::FLAG_ZERO), memory),     // BNE
            0xF0 => self.branch(self.get_flag(Self::FLAG_ZERO), memory),      // BEQ

            _ => {
                println!("Opcode não implementado: {:02X} em PC: {:06X}", opcode, self.pc - 1);
                2
            }
        }
    }

    // ++++ Load Instructions ++++

    fn lda_immediate(&mut self, memory: &mut Memory) -> u8 {
        if self.m_flag {
            let value = memory.read(self.pc) as u16;
            self.pc += 1;
            self.a = (self.a & 0xFF00) | value;
            self.update_nz_flags_a();
            2

        } else {
            let low = memory.read(self.pc) as u16;
            let high = memory.read(self.pc + 1) as u16;
            self.pc += 2;
            self.a = (high << 8) | low;
            self.update_nz_flags_a();
            3
        }
    }

    fn lda_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.m_flag {
            self.a = (self.a & 0xFF00) | memory.read(addr as u32) as u16;

        } else {
            let low = memory.read(addr as u32) as u16;
            let high = memory.read((addr + 1) as u32) as u16;
            self.a = (high << 8) | low;
        }
        self.update_nz_flags_a();
        if self.m_flag { 3 } else { 4 }
    }

    fn lda_absolute(&mut self, memory : &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.m_flag {
            self.a = (self.a & 0xFF00) | memory.read(addr) as u16;

        } else {
            let low = memory.read(addr) as u16;
            let high = memory.read(addr + 1) as u16;
            self.a = (high << 8) | low;
        }

        self.update_nz_flags_a();
        if self.m_flag { 4 } else { 5 }

    }

    fn ldx_immediate(&mut self, memory: &mut Memory) -> u8 {
        if self.x_flag {
            self.x = (self.x & 0xFF00) | memory.read(self.pc) as u16;
            self.pc += 1;
            self.update_nz_flags_x();
            2

        } else {
            let low = memory.read(self.pc) as u16;
            let high = memory.read(self.pc + 1) as u16;
            self.pc += 2;
            self.x = (high << 8) | low;
            self.update_nz_flags_x();
            3
        }
    }

    fn ldx_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.x_flag {
            self.x = (self.x & 0xFF00) | memory.read(addr as u32) as u16;

        } else {
            let low = memory.read(addr as u32) as u16;
            let high = memory.read((addr + 1) as u32) as u16;
            self.x = (high << 8) | low;

        }

        self.update_nz_flags_x();
        if self.x_flag { 3 } else { 4 }
    }

    fn ldx_absolute(&mut self, memory: &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.x_flag {
            self.x = (self.x & 0xFF00) | memory.read(addr) as u16;

        } else {
            let low = memory.read(addr) as u16;
            let high = memory.read(addr + 1) as u16;
            self.x = (high << 8) | low;

        }

        self.update_nz_flags_x();
        if self.x_flag { 4 } else { 5 }

    }

    fn ldy_immediate(&mut self, memory: &mut Memory) -> u8 {
        if self.x_flag {
            self.y = (self.y & 0xFF00) | memory.read(self.pc) as u16;
            self.pc += 1;
            self.update_nz_flags_y();
            2

        } else {
            let low = memory.read(self.pc) as u16;
            let high = memory.read(self.pc + 1) as u16;
            self.pc += 2;
            self.y = (high << 8) | low;
            self.update_nz_flags_y();
            3
        }
    }

    fn ldy_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.x_flag {
            self.y = (self.y & 0xFF00) | memory.read(addr as u32) as u16;

        } else {
            let low = memory.read(addr as u32) as u16;
            let high = memory.read((addr + 1) as u32) as u16;
            self.y = (high << 8) | low;

        }

        self.update_nz_flags_y();
        if self.x_flag { 3 } else { 4 }
    }

    fn ldy_absolute(&mut self, memory: &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.x_flag {
            self.y = (self.y & 0xFF00) | memory.read(addr) as u16;

        } else {
            let low = memory.read(addr) as u16;
            let high = memory.read(addr + 1) as u16;
            self.y = (high << 8) | low;
        }

        self.update_nz_flags_y();
        if self.x_flag { 4 } else { 5 }
    }

    // ++++ Store Instructions ++++

    fn sta_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.m_flag{
            memory.write(addr as u32, self.a as u8);

        } else {
            memory.write(addr as u32, self.a as u8);
            memory.write((addr + 1) as u32, (self.a >> 8) as u8);

        }

        if self.m_flag { 3 } else { 4 }
    }

    fn sta_absolute(&mut self, memory: &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.m_flag{
            memory.write(addr, self.a as u8);

        } else {
            memory.write(addr, self.a as u8);
            memory.write(addr + 1, (self.a >> 8) as u8);

        }

        if self.m_flag { 4 } else { 5 }
    }

    fn stx_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.x_flag {
            memory.write(addr as u32, self.x as u8);

        } else {
            memory.write(addr as u32, self.x as u8);
            memory.write((addr + 1) as u32, (self.x >> 8) as u8);

        }

        if self.x_flag { 3 } else { 4 }
    }

    fn stx_absolute(&mut self, memory: &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.x_flag {
            memory.write(addr, self.x as u8);

        } else {
            memory.write(addr, self.x as u8);
            memory.write(addr + 1, (self.x >> 8) as u8);

        }

        if self.x_flag { 4 } else { 5 }

    }

    fn sty_direct_page(&mut self, memory: &mut Memory) -> u8 {
        let addr = self.dp + memory.read(self.pc) as u16;
        self.pc += 1;

        if self.x_flag {
            memory.write(addr as u32, self.y as u8);

        } else {
            memory.write(addr as u32, self.y as u8);
            memory.write((addr + 1) as u32, (self.y >> 8) as u8);
        }

        if self.x_flag { 3 } else { 4 }
    }

    fn sty_absolute(&mut self, memory: &mut Memory) -> u8{
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        let addr = (addr_high << 8) | addr_low;
        self.pc += 2;

        if self.x_flag {
            memory.write(addr, self.y as u8);

        } else {
            memory.write(addr, self.y as u8);
            memory.write(addr + 1, (self.y >> 8) as u8);
        }

        if self.x_flag { 4 } else { 5 }
    }

    // ++++ Jump Instructions ++++

    fn jmp_absolute(&mut self, memory: &mut Memory) -> u8 {
        let addr_low = memory.read(self.pc) as u32;
        let addr_high = memory.read(self.pc + 1) as u32;
        self.pc = (addr_high << 8) | addr_low;
        3
    }

    fn jmp_indirect(&mut self, memory: &mut Memory) -> u8 {
        let ptr_low = memory.read(self.pc) as u32;
        let ptr_high = memory.read(self.pc + 1) as u32;
        let ptr = (ptr_high << 8) | ptr_low;

        let addr_low = memory.read(ptr) as u32;
        let addr_high = memory.read(ptr + 1) as u32;
        self.pc = (addr_high << 8) | addr_low;
        5
    }

    // ++++ Branch Instructions ++++

    fn branch(&mut self, condition: bool, memory: &mut Memory) -> u8 {
        let offset = memory.read(self.pc) as i8;
        self.pc += 1;

        if condition {
            let old_pc = self.pc;
            self.pc = ((self.pc as i32) + (offset as i32)) as u32;

            if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
                4 // Branch crossed a page boundary

            } else {
                3 // Normal branch
            }
            
        } else {
            2   // No branch taken
        }
    }

    // ++++ Flag Operations ++++

    fn set_flag(&mut self, flag: u8) {
        self.p |= flag;
    }

    fn clear_flag(&mut self, flag: u8) {
        self.p &= !flag;
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        (self.p & flag) != 0
    }

    fn update_nz_flags_a(&mut self) {
        let value = if self.m_flag { self.a & 0xFF } else { self.a };
        self.update_nz_flags(value);

    }

    fn update_nz_flags_x(&mut self) {
        let value = if self.x_flag { self.x & 0xFF } else { self.x };
        self.update_nz_flags(value);

    }

    fn update_nz_flags_y(&mut self) {
        let value = if self.x_flag { self.y & 0xFF } else { self.y };
        self.update_nz_flags(value);

    }

    fn update_nz_flags(&mut self, value: u16) {
        self.p &= !(Self::FLAG_ZERO | Self::FLAG_NEGATIVE);

        if value == 0 {
            self.p |= Self::FLAG_ZERO;
        }

        let test_bit = if self.m_flag || self.x_flag { 0x80 } else { 0x8000 };
        if (value & test_bit) != 0 {
            self.p |= Self::FLAG_NEGATIVE;
        }
    }

    // ++++ Debugging ++++

    pub fn get_register_state(&self) -> String {
        format!(
            "A:{:04X} X:{:04X} Y:{:04X} SP:{:04X} PC:{:06X} DP:{:04X} DB:{:02X} PB:{:02X} P:{:02X} M:{} X:{} E:{}",
            self.a, self.x, self.y, self.sp, self.pc, self.dp, self.db, self.pb, self.p,
            if self.m_flag { 8 } else { 16 },
            if self.x_flag { 8 } else { 16 },
            if self.e_flag { "E" } else { "N" }
        )
    }

}