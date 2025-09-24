use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]

pub enum Operation {
    LoadA, LoadX, LoadY,

    StoreA, StoreX, StoreY, StoreZero,

    Add, Sub, Inc, Dec,

    And, Or, Xor,

    Xce, Rep, Sep, Tcd,

    DecX, Rtl,

    Compare, CompareX, CompareY,

    ShiftLeft, ShiftRight,

    TransferAX, TransferAY, TransferXA, TransferXY, TransferYA, TransferYX, TransferSX, TransferXS,
    TransferSC, TransferCS,

    PushA, PullA, PushP, PullP, PushX, PullX, PushY, PullY,

    JumpSubroutine, ReturnFromSubroutine, ReturnFromInterrupt, SoftwareInterrupt,

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
    DirectPageIndexedX,
    DirectPageIndexedY,
    Absolute,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    AbsoluteLong,
    AbsoluteLongIndexedX,
    Indirect,
    IndirectIndexed,
    IndexedIndirect,
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

    //Transfers
    table.insert(0xAA, OpcodeInfo { operation: TransferAX, mode: Implied, cycles: 2 });
    table.insert(0xA8, OpcodeInfo { operation: TransferAY, mode: Implied, cycles: 2 });
    table.insert(0x8A, OpcodeInfo { operation: TransferXA, mode: Implied, cycles: 2 });
    table.insert(0x98, OpcodeInfo { operation: TransferYA, mode: Implied, cycles: 2 });
    table.insert(0x9B, OpcodeInfo { operation: TransferXY, mode: Implied, cycles: 2 });
    table.insert(0xBB, OpcodeInfo { operation: TransferYX, mode: Implied, cycles: 2 });
    table.insert(0xBA, OpcodeInfo { operation: TransferSX, mode: Implied, cycles: 2 });
    table.insert(0x9A, OpcodeInfo { operation: TransferXS, mode: Implied, cycles: 2 });
    table.insert(0x3B, OpcodeInfo { operation: TransferSC, mode: Implied, cycles: 2 });
    table.insert(0x1B, OpcodeInfo { operation: TransferCS, mode: Implied, cycles: 2 });

    //Load
    table.insert(0xA9, OpcodeInfo { operation: LoadA, mode: Immediate, cycles: 2 });
    table.insert(0xA5, OpcodeInfo { operation: LoadA, mode: DirectPage, cycles: 3 });
    table.insert(0xB5, OpcodeInfo { operation: LoadA, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0xAD, OpcodeInfo { operation: LoadA, mode: Absolute, cycles: 4 });
    table.insert(0xBD, OpcodeInfo { operation: LoadA, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0xB9, OpcodeInfo { operation: LoadA, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0xB1, OpcodeInfo { operation: LoadA, mode: IndirectIndexed, cycles: 5 });
    table.insert(0xA1, OpcodeInfo { operation: LoadA, mode: IndexedIndirect, cycles: 6 });
    table.insert(0xA2, OpcodeInfo { operation: LoadX, mode: Immediate, cycles: 2 });
    table.insert(0xA6, OpcodeInfo { operation: LoadX, mode: DirectPage, cycles: 3 });
    table.insert(0xB6, OpcodeInfo { operation: LoadX, mode: DirectPageIndexedY, cycles: 4 });
    table.insert(0xAE, OpcodeInfo { operation: LoadX, mode: Absolute, cycles: 4 });
    table.insert(0xBE, OpcodeInfo { operation: LoadX, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0xA0, OpcodeInfo { operation: LoadY, mode: Immediate, cycles: 2 });
    table.insert(0xA4, OpcodeInfo { operation: LoadY, mode: DirectPage, cycles: 3 });
    table.insert(0xB4, OpcodeInfo { operation: LoadY, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0xAC, OpcodeInfo { operation: LoadY, mode: Absolute, cycles: 4 });
    table.insert(0xBC, OpcodeInfo { operation: LoadY, mode: AbsoluteIndexedX, cycles: 4 });

    //Store
    table.insert(0x85, OpcodeInfo { operation: StoreA, mode: DirectPage, cycles: 3 });
    table.insert(0x95, OpcodeInfo { operation: StoreA, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x8D, OpcodeInfo { operation: StoreA, mode: Absolute, cycles: 4 });
    table.insert(0x8F, OpcodeInfo { operation: StoreA, mode: AbsoluteLong, cycles: 5 });
    table.insert(0x9D, OpcodeInfo { operation: StoreA, mode: AbsoluteIndexedX, cycles: 5 });
    table.insert(0x9F, OpcodeInfo { operation: StoreA, mode: AbsoluteLongIndexedX, cycles: 5 });
    table.insert(0x99, OpcodeInfo { operation: StoreA, mode: AbsoluteIndexedY, cycles: 5 });
    table.insert(0x91, OpcodeInfo { operation: StoreA, mode: IndirectIndexed, cycles: 6 });
    table.insert(0x81, OpcodeInfo { operation: StoreA, mode: IndexedIndirect, cycles: 6 });
    table.insert(0x86, OpcodeInfo { operation: StoreX, mode: DirectPage, cycles: 3 });
    table.insert(0x96, OpcodeInfo { operation: StoreX, mode: DirectPageIndexedY, cycles: 4 });
    table.insert(0x8E, OpcodeInfo { operation: StoreX, mode: Absolute, cycles: 4 });
    table.insert(0x84, OpcodeInfo { operation: StoreY, mode: DirectPage, cycles: 3 });
    table.insert(0x94, OpcodeInfo { operation: StoreY, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x8C, OpcodeInfo { operation: StoreY, mode: Absolute, cycles: 4 });
    table.insert(0x64, OpcodeInfo { operation: StoreZero, mode: DirectPage, cycles: 3 });
    table.insert(0x74, OpcodeInfo { operation: StoreZero, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x9C, OpcodeInfo { operation: StoreZero, mode: Absolute, cycles: 4 });
    table.insert(0x9E, OpcodeInfo { operation: StoreZero, mode: AbsoluteIndexedX, cycles: 5 });

    table.insert(0xFB, OpcodeInfo { operation: Xce, mode: Implied, cycles: 2 });
    table.insert(0xC2, OpcodeInfo { operation: Rep, mode: Immediate, cycles: 3 });
    table.insert(0xE2, OpcodeInfo { operation: Sep, mode: Immediate, cycles: 3 });
    table.insert(0x5B, OpcodeInfo { operation: Tcd, mode: Implied, cycles: 2 });

    //Arithmetic
    table.insert(0x69, OpcodeInfo { operation: Add, mode: Immediate, cycles: 2 });
    table.insert(0x65, OpcodeInfo { operation: Add, mode: DirectPage, cycles: 3 });
    table.insert(0x75, OpcodeInfo { operation: Add, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x6D, OpcodeInfo { operation: Add, mode: Absolute, cycles: 4 });
    table.insert(0x7D, OpcodeInfo { operation: Add, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0x79, OpcodeInfo { operation: Add, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0x71, OpcodeInfo { operation: Add, mode: IndirectIndexed, cycles: 5 });
    table.insert(0x61, OpcodeInfo { operation: Add, mode: IndexedIndirect, cycles: 6 });

    table.insert(0xE9, OpcodeInfo { operation: Sub, mode: Immediate, cycles: 2 });
    table.insert(0xE5, OpcodeInfo { operation: Sub, mode: DirectPage, cycles: 3 });
    table.insert(0xF5, OpcodeInfo { operation: Sub, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0xED, OpcodeInfo { operation: Sub, mode: Absolute, cycles: 4 });
    table.insert(0xFD, OpcodeInfo { operation: Sub, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0xF9, OpcodeInfo { operation: Sub, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0xF1, OpcodeInfo { operation: Sub, mode: IndirectIndexed, cycles: 5 });
    table.insert(0xE1, OpcodeInfo { operation: Sub, mode: IndexedIndirect, cycles: 6 });

    table.insert(0x1A, OpcodeInfo { operation: Inc, mode: Implied, cycles: 2 });
    table.insert(0xE6, OpcodeInfo { operation: Inc, mode: DirectPage, cycles: 5 });
    table.insert(0xEE, OpcodeInfo { operation: Inc, mode: Absolute, cycles: 6 });

    table.insert(0x3A, OpcodeInfo { operation: Dec, mode: Implied, cycles: 2 });
    table.insert(0xC6, OpcodeInfo { operation: Dec, mode: DirectPage, cycles: 5 });
    table.insert(0xCE, OpcodeInfo { operation: Dec, mode: Absolute, cycles: 6 });

    table.insert(0x29, OpcodeInfo { operation: And, mode: Immediate, cycles: 2 });
    table.insert(0x25, OpcodeInfo { operation: And, mode: DirectPage, cycles: 3 });
    table.insert(0x35, OpcodeInfo { operation: And, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x2D, OpcodeInfo { operation: And, mode: Absolute, cycles: 4 });
    table.insert(0x3D, OpcodeInfo { operation: And, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0x39, OpcodeInfo { operation: And, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0x31, OpcodeInfo { operation: And, mode: IndirectIndexed, cycles: 5 });
    table.insert(0x21, OpcodeInfo { operation: And, mode: IndexedIndirect, cycles: 6 });

    table.insert(0x09, OpcodeInfo { operation: Or, mode: Immediate, cycles: 2 });
    table.insert(0x05, OpcodeInfo { operation: Or, mode: DirectPage, cycles: 3 });
    table.insert(0x15, OpcodeInfo { operation: Or, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x0D, OpcodeInfo { operation: Or, mode: Absolute, cycles: 4 });
    table.insert(0x1D, OpcodeInfo { operation: Or, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0x19, OpcodeInfo { operation: Or, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0x11, OpcodeInfo { operation: Or, mode: IndirectIndexed, cycles: 5 });
    table.insert(0x01, OpcodeInfo { operation: Or, mode: IndexedIndirect, cycles: 6 });

    table.insert(0x49, OpcodeInfo { operation: Xor, mode: Immediate, cycles: 2 });
    table.insert(0x45, OpcodeInfo { operation: Xor, mode: DirectPage, cycles: 3 });
    table.insert(0x55, OpcodeInfo { operation: Xor, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0x4D, OpcodeInfo { operation: Xor, mode: Absolute, cycles: 4 });
    table.insert(0x5D, OpcodeInfo { operation: Xor, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0x59, OpcodeInfo { operation: Xor, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0x51, OpcodeInfo { operation: Xor, mode: IndirectIndexed, cycles: 5 });
    table.insert(0x41, OpcodeInfo { operation: Xor, mode: IndexedIndirect, cycles: 6 });

    table.insert(0xC9, OpcodeInfo { operation: Compare, mode: Immediate, cycles: 2 });
    table.insert(0xC5, OpcodeInfo { operation: Compare, mode: DirectPage, cycles: 3 });
    table.insert(0xD5, OpcodeInfo { operation: Compare, mode: DirectPageIndexedX, cycles: 4 });
    table.insert(0xCD, OpcodeInfo { operation: Compare, mode: Absolute, cycles: 4 });
    table.insert(0xDD, OpcodeInfo { operation: Compare, mode: AbsoluteIndexedX, cycles: 4 });
    table.insert(0xD9, OpcodeInfo { operation: Compare, mode: AbsoluteIndexedY, cycles: 4 });
    table.insert(0xD1, OpcodeInfo { operation: Compare, mode: IndirectIndexed, cycles: 5 });
    table.insert(0xC1, OpcodeInfo { operation: Compare, mode: IndexedIndirect, cycles: 6 });

    table.insert(0xE0, OpcodeInfo {operation: CompareX, mode: Immediate, cycles: 2});
    table.insert(0xE4, OpcodeInfo {operation: CompareX, mode: DirectPage, cycles: 3});
    table.insert(0xEC, OpcodeInfo {operation: CompareX, mode: Absolute, cycles: 4});

    table.insert(0xC0, OpcodeInfo {operation: CompareY, mode: Immediate, cycles: 2});
    table.insert(0xC4, OpcodeInfo {operation: CompareY, mode: DirectPage, cycles: 3});
    table.insert(0xCC, OpcodeInfo {operation: CompareY, mode: Absolute, cycles: 4});

    table.insert(0xCA, OpcodeInfo { operation: DecX, mode: Implied, cycles: 2 });
    table.insert(0x6B, OpcodeInfo { operation: Rtl, mode: Implied, cycles: 6 });

    //Stacks
    table.insert(0x48, OpcodeInfo { operation: PushA, mode: Implied, cycles: 3 });
    table.insert(0x68, OpcodeInfo { operation: PullA, mode: Implied, cycles: 4 });
    table.insert(0x08, OpcodeInfo { operation: PushP, mode: Implied, cycles: 3 });
    table.insert(0x28, OpcodeInfo { operation: PullP, mode: Implied, cycles: 4 });
    table.insert(0xDA, OpcodeInfo { operation: PushX, mode: Implied, cycles: 3 });
    table.insert(0xFA, OpcodeInfo { operation: PullX, mode: Implied, cycles: 4 });
    table.insert(0x5A, OpcodeInfo { operation: PushY, mode: Implied, cycles: 3 });
    table.insert(0x7A, OpcodeInfo { operation: PullY, mode: Implied, cycles: 4 }); 

    //Shifts
    table.insert(0x0A, OpcodeInfo { operation: ShiftLeft, mode: Implied, cycles: 2 });
    table.insert(0x06, OpcodeInfo { operation: ShiftLeft, mode: DirectPage, cycles: 5 });
    table.insert(0x0E, OpcodeInfo { operation: ShiftLeft, mode: Absolute, cycles: 6 });

    table.insert(0x4A, OpcodeInfo { operation: ShiftRight, mode: Implied, cycles: 2 });
    table.insert(0x46, OpcodeInfo { operation: ShiftRight, mode: DirectPage, cycles: 5 });
    table.insert(0x4E, OpcodeInfo { operation: ShiftRight, mode: Absolute, cycles: 6 });

    //Subroutines
    table.insert(0x20, OpcodeInfo { operation: JumpSubroutine, mode: Absolute, cycles: 6 });
    table.insert(0x60, OpcodeInfo { operation: ReturnFromSubroutine, mode: Implied, cycles: 6 });
    table.insert(0x40, OpcodeInfo { operation: ReturnFromInterrupt, mode: Implied, cycles: 6 });
    table.insert(0x00, OpcodeInfo { operation: SoftwareInterrupt, mode: Implied, cycles: 7 });

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