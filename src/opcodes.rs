use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]

pub enum Operation {
    LoadA, LoadX, LoadY,

    StoreA, StoreX, StoreY,

    Add, Sub, Inc, Dec,

    And, Or, Xor,

    SetFlag(u8), ClearFlag(u8),

    Jump, JumpIndirect,

    Branch { flag: u8, condition: bool },

    Nop,
}

#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    Implied,
    Immediate,
    DirectPage,
    Absolute,
    Indirect,
}

#[derive(Clone, Copy, Debug)]
pub struct OpcodeInfo {
    pub operation: Operation,
    pub mode: AddressingMode,
    pub cycles: u8,
}

use Operation::*;
use AddressingMode::*;

pub const FLAG_CARRY: u8 = 0x01;
pub const FLAG_ZERO: u8 = 0x02;
pub const FLAG_IRQ: u8 = 0x04;
pub const FLAG_DECIMAL: u8 = 0x08;
pub const FLAG_OVERFLOW: u8 = 0x40;
pub const FLAG_NEGATIVE: u8 = 0x80;

pub fn create_opcode_table() -> HashMap<u8, OpcodeInfo> {
    let mut table = HashMap::new();

    //Flags
    table.insert(0x18, OpcodeInfo { operation: ClearFlag(FLAG_CARRY), mode: Implied, cycles: 2 });
    table.insert(0x38, OpcodeInfo { operation: SetFlag(FLAG_CARRY), mode: Implied, cycles: 2 });
    table.insert(0x58, OpcodeInfo { operation: ClearFlag(FLAG_IRQ), mode: Implied, cycles: 2 });
    table.insert(0x78, OpcodeInfo { operation: SetFlag(FLAG_IRQ), mode: Implied, cycles: 2 });
    table.insert(0xB8, OpcodeInfo { operation: ClearFlag(FLAG_OVERFLOW), mode: Implied, cycles: 2 });
    table.insert(0xD8, OpcodeInfo { operation: ClearFlag(FLAG_DECIMAL), mode: Implied, cycles: 2 });
    table.insert(0xF8, OpcodeInfo { operation: SetFlag(FLAG_DECIMAL), mode: Implied, cycles: 2 });

    //Load
    table.insert(0xA9, OpcodeInfo { operation: LoadA, mode: Immediate, cycles: 2 });
    table.insert(0xA5, OpcodeInfo { operation: LoadA, mode: DirectPage, cycles: 3 });
    table.insert(0xAD, OpcodeInfo { operation: LoadA, mode: Absolute, cycles: 4 });
    table.insert(0xA2, OpcodeInfo { operation: LoadX, mode: Immediate, cycles: 2 });
    table.insert(0xA6, OpcodeInfo { operation: LoadX, mode: DirectPage, cycles: 3 });
    table.insert(0xAE, OpcodeInfo { operation: LoadX, mode: Absolute, cycles: 4 });
    table.insert(0xA0, OpcodeInfo { operation: LoadY, mode: Immediate, cycles: 2 });
    table.insert(0xA4, OpcodeInfo { operation: LoadY, mode: DirectPage, cycles: 3 });
    table.insert(0xAC, OpcodeInfo { operation: LoadY, mode: Absolute, cycles: 4 });

    //Store
    table.insert(0x85, OpcodeInfo { operation: StoreA, mode: DirectPage, cycles: 3 });
    table.insert(0x8D, OpcodeInfo { operation: StoreA, mode: Absolute, cycles: 4 });
    table.insert(0x86, OpcodeInfo { operation: StoreX, mode: DirectPage, cycles: 3 });
    table.insert(0x8E, OpcodeInfo { operation: StoreX, mode: Absolute, cycles: 4 });
    table.insert(0x84, OpcodeInfo { operation: StoreY, mode: DirectPage, cycles: 3 });
    table.insert(0x8C, OpcodeInfo { operation: StoreY, mode: Absolute, cycles: 4 });

    //Jumps
    table.insert(0x4C, OpcodeInfo { operation: Jump, mode: Absolute, cycles: 3 });
    table.insert(0x6C, OpcodeInfo { operation: JumpIndirect, mode: Indirect, cycles: 5 });

    //Branches
    table.insert(0x10, OpcodeInfo { operation: Branch { flag: FLAG_NEGATIVE, condition: false }, mode: Implied, cycles: 2 });
    table.insert(0x30, OpcodeInfo { operation: Branch { flag: FLAG_NEGATIVE, condition: true }, mode: Implied, cycles: 2 });
    table.insert(0x50, OpcodeInfo { operation: Branch { flag: FLAG_OVERFLOW, condition: false }, mode: Implied, cycles: 2 });
    table.insert(0x70, OpcodeInfo { operation: Branch { flag: FLAG_OVERFLOW, condition: true }, mode: Implied, cycles: 2 });
    table.insert(0x90, OpcodeInfo { operation: Branch { flag: FLAG_CARRY, condition: false }, mode: Implied, cycles: 2 });
    table.insert(0xB0, OpcodeInfo { operation: Branch { flag: FLAG_CARRY, condition: true }, mode: Implied, cycles: 2 });
    table.insert(0xD0, OpcodeInfo { operation: Branch { flag: FLAG_ZERO, condition: false }, mode: Implied, cycles: 2 });
    table.insert(0xF0, OpcodeInfo { operation: Branch { flag: FLAG_ZERO, condition: true }, mode: Implied, cycles: 2 });

    //Placeholder
    table.insert(0xEA, OpcodeInfo { operation: Nop, mode: Implied, cycles: 2 });

    table
}

use std::sync::OnceLock;

static OPCODE_MAP: OnceLock<HashMap<u8, OpcodeInfo>> = OnceLock::new();

pub fn get_opcode_info(opcode: u8) -> Option<&'static OpcodeInfo> {
    let map = OPCODE_MAP.get_or_init(|| create_opcode_table());
    map.get(&opcode)
}