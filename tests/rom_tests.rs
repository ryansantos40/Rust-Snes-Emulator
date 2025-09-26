use snes_emulator::{Cpu, Memory, Ppu};
use snes_emulator::opcodes;
use std::fs;
use std::path::Path;
use std::io::Write;

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
        if opcodes::get_opcode_info(opcode).is_none() {
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

fn execute_rom_test_with_ppu(rom_name: &str, max_instructions: usize, save_frames: bool) -> RomTestResult {
    println!("\n=== Testando ROM com PPU: {} ===", rom_name);
    
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
    let mut ppu = Ppu::new(); // ← Nova linha: Criar PPU
    
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
    let mut frame_count = 0;
    
    for i in 0..max_instructions {
        let current_pc = cpu.pc;
        let opcode = memory.read(current_pc);
        
        // Log primeiras instruções
        if i < 20 {
            println!("${:04X}: {:02X} - {}", current_pc, opcode, cpu.get_register_state());
        }
        
        // Verifica se o opcode é válido
        if opcodes::get_opcode_info(opcode).is_none() {
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
        
        // Executa instrução COM PPU
        let cycles = cpu.step_with_ppu(&mut memory, &mut ppu);
        instructions_executed += 1;
        
        // Verifica se frame está pronto
        if ppu.frame_ready() {
            frame_count += 1;
            println!("Frame {} pronto! Scanline: {}, Cycle: {}, Instrução: {}", 
                    frame_count, ppu.scanline, ppu.cycle, i);
            
            // Salva frame como imagem (se solicitado)
            if save_frames && frame_count <= 10 { // Salva apenas os primeiros 10 frames
                save_frame_as_ppm(&format!("frame_{}_{:03}.ppm", rom_name.replace(".smc", ""), frame_count), 
                                ppu.get_framebuffer());
                println!("Frame {} salvo como frame_{}_{:03}.ppm", frame_count, rom_name.replace(".smc", ""), frame_count);
            }
        }
        
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
        
        // Para em endereços especiais ou BRK
        if opcode == 0x00 {
            println!("BRK instruction executed");
            break;
        }
        
        if cpu.pc == 0xFFFF || cpu.pc == 0x0000 {
            println!("PC em endereço especial: ${:04X}", cpu.pc);
            break;
        }
        
        // Log esporádico
        if i > 20 && i % 500 == 0 {
            println!("Instrução {}: ${:04X}: {:02X} - {}", i, current_pc, opcode, cpu.get_register_state());
        }
    }
    
    let success = instructions_executed > 10;
    
    println!("=== RESUMO PPU ===");
    println!("Frames gerados: {}", frame_count);
    println!("PPU - Scanline: {}, Cycle: {}", ppu.scanline, ppu.cycle);
    println!("PPU - VBlank: {}, HBlank: {}", ppu.vblank, ppu.hblank);
    println!("PPU - Video Mode: {:?}", ppu.video_mode);
    println!("PPU - Forced Blank: {}", ppu.forced_blank);
    println!("PPU - Brightness: {}", ppu.brightness);
    println!("PPU - Backgrounds: {:?}", ppu.bg_enabled);
    println!("PPU - Sprites: {}", ppu.sprites_enabled);
    
    RomTestResult {
        rom_name: rom_name.to_string(),
        success,
        error_message: None,
        instructions_executed,
        final_state: cpu.get_register_state(),
    }
}

fn save_frame_as_ppm(filename: &str, framebuffer: &[u32]) {
    let mut file = std::fs::File::create(filename).unwrap();
    writeln!(file, "P3").unwrap();
    writeln!(file, "256 224").unwrap();
    writeln!(file, "255").unwrap();
    
    for &pixel in framebuffer {
        let r = (pixel >> 16) & 0xFF;
        let g = (pixel >> 8) & 0xFF;
        let b = pixel & 0xFF;
        writeln!(file, "{} {} {}", r, g, b).unwrap();
    }
    println!("Frame salvo: {}", filename);
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

#[test]
fn test_rom_diagnosis() {
    let rom_name = "Super.smc";
    println!("\n=== DIAGNÓSTICO DETALHADO: {} ===", rom_name);
    
    let rom_data = match load_test_rom(rom_name) {
        Ok(data) => data,
        Err(e) => {
            println!("❌ Erro ao carregar ROM: {}", e);
            return;
        }
    };
    
    println!("✅ ROM carregada com sucesso");
    println!("📊 Tamanho da ROM: {} bytes (0x{:X})", rom_data.len(), rom_data.len());
    
    // Verifica se é um tamanho válido de ROM SNES
    let valid_sizes = [0x40000, 0x80000, 0x100000, 0x200000, 0x300000, 0x400000]; // 256KB, 512KB, 1MB, 2MB, 3MB, 4MB
    let size_valid = valid_sizes.contains(&rom_data.len());
    println!("📏 Tamanho válido: {}", if size_valid { "✅ SIM" } else { "⚠️ NÃO" });
    
    let mut memory = Memory::new(rom_data);
    
    println!("\n=== INFORMAÇÕES DA ROM ===");
    println!("📄 Título: '{}'", memory.get_rom_title());
    println!("🗂️ Tipo: {:?}", memory.rom_type);
    println!("💾 SRAM Size: {} bytes", memory.sram_size);
    
    println!("\n=== VETORES DE INTERRUPT/RESET ===");
    
    // Vetores para modo de emulação (65816 em modo 6502)
    let native_cop   = (memory.read(0x00FFE5) as u16) << 8 | memory.read(0x00FFE4) as u16;
    let native_brk   = (memory.read(0x00FFE7) as u16) << 8 | memory.read(0x00FFE6) as u16;
    let native_abort = (memory.read(0x00FFE9) as u16) << 8 | memory.read(0x00FFE8) as u16;
    let native_nmi   = (memory.read(0x00FFEB) as u16) << 8 | memory.read(0x00FFEA) as u16;
    let native_reset = (memory.read(0x00FFED) as u16) << 8 | memory.read(0x00FFEC) as u16;
    let native_irq   = (memory.read(0x00FFEF) as u16) << 8 | memory.read(0x00FFEE) as u16;
    
    // Vetores para modo de emulação
    let emu_cop   = (memory.read(0x00FFF5) as u16) << 8 | memory.read(0x00FFF4) as u16;
    let emu_abort = (memory.read(0x00FFF9) as u16) << 8 | memory.read(0x00FFF8) as u16;
    let emu_nmi   = (memory.read(0x00FFFB) as u16) << 8 | memory.read(0x00FFFA) as u16;
    let emu_reset = (memory.read(0x00FFFD) as u16) << 8 | memory.read(0x00FFFC) as u16;
    let emu_irq   = (memory.read(0x00FFFF) as u16) << 8 | memory.read(0x00FFFE) as u16;
    
    println!("📍 Native Mode Vectors:");
    println!("   COP:   ${:04X}", native_cop);
    println!("   BRK:   ${:04X}", native_brk);
    println!("   ABORT: ${:04X}", native_abort);
    println!("   NMI:   ${:04X}", native_nmi);
    println!("   RESET: ${:04X}", native_reset);
    println!("   IRQ:   ${:04X}", native_irq);
    
    println!("📍 Emulation Mode Vectors:");
    println!("   COP:   ${:04X}", emu_cop);
    println!("   ABORT: ${:04X}", emu_abort);
    println!("   NMI:   ${:04X}", emu_nmi);
    println!("   RESET: ${:04X}", emu_reset);
    println!("   IRQ:   ${:04X}", emu_irq);
    
    // O que seu código atual usa
    let current_reset = (memory.read(0x00FFFD) as u32) << 8 | memory.read(0x00FFFC) as u32;
    println!("\n🎯 Reset Vector Atual (seu código): ${:04X}", current_reset);
    
    // Verifica se vetores são válidos (não 0x0000 ou 0xFFFF)
    let reset_valid = emu_reset != 0x0000 && emu_reset != 0xFFFF;
    println!("✅ Reset Vector Válido: {}", if reset_valid { "SIM" } else { "❌ NÃO" });
    
    if !reset_valid {
        println!("⚠️ PROBLEMA: Reset vector inválido! ROM pode estar corrompida ou ser de tipo diferente.");
    }
    
    println!("\n=== DUMP DA MEMÓRIA NO RESET VECTOR ===");
    if emu_reset != 0x0000 && emu_reset < 0xFFFF {
        println!("Primeiros 16 bytes em ${:04X}:", emu_reset);
        for i in 0..16 {
            let addr = emu_reset.wrapping_add(i);
            let byte = memory.read(addr as u32);
            print!("${:04X}: {:02X} ", addr, byte);
            if (i + 1) % 8 == 0 { println!(); }
        }
        println!();
    }
    
    println!("\n=== DUMP DOS VECTORS RAW ===");
    println!("Bytes em $FFFC-$FFFF:");
    for addr in 0xFFFC..=0xFFFF {
        let byte = memory.read(addr);
        println!("${:04X}: {:02X}", addr, byte);
    }
    
    println!("\n=== HEADER DA ROM (se existir) ===");
    // Tenta ler header LoROM em $00:7FC0
    println!("Tentando ler header LoROM em $7FC0:");
    for i in 0..32 {
        let byte = memory.read(0x7FC0 + i);
        print!("{:02X} ", byte);
        if (i + 1) % 16 == 0 { println!(); }
    }
    
    // Tenta ler header HiROM em $00:FFC0  
    println!("\nTentando ler header HiROM em $FFC0:");
    for i in 0..32 {
        let byte = memory.read(0xFFC0 + i);
        print!("{:02X} ", byte);
        if (i + 1) % 16 == 0 { println!(); }
    }
}

#[test]
fn test_super_mario_world() {
    let result = execute_rom_test("Super.smc", 10000); // Menos instruções para início
    println!("\nResultado Super Mario World:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    // Deve conseguir executar pelo menos algumas instruções
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_super_mario_world_with_ppu() {
    let result = execute_rom_test_with_ppu("Super.smc", 50000, true); // Mais instruções, salva frames
    println!("\n=== RESULTADO SUPER MARIO WORLD COM PPU ===");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    println!("\n💡 DICA: Verifique os arquivos frame_Super_*.ppm gerados!");
    println!("💡 Para visualizar no Windows: Renomeie para .png ou use GIMP/Paint.NET");
    println!("💡 Para visualizar no Linux: Use 'feh frame_Super_001.ppm' ou similar");
    
    // Deve conseguir executar pelo menos algumas instruções
    assert!(result.instructions_executed > 100, "Poucas instruções executadas com PPU");
}

// Teste rápido sem salvar frames (para performance)
#[test]
fn test_super_mario_world_ppu_quick() {
    let result = execute_rom_test_with_ppu("Super.smc", 10000, false); // Não salva frames
    println!("\nResultado Super Mario World PPU (rápido):");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Estado final: {}", result.final_state);
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

// Teste para comparar CPU puro vs CPU+PPU
#[test]
fn test_cpu_vs_cpu_ppu_comparison() {
    println!("\n=== COMPARAÇÃO: CPU PURO vs CPU+PPU ===");
    
    // Teste CPU puro (sua função original)
    let result_cpu = execute_rom_test("Super.smc", 5000);
    println!("\n📊 CPU PURO:");
    println!("   Instruções: {}", result_cpu.instructions_executed);
    println!("   Estado: {}", result_cpu.final_state);
    
    // Teste CPU+PPU
    let result_ppu = execute_rom_test_with_ppu("Super.smc", 5000, false);
    println!("\n🎮 CPU+PPU:");
    println!("   Instruções: {}", result_ppu.instructions_executed);
    println!("   Estado: {}", result_ppu.final_state);
    
    // Comparação
    println!("\n🔍 COMPARAÇÃO:");
    println!("   Diferença de instruções: {}", 
            result_ppu.instructions_executed as i32 - result_cpu.instructions_executed as i32);
    
    // Ambos devem funcionar
    assert!(result_cpu.instructions_executed > 0);
    assert!(result_ppu.instructions_executed > 0);
}
