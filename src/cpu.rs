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

            Operation::StoreZero => {
                let addr = self.get_effective_address(mode, memory);

                if self.m_flag {
                    memory.write(addr, 0);

                } else {
                    memory.write(addr, 0);
                    memory.write(addr + 1, 0);
                }
            }

            Operation::Add => {
                let operand = self.read_operand(mode, memory, false);
                self.adc(operand);
            }

            Operation::Sub => {
                let operand = self.read_operand(mode, memory, false);
                self.sbc(operand);
            }

            Operation::Inc => {
                match mode {
                    AddressingMode::Implied => {
                        if self.m_flag {
                            let result = (self.a & 0xFF).wrapping_add(1) & 0xFF;
                            self.a = (self.a & 0xFF00) | result;

                        } else {
                            self.a = self.a.wrapping_add(1);
                        }
                        self.update_nz_flags_a();
                    }

                    _=> {
                        let addr = self.get_effective_address(mode, memory);
                        let is_8bit = self.m_flag;

                        if is_8bit {
                            let value = memory.read(addr).wrapping_add(1);
                            memory.write(addr, value);
                            self.update_nz_flags(value as u16);

                        } else {
                            let low = memory.read(addr) as u16;
                            let high = memory.read(addr + 1) as u16;
                            let value = ((high << 8) | low).wrapping_add(1);
                            memory.write(addr, value as u8);
                            memory.write(addr + 1, (value >> 8) as u8);
                            self.update_nz_flags(value);
                        }
                    }
                }
            }

            Operation::Dec => {
                match mode {
                    AddressingMode::Implied => {
                        if self.m_flag {
                            let result = (self.a & 0xFF).wrapping_sub(1) & 0xFF;
                            self.a = (self.a & 0xFF00) | result;

                        } else {
                            self.a = self.a.wrapping_sub(1);
                        }

                        self.update_nz_flags_a();
                    }

                    _=> {
                        let addr = self.get_effective_address(mode, memory);
                        let is_8bit = self.m_flag;

                        if is_8bit {
                            let value = memory.read(addr).wrapping_sub(1);
                            memory.write(addr, value);
                            self.update_nz_flags(value as u16);

                        } else {
                            let low = memory.read(addr) as u16;
                            let high = memory.read(addr + 1) as u16;
                            let value = ((high << 8) | low).wrapping_sub(1);
                            memory.write(addr, value as u8);
                            memory.write(addr + 1, (value >> 8) as u8);
                            self.update_nz_flags(value);
                        }
                    }
                }
            }

            Operation::And => {
                let operand = self.read_operand(mode, memory, false);
                if self.m_flag {
                    let result = (self.a & 0xFF) & operand;
                    self.a = (self.a & 0xFF00) | result;

                } else {
                    self.a &= operand;

                }

                self.update_nz_flags_a();
            }

            Operation::Or => {
                let operand = self.read_operand(mode, memory, false);
                if self.m_flag {
                    let result = (self.a & 0xFF) | operand;
                    self.a = (self.a & 0xFF00) | result;

                } else {
                    self.a |= operand;

                }

                self.update_nz_flags_a();
            }

            Operation::Xor => {
                let operand = self.read_operand(mode, memory, false);
                if self.m_flag {
                    let result = (self.a & 0xFF) ^ operand;
                    self.a = (self.a & 0xFF00) | result;

                } else {
                    self.a ^= operand;

                }

                self.update_nz_flags_a();
            }

            Operation::Compare => {
                let operand = self.read_operand(mode, memory, false);
                let acc_value = if self.m_flag { self.a & 0xFF } else { self.a };
                self.compare(acc_value, operand);
            }

            Operation::CompareX => {
                let operand = self.read_operand(mode, memory, true);
                let x_value = if self.x_flag { self.x & 0xFF } else { self.x };
                self.compare(x_value, operand);
            }

            Operation::CompareY => {
                let operand = self.read_operand(mode, memory, true);
                let y_value = if self.x_flag { self.y & 0xFF } else { self.y };
                self.compare(y_value, operand);
            }

            Operation::ShiftLeft => {
                match mode {
                    AddressingMode::Implied => {
                        if self.m_flag {
                            let value = self.a & 0xFF;
                            self.set_carry_flag((value & 0x80) != 0);
                            let result = (value << 1) & 0xFF;
                            self.a = (self.a & 0xFF00) | result;
                            self.update_nz_flags(result);

                        } else {
                            self.set_carry_flag((self.a & 0x8000) != 0);
                            self.a <<= 1;
                            self.update_nz_flags(self.a);

                        }
                    }

                    _=> {
                        let addr = self.get_effective_address(mode, memory);
                        let is_8bit = self.m_flag;

                        if is_8bit {
                            let value = memory.read(addr);
                            self.set_carry_flag((value & 0x80) != 0);
                            let result = value << 1;
                            memory.write(addr, result);
                            self.update_nz_flags(result as u16);

                        } else {
                            let low = memory.read(addr) as u16;
                            let high = memory.read(addr + 1) as u16;
                            let value = (high << 8) | low;
                            self.set_carry_flag((value & 0x8000) != 0);
                            let result = value << 1;
                            memory.write(addr, result as u8);
                            memory.write(addr + 1, (result >> 8) as u8);
                            self.update_nz_flags(result);
                        }
                    }
                }
            }

            Operation::ShiftRight => {
                match mode {
                    AddressingMode::Implied => {
                        if self.m_flag {
                            let value = self.a & 0xFF;
                            self.set_carry_flag((value & 0x01) != 0);
                            let result = value >> 1;
                            self.a = (self.a & 0xFF00) | result;
                            self.update_nz_flags(result);

                        } else {
                            self.set_carry_flag((self.a & 0x0001) != 0);
                            self.a >>= 1;
                            self.update_nz_flags(self.a);
                        }
                    }

                    _=> {
                        let addr = self.get_effective_address(mode, memory);
                        let is_8bit = self.m_flag;

                        if is_8bit {
                            let value = memory.read(addr);
                            self.set_carry_flag((value & 0x01) != 0);
                            let result = value >> 1;
                            memory.write(addr, result);
                            self.update_nz_flags(result as u16);

                        } else {
                            let low = memory.read(addr) as u16;
                            let high = memory.read(addr + 1) as u16;
                            let value = (high << 8) | low;
                            self.set_carry_flag((value & 0x0001) != 0);
                            let result = value >> 1;
                            memory.write(addr, result as u8);
                            memory.write(addr + 1, (result >> 8) as u8);
                            self.update_nz_flags(result);
                        }
                    }
                }
            }

            Operation::TransferAX => {
                if self.x_flag {
                    let value = self.a & 0xFF;
                    self.x = (self.x & 0xFF00) | value;

                } else {
                    self.x = self.a;

                }

                self.update_nz_flags_x();
            }

            Operation::TransferAY => {
                if self.x_flag {
                    let value = self.a & 0xFF;
                    self.y = (self.y & 0xFF00) | value;

                } else {
                    self.y = self.a;

                }

                self.update_nz_flags_y();
            }

            Operation::TransferXA => {
                if self.m_flag {
                    let value = self.x & 0xFF;
                    self.a = (self.a & 0xFF00) | value;

                } else {
                    self.a = self.x;

                }

                self.update_nz_flags_a();
            }

            Operation::TransferYA => {
                if self.m_flag {
                    let value = self.y & 0xFF;
                    self.a = (self.a & 0xFF00) | value;

                } else {
                    self.a = self.y;

                }

                self.update_nz_flags_a();
            }

            Operation::TransferSX => {
                if self.x_flag {
                    let value = self.sp & 0xFF;
                    self.x = (self.x & 0xFF00) | value;

                } else {
                    self.x = self.sp;

                }

                self.update_nz_flags_x();
            }

            Operation::TransferXS => {
                if self.e_flag {
                    self.sp = 0x0100 | (self.x & 0xFF);

                } else {
                    if self.x_flag{
                        self.sp = (self.sp & 0xFF00) | (self.x & 0xFF);

                    } else {
                        self.sp = self.x;

                    }

                }
            }

            Operation::TransferXY => {
                if self.x_flag {
                    let value = self.x & 0xFF;
                    self.y = (self.y & 0xFF00) | value;

                } else {

                    self.y = self.x;
                }

                self.update_nz_flags_y();

            }

            Operation::TransferYX => {
                if self.x_flag {
                    let value = self.y & 0xFF;
                    self.x = (self.x & 0xFF00) | value;

                } else {

                    self.x = self.y;
                }

                self.update_nz_flags_x();
            }

            Operation::TransferSC => {
                if self.m_flag {
                    let value = self.sp & 0xFF;
                    self.a = (self.sp & 0xFF00) | value;

                } else {
                    self.a = self.sp;
                }

                self.update_nz_flags_a();
            }

            Operation::TransferCS => {
                if self.e_flag {
                    self.sp = 0x0100 | (self.sp & 0xFF);

                } else {
                    if self.m_flag {
                        self.sp = (self.sp & 0xFF00) | (self.a & 0xFF);

                    } else {
                        self.sp = self.a;
                    }
                }
            }

            Operation::PushA => {
                if self.m_flag {
                    self.push_byte(memory, (self.a & 0xFF) as u8);

                } else {
                    self.push_byte(memory, (self.a >> 8) as u8);
                    self.push_byte(memory, (self.a & 0xFF) as u8);
                }
            }

            Operation::PullA => {
                if self.m_flag {
                    let value = self.pull_byte(memory) as u16;
                    self.a = (self.a & 0xFF00) | value;

                } else {
                    let low = self.pull_byte(memory) as u16;
                    let high = self.pull_byte(memory) as u16;
                    self.a = (high << 8) | low;
                }

                self.update_nz_flags_a();
            }

            Operation::PushP => {
                self.push_byte(memory, self.p);
            }

            Operation::PullP => {
                self.p = self.pull_byte(memory);
                self.update_mode_flags();
            }

            Operation::PushX => {
                if self.x_flag {
                    self.push_byte(memory, (self.x & 0xFF) as u8);

                } else {
                    self.push_byte(memory, (self.x >> 8) as u8);
                    self.push_byte(memory, (self.x & 0xFF) as u8);
                }
            }

            Operation::PullX => {
                if self.x_flag {
                    let value = self.pull_byte(memory) as u16;
                    self.x = (self.x & 0xFF00) | value;

                } else {
                    let low = self.pull_byte(memory) as u16;
                    let high = self.pull_byte(memory) as u16;
                    self.x = (high << 8) | low;
                }

                self.update_nz_flags_x();
            }

            Operation::PushY => {
                if self.x_flag {
                    self.push_byte(memory, (self.y & 0xFF) as u8);

                } else {
                    self.push_byte(memory, (self.y >> 8) as u8);
                    self.push_byte(memory, (self.y & 0xFF) as u8);
                }
            }

            Operation::PullY => {
                if self.x_flag {
                    let value = self.pull_byte(memory) as u16;
                    self.y = (self.y & 0xFF00) | value;

                } else {
                    let low = self.pull_byte(memory) as u16;
                    let high = self.pull_byte(memory) as u16;
                    self.y = (high << 8) | low;
                }

                self.update_nz_flags_y();
            }

            Operation::JumpSubroutine => {
                let target = self.read_address(mode, memory);

                let return_addr = self.pc -1;
                self.push_byte(memory, (return_addr >> 8) as u8);
                self.push_byte(memory, return_addr as u8);

                self.pc = target;
            }

            Operation::ReturnFromSubroutine => {
                let low = self.pull_byte(memory) as u32;
                let high = self.pull_byte(memory) as u32;
                self.pc = ((high << 8) | low) + 1;
            }

            Operation::ReturnFromInterrupt => {
                self.p = self.pull_byte(memory);
                let low = self.pull_byte(memory) as u32;
                let high = self.pull_byte(memory) as u32;
                self.pc = (high << 8) | low;

                self.update_mode_flags();
            }

            Operation::SoftwareInterrupt => {
                self.pc += 1;

                self.push_byte(memory, (self.pc >> 8) as u8);
                self.push_byte(memory, self.pc as u8);
                self.push_byte(memory, self.p | 0x10);

                self.p |= Self::FLAG_IRQ;

                let brk_low = memory.read(0x00FFFE) as u32;
                let brk_high = memory.read(0x00FFFF) as u32;
                self.pc = (brk_high << 8) | brk_low;

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

            AddressingMode:: DirectPageIndexedX => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let addr = (self.dp + base + (self.x & 0xFF)) & 0xFFFF;

                if is_8bit {
                    memory.read(addr as u32) as u16
                } else {
                    let low = memory.read(addr as u32) as u16;
                    let high = memory.read((addr + 1) as u32) as u16;
                    (high << 8) | low
                }
            }

            AddressingMode::DirectPageIndexedY => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let addr = (self.dp + base + (self.y & 0xFF)) & 0xFFFF;

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

            AddressingMode::AbsoluteIndexedX => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                let addr = base + (self.x & 0xFFFF) as u32;

                if is_8bit {
                    memory.read(addr) as u16
                } else {
                    let low = memory.read(addr) as u16;
                    let high = memory.read(addr + 1) as u16;
                    (high << 8) | low
                }
            }

            AddressingMode::AbsoluteIndexedY => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                let addr = base + (self.y & 0xFFFF) as u32;

                if is_8bit {
                    memory.read(addr) as u16
                } else {
                    let low = memory.read(addr) as u16;
                    let high = memory.read(addr + 1) as u16;
                    (high << 8) | low
                }
            }

            AddressingMode::IndirectIndexed => {
                let dp_addr = (self.dp + memory.read(self.pc) as u16) & 0xFFFF;
                self.pc += 1;

                let prt_low = memory.read(dp_addr as u32) as u32;
                let ptr_high = memory.read(((dp_addr + 1) & 0xFFFF) as u32) as u32;
                let base_addr = (ptr_high << 8) | prt_low;
                let addr = base_addr + (self.y & 0xFFFF) as u32;

                if is_8bit{
                    memory.read(addr) as u16
                } else {
                    let low = memory.read(addr) as u16;
                    let high = memory.read(addr + 1) as u16;
                    (high << 8) | low
                }

            }

            AddressingMode::IndexedIndirect => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let dp_addr = (self.dp + base + (self.x & 0xFF)) & 0xFFFF;

                let ptr_low = memory.read(dp_addr as u32) as u32;
                let ptr_high = memory.read(((dp_addr + 1) & 0xFFFF) as u32) as u32;
                let addr = (ptr_high << 8) | ptr_low;

                if is_8bit{
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

            AddressingMode::DirectPageIndexedX => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let addr = (self.dp + base + (self.x & 0xFF)) & 0xFFFF;

                memory.write(addr as u32, value as u8);
                if !is_8bit {
                    memory.write((addr + 1) as u32, (value >> 8) as u8);
                }
            }

            AddressingMode::DirectPageIndexedY => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let addr = (self.dp + base + (self.y & 0xFF)) & 0xFFFF;

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

            AddressingMode::AbsoluteIndexedX => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                let addr = base + (self.x & 0xFFFF) as u32;

                memory.write(addr, value as u8);
                if !is_8bit {
                    memory.write(addr + 1, (value >> 8) as u8);
                }
            }

            AddressingMode::AbsoluteIndexedY => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                let addr = base + (self.y & 0xFFFF) as u32;

                memory.write(addr, value as u8);
                if !is_8bit {
                    memory.write(addr + 1, (value >> 8) as u8);
                }
            }

            AddressingMode::IndirectIndexed => {
                let dp_addr = (self.dp + memory.read(self.pc) as u16) & 0xFFFF;
                self.pc += 1;

                let ptr_low = memory.read(dp_addr as u32) as u32;
                let ptr_high = memory.read(((dp_addr + 1) & 0xFFFF) as u32) as u32;
                let base_addr = (ptr_high << 8) | ptr_low;
                let addr = base_addr + (self.y & 0xFFFF) as u32;

                memory.write(addr, value as u8);
                if !is_8bit {
                    memory.write(addr + 1, (value >> 8) as u8);
                }
            }

            AddressingMode::IndexedIndirect => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                let dp_addr = (self.dp + base + (self.x & 0xFF)) & 0xFFFF;

                let ptr_low = memory.read(dp_addr as u32) as u32;
                let ptr_high = memory.read(((dp_addr + 1) & 0xFFFF) as u32) as u32;
                let addr = (ptr_high << 8) | ptr_low;

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

    fn adc(&mut self, operand: u16) {
        let acc_value = if self.m_flag { self.a & 0xFF } else { self.a };
        let carry = if self.get_flag(Self::FLAG_CARRY) { 1 } else { 0 };

        if self.m_flag{
            let result = acc_value + operand + carry;

            self.set_carry_flag(result > 0xFF);
            self.set_overflow_flag_add(acc_value as u8, operand as u8, result as u8);

            self.a = (self.a & 0xFF00) | (result & 0xFF);

        } else {
            let result = (acc_value as u32) + (operand as u32) + (carry as u32);

            self.set_carry_flag(result > 0xFFFF);
            self.set_overflow_flag_add16(acc_value, operand, result as u16);

            self.a = result as u16;
        }

        self.update_nz_flags_a();
    }

    fn sbc(&mut self, operand: u16) {
        let acc_value = if self.m_flag {self.a & 0xFF } else { self.a };
        let carry = if self.get_flag(Self::FLAG_CARRY) { 0 } else { 1 };

        if self.m_flag {
            let result = acc_value as i16 - operand as i16 - carry;

            self.set_carry_flag(result >= 0);
            self.set_overflow_flag_sub(acc_value as u8, operand as u8, result as u8);

            self.a = (self.a & 0xFF00) | ((result as u16) & 0xFF);

        } else {
            let result = (acc_value as i32) - (operand as i32) - (carry as i32);

            self.set_carry_flag(result >= 0);
            self.set_overflow_flag_sub16(acc_value, operand, result as u16);

            self.a = result as u16;
        }

        self.update_nz_flags_a();
    }

    fn compare(&mut self, register_value: u16, operand: u16) {
        let result = register_value as i16 - operand as i16;

        self.set_carry_flag(register_value >= operand);

        if register_value == operand {
            self.p |= Self::FLAG_ZERO;
        } else {
            self.p &= !Self::FLAG_ZERO;
        }

        let test_bit = if self.m_flag || self.x_flag { 0x80 } else { 0x8000 };

        if ((result as u16) & test_bit) != 0 {
            self.p |= Self::FLAG_NEGATIVE;
        } else {
            self.p &= !Self::FLAG_NEGATIVE;
        }
    }

    fn get_effective_address(&mut self, mode: AddressingMode, memory: &mut Memory) -> u32 {
        match mode {
            AddressingMode::DirectPage => {
                let addr = self.dp + memory.read(self.pc) as u16;
                self.pc += 1;
                addr as u32
            }

            AddressingMode::DirectPageIndexedX => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                ((self.dp + base + (self.x & 0xFF)) & 0xFFFF) as u32
            }

            AddressingMode::DirectPageIndexedY => {
                let base = memory.read(self.pc) as u16;
                self.pc += 1;
                ((self.dp + base + (self.y & 0xFF)) & 0xFFFF) as u32
            }

            AddressingMode::Absolute => {
                self.read_address(mode, memory)
            }

            AddressingMode::AbsoluteIndexedX => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                base + (self.x & 0xFFFF) as u32
            }

            AddressingMode::AbsoluteIndexedY => {
                let base = self.read_address(AddressingMode::Absolute, memory);
                base + (self.y & 0xFFFF) as u32
            }

            _ => {
                println!("Unsupported addressing mode for effective address: {:?}", mode);
                0
            }
        }
    }

    fn push_byte(&mut self, memory: &mut Memory, value: u8) {
        memory.write(self.sp as u32, value);

        if self.e_flag {
            if (self.sp & 0xFF) == 0x00 {
                self.sp = 0x01FF;

            } else {
                self.sp -= 1;


            }

        } else {
            self.sp = self.sp.wrapping_sub(1);
        }
    }

    fn pull_byte(&mut self, memory: &mut Memory) -> u8 {
        if self.e_flag {
            if (self.sp & 0xFF) == 0xFF {
                self.sp = 0x0100;

            } else {
                self.sp += 1;
            }

        } else {
            self.sp = self.sp.wrapping_add(1);
        }

        memory.read(self.sp as u32)
    }
    // ++++ Flag Operations ++++

    fn set_flag(&mut self, flag: u8) {
        self.p |= flag;
    }

    fn clear_flag(&mut self, flag: u8) {
        self.p &= !flag;
    }

    fn set_carry_flag(&mut self, set: bool) {
        if set {
            self.p |= Self::FLAG_CARRY;
        } else {
            self.p &= !Self::FLAG_CARRY;
        }
    }

    fn set_overflow_flag_add(&mut self, a: u8, b: u8, result: u8) {
        let overflow = ((a ^result) & (b ^ result) & 0x80) != 0;

        if overflow {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }
    }

    fn set_overflow_flag_add16(&mut self, a: u16, b: u16, result: u16) {
        let overflow = ((a ^result) & (b ^ result) & 0x8000) != 0;

        if overflow {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }
    }

    fn set_overflow_flag_sub(&mut self, a: u8, b: u8, result: u8) {
        let overflow = ((a ^ b) & (a ^ result) & 0x80) != 0;

        if overflow {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }
    }

    fn set_overflow_flag_sub16(&mut self, a: u16, b: u16, result: u16) {
        let overflow = ((a ^ b) & (a ^ result) & 0x8000) != 0;

        if overflow {
            self.p |= Self::FLAG_OVERFLOW;
        } else {
            self.p &= !Self::FLAG_OVERFLOW;
        }
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

    fn update_mode_flags(&mut self) {
        if !self.e_flag {
            self.m_flag = (self.p & 0x20) != 0;
            self.x_flag = (self.p & 0x10) != 0;
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
