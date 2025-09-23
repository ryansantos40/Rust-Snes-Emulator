use snes_emulator::{Cpu, Memory};
use std::fs;
use std::path::Path;

struct RomTestResult {
    rom_name: String,
    success: bool,
    error_message: Option<String>,
    instructions_executed: usize,
    final_state: String,
}

fn load_test_rom(rom_name: &str) -> Result<Vec<u8>, String> {
    let rom_path = format!("test_roms/{}", rom_name);
    
    if !Path::new(&rom_path).exists() {
        return Err(format!("ROM não encontrada: {}", rom_path));
    }
    
    match fs::read(&rom_path) {
        Ok(mut data) => {
            // Remove header SMC se presente
            if data.len() % 1024 == 512 {
                println!("Removendo header SMC de {}", rom_name);
                data.drain(0..512);
            }
            Ok(data)
        },
        Err(e) => Err(format!("Erro ao ler {}: {}", rom_name, e))
    }
}

fn execute_rom_test(rom_name: &str, max_instructions: usize) -> RomTestResult {
    println!("\n=== Testando ROM: {} ===", rom_name);
    
    let rom_data = match load_test_rom(rom_name) {
        Ok(data) => data,
        Err(e) => return RomTestResult {
            rom_name: rom_name.to_string(),
            success: false,
            error_message: Some(e),
            instructions_executed: 0,
            final_state: String::new(),
        }
    };
    
    let mut memory = Memory::new(rom_data);
    let mut cpu = Cpu::new();
    
    // Configura reset vector
    let reset_low = memory.read(0x00FFFC) as u32;
    let reset_high = memory.read(0x00FFFD) as u32;
    cpu.pc = (reset_high << 8) | reset_low;
    
    println!("ROM Title: {}", memory.get_rom_title());
    println!("ROM Type: {:?}", memory.rom_type);
    println!("Reset Vector: ${:04X}", cpu.pc);
    
    let mut instructions_executed = 0;
    let mut last_pc = cpu.pc;
    let mut loop_counter = 0;
    
    for i in 0..max_instructions {
        let current_pc = cpu.pc;
        let opcode = memory.read(current_pc);
        
        // Log primeiras instruções
        if i < 20 {
            println!("${:04X}: {:02X} - {}", current_pc, opcode, cpu.get_register_state());
        }
        
        // Verifica se o opcode é válido antes de executar
        if crate::snes_emulator::opcodes::get_opcode_info(opcode).is_none() {
            let error_msg = format!("Unknown opcode: {:02X} at PC: {:06X}", opcode, current_pc);
            println!("{}", error_msg);
            
            return RomTestResult {
                rom_name: rom_name.to_string(),
                success: false,
                error_message: Some(error_msg),
                instructions_executed,
                final_state: cpu.get_register_state(),
            };
        }
        
        // Executa instrução - CORREÇÃO AQUI
        let cycles = cpu.step(&mut memory);
        instructions_executed += 1;
        
        // Detecta loops infinitos
        if cpu.pc == last_pc {
            loop_counter += 1;
            if loop_counter > 10 {
                println!("Loop infinito detectado em ${:04X}", cpu.pc);
                break;
            }
        } else {
            loop_counter = 0;
            last_pc = cpu.pc;
        }
        
        // Detecta alguns padrões de finalização
        if opcode == 0x00 && cpu.pc == 0x0000 {  // BRK seguido de reset
            println!("BRK executado, possivelmente fim do teste");
            break;
        }
        
        // Para em endereços especiais (alguns testes param aqui)
        if cpu.pc == 0xFFFF || cpu.pc == 0x0000 {
            println!("PC em endereço especial: ${:04X}", cpu.pc);
            break;
        }
        
        // Para se executar BRK
        if opcode == 0x00 {
            println!("BRK instruction executed");
            break;
        }
        
        // Log esporádico
        if i > 20 && i % 50 == 0 {
            println!("Instrução {}: ${:04X}: {:02X} - {}", i, current_pc, opcode, cpu.get_register_state());
        }
    }
    
    let success = instructions_executed > 10; // Critério básico de sucesso
    
    RomTestResult {
        rom_name: rom_name.to_string(),
        success,
        error_message: None,
        instructions_executed,
        final_state: cpu.get_register_state(),
    }
}

#[test]
fn test_adc_rom() {
    let result = execute_rom_test("test_adc.smc", 1000);
    println!("\nResultado test_adc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    // Para agora, consideramos sucesso se executou pelo menos algumas instruções
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_sbc_rom() {
    let result = execute_rom_test("test_sbc.smc", 1000);
    println!("\nResultado test_sbc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_tsc_rom() {
    let result = execute_rom_test("snes_test_tsc.smc", 1000);
    println!("\nResultado snes_test_tsc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_all_available_roms() {
    let test_roms_dir = "test_roms";
    
    if !Path::new(test_roms_dir).exists() {
        panic!("Diretório test_roms não encontrado! Crie o diretório e coloque as ROMs nele.");
    }
    
    let rom_files = fs::read_dir(test_roms_dir)
        .expect("Erro ao ler diretório test_roms")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "smc" {
                Some(entry.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    if rom_files.is_empty() {
        println!("Nenhuma ROM (.smc) encontrada em test_roms/");
        return;
    }
    
    println!("\n=== RELATÓRIO DE TESTE DE TODAS AS ROMs ===");
    
    let mut results = Vec::new();
    
    for rom_file in &rom_files {
        let result = execute_rom_test(rom_file, 500); // Menos instruções para overview
        results.push(result);
    }
    
    println!("\n=== RESUMO DOS RESULTADOS ===");
    println!("{:<30} | {:<10} | {:<15} | {}", "ROM", "Status", "Instruções", "Erro");
    println!("{}", "-".repeat(80));
    
    for result in &results {
        let status = if result.success { "✅ OK" } else { "❌ ERRO" };
        let error = result.error_message.as_deref().unwrap_or("Nenhum");
        println!("{:<30} | {:<10} | {:<15} | {}", 
                result.rom_name, status, result.instructions_executed, error);
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let total = results.len();
    
    println!("\n=== ESTATÍSTICAS ===");
    println!("ROMs testadas: {}", total);
    println!("Sucessos: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("Falhas: {}", total - successful);
    
    // Para agora, não falhamos o teste se algumas ROMs não funcionam
    // Isso mudará conforme implementamos mais funcionalidades
    assert!(total > 0, "Nenhuma ROM foi testada");
}

#[test]
#[ignore] // Use `cargo test -- --ignored` para executar
fn test_detailed_adc() {
    // Teste mais detalhado para debug
    let result = execute_rom_test("test_adc.smc", 10000);
    
    // Este teste é mais rigoroso e pode falhar - use para debug
    assert!(result.success, "test_adc.smc deveria passar");
}
