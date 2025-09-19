use crate::memory::Memory;
use crate::opcodes::{get_opcode_info, Operation, AddressingMode, FLAG_CARRY, FLAG_ZERO, FLAG_IRQ, FLAG_DECIMAL, FLAG_OVERFLOW, FLAG_NEGATIVE};

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
    pub const FLAG_CARRY: u8 = FLAG_CARRY;
    pub const FLAG_ZERO: u8 = FLAG_ZERO;
    pub const FLAG_IRQ: u8 = FLAG_IRQ;
    pub const FLAG_DECIMAL: u8 = FLAG_DECIMAL;
    pub const FLAG_OVERFLOW: u8 = FLAG_OVERFLOW;
    pub const FLAG_NEGATIVE: u8 = FLAG_NEGATIVE;

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
        match get_opcode_info(opcode){
            Some(info) => {
                self.execute_operation(info.operation, info.mode, memory);
                self.adjust_cycles(info.cycles, info.mode)
            }

            None => {
                println!("Unknown opcode: {:02X} at PC: {:06X}", opcode, self.pc - 1);
                2
            }
        }
    }

    fn execute_operation(&mut self, op: Operation, mode: AddressingMode, memory: &mut Memory) {
        match op {
            Operation::LoadA => {
                let value = self.read_operand(mode, memory, false);
                self.a = if self.m_flag { (self.a & 0xFF00) | value} else { value };
                self.update_nz_flags_a();
            }

            Operation::LoadX => {
                let value = self.read_operand(mode, memory, false);
                self.x = if self.x_flag { (self.x & 0xFF00) | value} else { value };
                self.update_nz_flags_x();
            }

            Operation::LoadY => {
                let value = self.read_operand(mode, memory, false);
                self.y = if self.x_flag { (self.y & 0xFF00) | value} else { value };
                self.update_nz_flags_y();
            }

            Operation::StoreA => {
                let value = if self.m_flag { self.a & 0xFF } else { self.a };
                self.write_operand(mode, memory, value, false);
            }

            Operation::StoreX => {
                let value = if self.x_flag { self.x & 0xFF } else { self.x };
                self.write_operand(mode, memory, value, true);
            }

            Operation::StoreY => {
                let value = if self.x_flag { self.y & 0xFF } else { self.y };
                self.write_operand(mode, memory, value, true);
            }

            Operation::SetFlag(flag) => self.set_flag(flag),
            Operation::ClearFlag(flag) => self.clear_flag(flag),

            Operation::Jump => {
                let addr = self.read_address(mode, memory);
                self.pc = addr;
            }

            Operation::JumpIndirect => {
                let ptr = self.read_address(AddressingMode::Absolute, memory);
                let addr_low = memory.read(ptr) as u32;
                let addr_high = memory.read(ptr + 1) as u32;
                self.pc = (addr_high << 8) | addr_low;
            }

            Operation::Branch { flag, condition} => {
                let flag_set = self.get_flag(flag);
                let should_branch = flag_set == condition;

                let offset = memory.read(self.pc) as i8;
                self.pc += 1;

                if should_branch {
                    self.pc = ((self.pc as i32) + (offset as i32)) as u32;
                }
            }

            Operation::Nop => { /* Do nothing */}

            _ => {
                println!("Unimplemented operation: {:?} in mode: {:?}", op, mode);
            }
        }
    }

    fn read_operand(&mut self, mode: AddressingMode, memory: &mut Memory, use_x_flag: bool) -> u16 {
        let is_8bit = if use_x_flag { self.x_flag } else { self.m_flag };

        match mode {
            AddressingMode::Immediate => {
                if is_8bit {
                    let value = memory.read(self.pc) as u16;
                    self.pc += 1;
                    value
                } else {
                    let low = memory.read(self.pc) as u16;
                    let high = memory.read(self.pc + 1) as u16;
                    self.pc += 2;
                    (high << 8) | low
                }
            }

            AddressingMode::DirectPage => {
                let addr = self.dp + memory.read(self.pc) as u16;
                self.pc += 1;

                if is_8bit {
                    memory.read(addr as u32) as u16
                } else {
                    let low = memory.read(addr as u32) as u16;
                    let high = memory.read((addr + 1) as u32) as u16;
                    (high << 8) | low
                }
            }

            AddressingMode::Absolute => {
                let addr = self.read_address(mode, memory);

                if is_8bit {
                    memory.read(addr) as u16
                } else {
                    let low = memory.read(addr) as u16;
                    let high = memory.read(addr + 1) as u16;
                    (high << 8) | low
                }
            }

            _ => {
                println!("Unsupported addressing mode for read_operand: {:?}", mode);
                0
            }
        }
    }

    fn write_operand(&mut self, mode: AddressingMode, memory: &mut Memory, value: u16, use_x_flag: bool) {
        let is_8bit = if use_x_flag { self.x_flag } else { self.m_flag };

        match mode {
            AddressingMode::DirectPage => {
                let addr = self.dp + memory.read(self.pc) as u16;
                self.pc += 1;

                memory.write(addr as u32, value as u8);

                if !is_8bit {
                    memory.write((addr + 1) as u32, (value >> 8) as u8);
                }
            }

            AddressingMode::Absolute => {
                let addr = self.read_address(mode, memory);

                memory.write(addr, value as u8);
                if !is_8bit {
                    memory.write(addr + 1, (value >> 8) as u8);
                }
            }

            _ => {
                println!("Unsupported addressing mode for write_operand: {:?}", mode);
            }
        }
    }

    fn read_address(&mut self, mode: AddressingMode, memory: &mut Memory) -> u32 {
        match mode {
            AddressingMode::Absolute => {
                let addr_low = memory.read(self.pc) as u32;
                let addr_high = memory.read(self.pc + 1) as u32;
                self.pc += 2;
                (addr_high << 8) | addr_low
                
            }

            _ => {
                println!("Unsupported addressing mode for read_address: {:?}", mode);
                0
            }
        }
    }

    fn adjust_cycles(&self, base_cycles: u8, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode:: Immediate => {
                if !self.m_flag || !self.x_flag {
                    base_cycles + 1
                } else {
                    base_cycles
                }
            }

            _ => base_cycles
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
