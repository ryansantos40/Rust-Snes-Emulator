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

#[test]
fn test_adc_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x18,       // CLC (clear carry)
        0xA9, 0x10, // LDA #$10
        0x69, 0x05, // ADC #$05
        0x69, 0xFF, // ADC #$FF (should cause carry)
    ]);
    
    cpu.step(&mut memory); // CLC
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
    
    cpu.step(&mut memory); // LDA #$10
    assert_eq!(cpu.a & 0xFF, 0x10);
    
    cpu.step(&mut memory); // ADC #$05
    assert_eq!(cpu.a & 0xFF, 0x15); // 0x10 + 0x05 = 0x15
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    
    cpu.step(&mut memory); // ADC #$FF
    assert_eq!(cpu.a & 0xFF, 0x14); // 0x15 + 0xFF = 0x114 (carry set, result 0x14)
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
}

#[test]
fn test_adc_with_carry() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x38,       // SEC (set carry)
        0xA9, 0x10, // LDA #$10
        0x69, 0x05, // ADC #$05 (should add carry too)
    ]);
    
    cpu.step(&mut memory); // SEC
    cpu.step(&mut memory); // LDA #$10
    cpu.step(&mut memory); // ADC #$05
    
    assert_eq!(cpu.a & 0xFF, 0x16); // 0x10 + 0x05 + 1 (carry) = 0x16
}

#[test]
fn test_sbc_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x38,       // SEC (set carry for proper SBC)
        0xA9, 0x20, // LDA #$20
        0xE9, 0x10, // SBC #$10
        0xE9, 0x20, // SBC #$20 (should cause borrow)
    ]);
    
    cpu.step(&mut memory); // SEC
    cpu.step(&mut memory); // LDA #$20
    
    cpu.step(&mut memory); // SBC #$10
    assert_eq!(cpu.a & 0xFF, 0x10); // 0x20 - 0x10 = 0x10
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true); // No borrow
    
    cpu.step(&mut memory); // SBC #$20
    assert_eq!(cpu.a & 0xFF, 0xF0); // 0x10 - 0x20 = -0x10 = 0xF0 (two's complement)
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false); // Borrow occurred
}

#[test]
fn test_inc_accumulator() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0xFE, // LDA #$FE
        0x1A,       // INC A
        0x1A,       // INC A (should wrap to 0)
    ]);
    
    cpu.step(&mut memory); // LDA #$FE
    
    cpu.step(&mut memory); // INC A
    assert_eq!(cpu.a & 0xFF, 0xFF);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
    
    cpu.step(&mut memory); // INC A (wrap to 0)
    assert_eq!(cpu.a & 0xFF, 0x00);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
}

#[test]
fn test_dec_accumulator() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x01, // LDA #$01
        0x3A,       // DEC A
        0x3A,       // DEC A (should wrap to 0xFF)
    ]);
    
    cpu.step(&mut memory); // LDA #$01
    
    cpu.step(&mut memory); // DEC A
    assert_eq!(cpu.a & 0xFF, 0x00);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    
    cpu.step(&mut memory); // DEC A (wrap to 0xFF)
    assert_eq!(cpu.a & 0xFF, 0xFF);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
}

#[test]
fn test_inc_memory() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x42,       // LDA #$42
        0x85, 0x10,       // STA $10 (store to direct page)
        0xE6, 0x10,       // INC $10
    ]);
    
    cpu.step(&mut memory); // LDA #$42
    cpu.step(&mut memory); // STA $10
    cpu.step(&mut memory); // INC $10
    
    assert_eq!(memory.read(0x000010), 0x43);
}

// === LOGICAL OPERATION TESTS ===

#[test]
fn test_and_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0xFF, // LDA #$FF
        0x29, 0x0F, // AND #$0F
        0x29, 0x00, // AND #$00 (should set zero flag)
    ]);
    
    cpu.step(&mut memory); // LDA #$FF
    
    cpu.step(&mut memory); // AND #$0F
    assert_eq!(cpu.a & 0xFF, 0x0F);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    
    cpu.step(&mut memory); // AND #$00
    assert_eq!(cpu.a & 0xFF, 0x00);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
}

#[test]
fn test_or_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x0F, // LDA #$0F
        0x09, 0xF0, // ORA #$F0
    ]);
    
    cpu.step(&mut memory); // LDA #$0F
    cpu.step(&mut memory); // ORA #$F0
    
    assert_eq!(cpu.a & 0xFF, 0xFF); // 0x0F | 0xF0 = 0xFF
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
}

#[test]
fn test_xor_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0xFF, // LDA #$FF
        0x49, 0xFF, // EOR #$FF (should result in 0)
    ]);
    
    cpu.step(&mut memory); // LDA #$FF
    cpu.step(&mut memory); // EOR #$FF
    
    assert_eq!(cpu.a & 0xFF, 0x00); // 0xFF ^ 0xFF = 0x00
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
}

// === COMPARE OPERATION TESTS ===

#[test]
fn test_cmp_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x42, // LDA #$42
        0xC9, 0x42, // CMP #$42 (equal)
        0xC9, 0x30, // CMP #$30 (A > operand)
        0xC9, 0x50, // CMP #$50 (A < operand)
    ]);
    
    cpu.step(&mut memory); // LDA #$42
    
    cpu.step(&mut memory); // CMP #$42 (equal)
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    
    cpu.step(&mut memory); // CMP #$30 (A > operand)
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
    
    cpu.step(&mut memory); // CMP #$50 (A < operand)
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
}

#[test]
fn test_cpx_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA2, 0x30, // LDX #$30
        0xE0, 0x30, // CPX #$30 (equal)
        0xE0, 0x20, // CPX #$20 (X > operand)
    ]);
    
    cpu.step(&mut memory); // LDX #$30
    
    cpu.step(&mut memory); // CPX #$30
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    
    cpu.step(&mut memory); // CPX #$20
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
}

#[test]
fn test_cpy_immediate() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA0, 0x25, // LDY #$25
        0xC0, 0x25, // CPY #$25 (equal)
        0xC0, 0x30, // CPY #$30 (Y < operand)
    ]);
    
    cpu.step(&mut memory); // LDY #$25
    
    cpu.step(&mut memory); // CPY #$25
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    
    cpu.step(&mut memory); // CPY #$30
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
}

// === SHIFT OPERATION TESTS ===

#[test]
fn test_asl_accumulator() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x40, // LDA #$40
        0x0A,       // ASL A
        0x0A,       // ASL A (should set carry)
    ]);
    
    cpu.step(&mut memory); // LDA #$40
    
    cpu.step(&mut memory); // ASL A
    assert_eq!(cpu.a & 0xFF, 0x80);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
    
    cpu.step(&mut memory); // ASL A (should set carry)
    assert_eq!(cpu.a & 0xFF, 0x00);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
}

#[test]
fn test_lsr_accumulator() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x81, // LDA #$81
        0x4A,       // LSR A
        0x4A,       // LSR A (should set carry)
    ]);
    
    cpu.step(&mut memory); // LDA #$81
    
    cpu.step(&mut memory); // LSR A
    assert_eq!(cpu.a & 0xFF, 0x40);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true); // LSB was 1
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
    
    cpu.step(&mut memory); // LSR A
    assert_eq!(cpu.a & 0xFF, 0x20);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), false); // LSB was 0
}

#[test]
fn test_asl_memory() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x40,       // LDA #$40
        0x85, 0x10,       // STA $10
        0x06, 0x10,       // ASL $10
    ]);
    
    cpu.step(&mut memory); // LDA #$40
    cpu.step(&mut memory); // STA $10
    cpu.step(&mut memory); // ASL $10
    
    assert_eq!(memory.read(0x000010), 0x80);
}

// === OVERFLOW FLAG TESTS ===

#[test]
fn test_adc_overflow() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x18,       // CLC
        0xA9, 0x7F, // LDA #$7F (127, maximum positive 8-bit)
        0x69, 0x01, // ADC #$01 (should cause overflow)
    ]);
    
    cpu.step(&mut memory); // CLC
    cpu.step(&mut memory); // LDA #$7F
    cpu.step(&mut memory); // ADC #$01
    
    assert_eq!(cpu.a & 0xFF, 0x80); // 127 + 1 = 128 (negative in signed)
    assert_eq!(cpu.get_flag(Cpu::FLAG_OVERFLOW), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), true);
}

#[test]
fn test_sbc_overflow() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x38,       // SEC
        0xA9, 0x80, // LDA #$80 (-128, minimum negative 8-bit)
        0xE9, 0x01, // SBC #$01 (should cause overflow)
    ]);
    
    cpu.step(&mut memory); // SEC
    cpu.step(&mut memory); // LDA #$80
    cpu.step(&mut memory); // SBC #$01
    
    assert_eq!(cpu.a & 0xFF, 0x7F); // -128 - 1 = 127 (overflow to positive)
    assert_eq!(cpu.get_flag(Cpu::FLAG_OVERFLOW), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_NEGATIVE), false);
}

// === COMPLEX OPERATION TESTS ===

#[test]
fn test_complex_arithmetic_sequence() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0x18,       // CLC
        0xA9, 0x10, // LDA #$10
        0x69, 0x05, // ADC #$05  (A = 0x15)
        0x29, 0x0F, // AND #$0F  (A = 0x05)
        0x09, 0x20, // ORA #$20  (A = 0x25)
        0x49, 0xFF, // EOR #$FF  (A = 0xDA)
        0xC9, 0xDA, // CMP #$DA  (should set zero and carry)
    ]);
    
    cpu.step(&mut memory); // CLC
    cpu.step(&mut memory); // LDA #$10
    cpu.step(&mut memory); // ADC #$05
    assert_eq!(cpu.a & 0xFF, 0x15);
    
    cpu.step(&mut memory); // AND #$0F
    assert_eq!(cpu.a & 0xFF, 0x05);
    
    cpu.step(&mut memory); // ORA #$20
    assert_eq!(cpu.a & 0xFF, 0x25);
    
    cpu.step(&mut memory); // EOR #$FF
    assert_eq!(cpu.a & 0xFF, 0xDA);
    
    cpu.step(&mut memory); // CMP #$DA
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
    assert_eq!(cpu.get_flag(Cpu::FLAG_CARRY), true);
}

#[test]
fn test_memory_operations_sequence() {
    let mut cpu = Cpu::new();
    let mut memory = create_test_memory_with_program(&[
        0xA9, 0x42,       // LDA #$42
        0x8D, 0x00, 0x30, // STA $3000
        0xEE, 0x00, 0x30, // INC $3000
        0xAD, 0x00, 0x30, // LDA $3000
        0xC9, 0x43,       // CMP #$43
    ]);
    
    cpu.step(&mut memory); // LDA #$42
    cpu.step(&mut memory); // STA $3000
    cpu.step(&mut memory); // INC $3000
    
    assert_eq!(memory.read(0x003000), 0x43);
    
    cpu.step(&mut memory); // LDA $3000
    assert_eq!(cpu.a & 0xFF, 0x43);
    
    cpu.step(&mut memory); // CMP #$43
    assert_eq!(cpu.get_flag(Cpu::FLAG_ZERO), true);
}