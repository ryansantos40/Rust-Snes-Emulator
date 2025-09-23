use snes_emulator::{Cpu, Memory};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Uso: cargo run --example rom_runner <rom_file.smc>");
        println!("Exemplo: cargo run --example rom_runner test_adc.smc");
        return;
    }
    
    let rom_name = &args[1];
    let rom_path = if rom_name.starts_with("test_roms/") {
        rom_name.to_string()
    } else {
        format!("test_roms/{}", rom_name)
    };
    
    println!("Carregando ROM: {}", rom_path);
    
    let rom_data = match fs::read(&rom_path) {
        Ok(mut data) => {
            // Remove header SMC se presente
            if data.len() % 1024 == 512 {
                println!("Removendo header SMC...");
                data.drain(0..512);
            }
            data
        },
        Err(e) => {
            eprintln!("Erro ao carregar ROM: {}", e);
            return;
        }
    };
    
    let mut memory = Memory::new(rom_data);
    let mut cpu = Cpu::new();
    
    // Configura reset vector
    let reset_low = memory.read(0x00FFFC) as u32;
    let reset_high = memory.read(0x00FFFD) as u32;
    cpu.pc = (reset_high << 8) | reset_low;
    
    println!("=== INFORMAÇÕES DA ROM ===");
    println!("Título: {}", memory.get_rom_title());
    println!("Tipo: {:?}", memory.rom_type);
    println!("Reset Vector: ${:04X}", cpu.pc);
    println!("Estado inicial: {}", cpu.get_register_state());
    
    println!("\n=== EXECUÇÃO ===");
    
    let max_instructions = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    
    for i in 0..max_instructions {
        let current_pc = cpu.pc;
        let opcode = memory.read(current_pc);
        let old_state = cpu.get_register_state();
        
        match cpu.step(&mut memory) {
            Ok(cycles) => {
                // Log detalhado das primeiras instruções
                if i < 50 {
                    println!("{:4}: ${:04X}: {:02X} {} -> {} ({}c)", 
                             i + 1, current_pc, opcode, old_state, cpu.get_register_state(), cycles);
                } else if i % 100 == 0 {
                    // Log esporádico para instruções posteriores
                    println!("{:4}: ${:04X}: {:02X} - {}", 
                             i + 1, current_pc, opcode, cpu.get_register_state());
                }
                
                // Detecta padrões de finalização
                if opcode == 0x00 {  // BRK
                    println!("\nBRK executado! Possível fim do programa.");
                    break;
                }
                
                if current_pc == cpu.pc {  // Loop infinito
                    println!("\nLoop infinito detectado em ${:04X}", current_pc);
                    break;
                }
                
            },
            Err(e) => {
                println!("\nERRO em ${:04X}: {:02X} - {}", current_pc, opcode, e);
                break;
            }
        }
    }
    
    println!("\n=== RESULTADO ===");
    println!("Estado final: {}", cpu.get_register_state());
    println!("Ciclos totais: {}", cpu.cycles);
}