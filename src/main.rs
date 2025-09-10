use std::fs;
use std::io::Read;

mod cpu;
mod memory;

fn main (){
    println!("Iniciando emulador SNES...");

    let test_rom = create_test_rom();
    let mut memory = memory::Memory::new(test_rom);
    let mut cpu = cpu::Cpu::new();

    println!("ROM Carregada: {}", memory.get_rom_title());
    println!("Tipo de ROM: {:?}", memory.rom_type);
    println!("Tamanho SRAM: {} bytes", memory.sram_size);

    for _ in 0..10 {
        cpu.step(&mut memory);
    }

    println!("Emulador SNES finalizado.");
}

fn create_test_rom() -> Vec<u8> {
    let mut rom = vec![0xEA; 0x10000]; // ROM preenchida com NOPs (0xEA)
    
    // Adicionar header LoROM válido
    let header_start = 0x7FC0;
    let title = b"SNES EMU TEST       ";
    rom[header_start..header_start + 21].copy_from_slice(title);
    
    // Checksum válido
    rom[header_start + 0x1C] = 0x34;
    rom[header_start + 0x1D] = 0x12;
    rom[header_start + 0x1E] = 0xCB;
    rom[header_start + 0x1F] = 0xED;
    
    // SRAM size (32KB)
    rom[0x7FD8] = 0x03;
    
    rom
}