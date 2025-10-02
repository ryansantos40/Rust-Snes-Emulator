use std::fs;
use std::io::Read;
use system::System;

mod cpu;
mod memory;
mod opcodes;
mod ppu;
mod system;

fn main (){
    println!("Iniciando emulador SNES...");

    let test_rom = create_test_rom();
    let mut system = System::new(test_rom);

    println!("ROM Carregada: {}", system.memory.get_rom_title());
    println!("Tipo de ROM: {:?}", system.memory.rom_type);
    println!("Tamanho SRAM: {} bytes", system.memory.sram_size);
    println!("Estado inicial do CPU: {}", system.cpu.get_register_state());
    println!("Estado inicial da PPU: Scanline {}, Cycle {}", system.get_scanline(), system.get_ppu().cycle);

    println!("Executando alguns ciclos do CPU...");
    for i in 0..10 {
        let old_state = system.get_cpu_state();
        
        let cycles = system.step();
        
        println!("Instrução {}: {} ({}c) -> {}", i+1, old_state, cycles, system.get_cpu_state());
        
        if system.frame_ready() {
            println!("  └─ Frame PPU pronto! Scanline: {}, Cycle: {}", 
                     system.get_scanline(), 
                     system.get_ppu().cycle);
        }
        
        if system.is_vblank() {
            println!("  └─ PPU em VBlank!");
        }
    }

    println!("\n=== Estatísticas Finais ===");
    println!("Total de ciclos executados: {}", system.cpu.cycles);
    println!("PPU - Scanline: {}, Cycle: {}", system.get_scanline(), system.get_ppu().cycle);
    println!("Emulador SNES finalizado.");
}

fn create_test_rom() -> Vec<u8> {
    let mut rom = vec![0xEA; 0x10000]; // NOPs
    
    // Programa de teste mais interessante - Corrigido para rom[0x0000] (mapeia para $00:8000)
    rom[0x0000] = 0x18;       // CLC
    rom[0x0001] = 0xA9; rom[0x0002] = 0x42; // LDA #$42
    rom[0x0003] = 0x8D; rom[0x0004] = 0x00; rom[0x0005] = 0x30; // STA $3000
    rom[0x0006] = 0xA2; rom[0x0007] = 0x10; // LDX #$10
    rom[0x0008] = 0xA0; rom[0x0009] = 0x20; // LDY #$20
    rom[0x000A] = 0x38;       // SEC
    rom[0x000B] = 0x4C; rom[0x000C] = 0x00; rom[0x000D] = 0x80; // JMP $8000
    
    // Header...
    let header_start = 0x7FC0;
    let title = b"SNES EMU TEST        ";
    rom[header_start..header_start + 21].copy_from_slice(title);
    
    rom[header_start + 0x1C] = 0x34;
    rom[header_start + 0x1D] = 0x12;
    rom[header_start + 0x1E] = 0xCB;
    rom[header_start + 0x1F] = 0xED;
    rom[0x7FD8] = 0x03;
    
    rom
}