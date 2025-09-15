use snes_emulator::{Cpu, Memory};

fn create_test_memory_with_program(program: &[u8]) -> Memory {
    let mut rom = vec![0xEA; 0x10000]; // Fill with NOPs
    
    // Copy program to ROM start (posição 0x0000 no array, que mapeia para $00:8000)
    for (i, &byte) in program.iter().enumerate() {
        if i < rom.len() {
            rom[i] = byte; // ← CORREÇÃO: era rom[0x8000 + i] = byte;
        }
    }
    
    // Add minimal header
    let header_start = 0x7FC0;
    let title = b"CPU TEST             ";
    rom[header_start..header_start + 21].copy_from_slice(title);
    
    Memory::new(rom)
}

#[test]
fn test_cpu_initialization() {
    let cpu = Cpu::new();
    
    assert_eq!(cpu.pc, 0x008000);
    assert_eq!(cpu.a, 0x0000);
    assert_eq!(cpu.x, 0x0000);
    assert_eq!(cpu.y, 0x0000);
    assert_eq!(cpu.sp, 0x01FF);
    assert_eq!(cpu.p, 0x34);
    assert_eq!(cpu.m_flag, true);
    assert_eq!(cpu.x_flag, true);
    assert_eq!(cpu.e_flag, true);
}

#[test]
fn test_lda_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x42, // LDA #$42
        0xA9, 0x00, // LDA #$00
    ]);
    
    // LDA #$42
    cpu.step(&mut memory);
    assert_eq!(cpu.a & 0xFF, 0x42);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
    
    // LDA #$00
    cpu.step(&mut memory);
    assert_eq!(cpu.a & 0xFF, 0x00);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
}

#[test]
fn test_ldx_ldy_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA2, 0x33, // LDX #$33
        0xA0, 0x44, // LDY #$44
    ]);
    
    cpu.step(&mut memory);
    assert_eq!(cpu.x & 0xFF, 0x33);
    
    cpu.step(&mut memory);
    assert_eq!(cpu.y & 0xFF, 0x44);
}

#[test]
fn test_sta_absolute() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x55,       // LDA #$55
        0x8D, 0x00, 0x30, // STA $3000
    ]);
    
    cpu.step(&mut memory); // LDA
    cpu.step(&mut memory); // STA
    
    assert_eq!(memory.read(0x003000), 0x55);
}

#[test]
fn test_flag_operations() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x18, // CLC
        0x38, // SEC  
        0x58, // CLI
        0x78, // SEI
    ]);
    
    // Initially carry should be clear (from reset state)
    cpu.step(&mut memory); // CLC
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
    
    cpu.step(&mut memory); // SEC
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    
    cpu.step(&mut memory); // CLI
    assert_eq!(cpu.get_flag(Cpu::FLAG_IRQ), false);
    
    cpu.step(&mut memory); // SEI
    assert_eq!(cpu.get_flag(Cpu::FLAG_IRQ), true);
}

#[test]
fn test_jmp_absolute() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x4C, 0x10, 0x80, // JMP $8010
    ]);
    
    cpu.step(&mut memory);
    assert_eq!(cpu.pc, 0x008010);
}

#[test]
fn test_branch_instructions() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x00,       // LDA #$00 (sets zero flag)
        0xF0, 0x02,       // BEQ +2 (should branch)
        0xA9, 0xFF,       // LDA #$FF (should be skipped)
        0xA9, 0x11,       // LDA #$11 (should execute)
    ]);
    
    cpu.step(&mut memory); // LDA #$00
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    
    let old_pc = cpu.pc;
    cpu.step(&mut memory); // BEQ +2
    assert_eq!(cpu.pc, old_pc + 2 + 2); // Branch taken, skips LDA #$FF
    
    cpu.step(&mut memory); // LDA #$11
    assert_eq!(cpu.a & 0xFF, 0x11);
}

#[test]
fn test_negative_flag() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x80, // LDA #$80 (negative in 8-bit)
    ]);
    
    cpu.step(&mut memory);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
}

#[test]
fn test_cycle_counting() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xEA,       // NOP (2 cycles)
        0xA9, 0x42, // LDA #$42 (2 cycles)
    ]);
    
    let cycles1 = cpu.step(&mut memory);
    assert_eq!(cycles1, 2);
    assert_eq!(cpu.cycles, 2);
    
    let cycles2 = cpu.step(&mut memory);
    assert_eq!(cycles2, 2);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn test_reset() {
    let mut cpu = Cpu::new();
    
    // Modify some state
    cpu.a = 0x1234;
    cpu.x = 0x5678;
    cpu.pc = 0x123456;
    cpu.cycles = 1000;
    
    cpu.reset();
    
    assert_eq!(cpu.a, 0x0000);
    assert_eq!(cpu.x, 0x0000);
    assert_eq!(cpu.pc, 0x008000);
    assert_eq!(cpu.cycles, 0);
}


#[test]
fn debug_cpu_execution() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x42, // LDA #$42
    ]);
    
    println!("=== DEBUG CPU EXECUTION ===");
    println!("CPU inicial: PC={:06X}, A={:04X}, P={:02X}", cpu.pc, cpu.a, cpu.p);
    println!("Memória em $00:8000: {:02X}", memory.read(0x008000));
    println!("Memória em $00:8001: {:02X}", memory.read(0x008001));
    println!("ROM[0]: {:02X}", memory.rom[0]);
    println!("ROM[1]: {:02X}", memory.rom[1]);
    
    let cycles = cpu.step(&mut memory);
    println!("Após LDA #$42: PC={:06X}, A={:04X}, P={:02X}, cycles={}", 
             cpu.pc, cpu.a, cpu.p, cycles);
    
    assert_eq!(cpu.a & 0xFF, 0x42);
}

#[test]
fn debug_sta_absolute() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x55,       // LDA #$55
        0x8D, 0x00, 0x30, // STA $3000
    ]);
    
    println!("=== DEBUG STA ABSOLUTE ===");
    println!("CPU inicial: PC={:06X}, A={:04X}", cpu.pc, cpu.a);
    
    // Execute LDA
    cpu.step(&mut memory);
    println!("Após LDA #$55: PC={:06X}, A={:04X}", cpu.pc, cpu.a);
    
    // Execute STA
    println!("Próxima instrução: {:02X} {:02X} {:02X}", 
             memory.read(cpu.pc), memory.read(cpu.pc + 1), memory.read(cpu.pc + 2));
    
    cpu.step(&mut memory);
    println!("Após STA $3000: PC={:06X}, A={:04X}", cpu.pc, cpu.a);
    
    println!("Valor em $3000: {:02X}", memory.read(0x003000));
    
    // Debug da memória
    println!("Testando escrita direta na memória:");
    memory.write(0x003000, 0xAB);
    println!("Após escrita direta, valor em $3000: {:02X}", memory.read(0x003000));
    
    assert_eq!(memory.read(0x003000), 0xAB);
}