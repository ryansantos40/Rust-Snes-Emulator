use snes_emulator::System;
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
    frames_generated: usize,
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

fn execute_rom_test(rom_name: &str, max_instructions: usize, save_frames: bool) -> RomTestResult {
    println!("\n=== Testando ROM: {} ===", rom_name);
    
    let rom_data = match load_test_rom(rom_name) {
        Ok(data) => data,
        Err(e) => return RomTestResult {
            rom_name: rom_name.to_string(),
            success: false,
            error_message: Some(e),
            instructions_executed: 0,
            final_state: String::new(),
            frames_generated: 0,
        }
    };
    
    let mut system = System::new(rom_data);
    
    // Configura reset vector
    let reset_low = system.memory.read(0x00FFFC) as u32;
    let reset_high = system.memory.read(0x00FFFD) as u32;
    system.cpu.pc = (reset_high << 8) | reset_low;
    
    println!("ROM Title: {}", system.memory.get_rom_title());
    println!("ROM Type: {:?}", system.memory.rom_type);
    println!("Reset Vector: ${:04X}", system.cpu.pc);
    println!("Estado inicial PPU: Scanline {}, Cycle {}", system.get_scanline(), system.get_ppu().cycle);
    
    let mut instructions_executed = 0;
    let mut last_pc = system.cpu.pc;
    let mut loop_counter = 0;
    let mut frame_count = 0;
    
    for i in 0..max_instructions {
        let current_pc = system.cpu.pc;
        let opcode = system.memory.read(current_pc);
        
        // Log primeiras instruções
        if i < 20 {
            println!("${:04X}: {:02X} - {} | PPU: L{} C{}", 
                     current_pc, opcode, system.get_cpu_state(),
                     system.get_scanline(), system.get_ppu().cycle);
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
                final_state: system.get_cpu_state(),
                frames_generated: frame_count,
            };
        }
        
        // Executa instrução (CPU + PPU integrados)
        let cycles = system.step();
        instructions_executed += 1;
        
        // Verifica se frame está pronto
        if system.frame_ready() {
            frame_count += 1;
            println!("Frame {} pronto! Scanline: {}, Cycle: {}, Instrução: {}", 
                    frame_count, system.get_scanline(), system.get_ppu().cycle, i);
            
            // Salva frame como imagem (se solicitado)
            if save_frames && frame_count <= 10 {
                save_frame_as_ppm(
                    &format!("frame_{}_{:03}.ppm", rom_name.replace(".smc", ""), frame_count), 
                    system.get_framebuffer().as_slice()
                );
                println!("Frame {} salvo como frame_{}_{:03}.ppm", frame_count, rom_name.replace(".smc", ""), frame_count);
            }
        }
        
        // Detecta VBlank
        if system.is_vblank() && i < 100 {
            println!("  └─ VBlank ativo na instrução {}", i + 1);
        }
        
        // Detecta loops infinitos
        if system.cpu.pc == last_pc {
            loop_counter += 1;
            if loop_counter > 10 {
                println!("Loop infinito detectado em ${:04X}", system.cpu.pc);
                break;
            }
        } else {
            loop_counter = 0;
            last_pc = system.cpu.pc;
        }
        
        // Para em endereços especiais ou BRK
        if opcode == 0x00 {
            println!("BRK instruction executed");
            break;
        }
        
        if system.cpu.pc == 0xFFFF || system.cpu.pc == 0x0000 {
            println!("PC em endereço especial: ${:04X}", system.cpu.pc);
            break;
        }
        
        // Log esporádico
        if i > 20 && i % 500 == 0 {
            println!("Instrução {}: ${:04X}: {:02X} - {} | PPU: L{} C{}", 
                     i, current_pc, opcode, system.get_cpu_state(),
                     system.get_scanline(), system.get_ppu().cycle);
        }
    }
    
    let success = instructions_executed > 10;
    
    println!("\n=== RESUMO ===");
    println!("Instruções executadas: {}", instructions_executed);
    println!("Frames gerados: {}", frame_count);
    println!("Estado final CPU: {}", system.get_cpu_state());
    println!("Estado final PPU:");
    println!("  - Scanline: {}, Cycle: {}", system.get_scanline(), system.get_ppu().cycle);
    println!("  - VBlank: {}", system.is_vblank());
    println!("  - Video Mode: {:?}", system.get_ppu().video_mode);
    println!("  - Brightness: {}", system.get_ppu().brightness);
    println!("  - NMI Enabled: {}", system.get_ppu().nmi_enabled);
    
    RomTestResult {
        rom_name: rom_name.to_string(),
        success,
        error_message: None,
        instructions_executed,
        final_state: system.get_cpu_state(),
        frames_generated: frame_count,
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
    let result = execute_rom_test("test_adc.smc", 1000, false);
    println!("\nResultado test_adc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_sbc_rom() {
    let result = execute_rom_test("test_sbc.smc", 1000, false);
    println!("\nResultado test_sbc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_tsc_rom() {
    let result = execute_rom_test("snes_test_tsc.smc", 1000, false);
    println!("\nResultado snes_test_tsc.smc:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
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
        let result = execute_rom_test(rom_file, 500, false); // Menos instruções para overview
        results.push(result);
    }
    
    println!("\n=== RESUMO DOS RESULTADOS ===");
    println!("{:<30} | {:<10} | {:<15} | {:<8} | {}", "ROM", "Status", "Instruções", "Frames", "Erro");
    println!("{}", "-".repeat(90));
    
    for result in &results {
        let status = if result.success { "✅ OK" } else { "❌ ERRO" };
        let error = result.error_message.as_deref().unwrap_or("Nenhum");
        println!("{:<30} | {:<10} | {:<15} | {:<8} | {}", 
                result.rom_name, status, result.instructions_executed, result.frames_generated, error);
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let total = results.len();
    
    println!("\n=== ESTATÍSTICAS ===");
    println!("ROMs testadas: {}", total);
    println!("Sucessos: {} ({:.1}%)", successful, (successful as f64 / total as f64) * 100.0);
    println!("Falhas: {}", total - successful);
    
    assert!(total > 0, "Nenhuma ROM foi testada");
}

#[test]
#[ignore]
fn test_detailed_adc() {
    let result = execute_rom_test("test_adc.smc", 10000, false);
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
    
    let valid_sizes = [0x40000, 0x80000, 0x100000, 0x200000, 0x300000, 0x400000];
    let size_valid = valid_sizes.contains(&rom_data.len());
    println!("📏 Tamanho válido: {}", if size_valid { "✅ SIM" } else { "⚠️ NÃO" });
    
    let system = System::new(rom_data);
    
    println!("\n=== INFORMAÇÕES DA ROM ===");
    println!("📄 Título: '{}'", system.memory.get_rom_title());
    println!("🗂️ Tipo: {:?}", system.memory.rom_type);
    println!("💾 SRAM Size: {} bytes", system.memory.sram_size);
    
    println!("\n=== VETORES DE INTERRUPT/RESET ===");
    
    let native_cop   = (system.memory.read(0x00FFE5) as u16) << 8 | system.memory.read(0x00FFE4) as u16;
    let native_brk   = (system.memory.read(0x00FFE7) as u16) << 8 | system.memory.read(0x00FFE6) as u16;
    let native_abort = (system.memory.read(0x00FFE9) as u16) << 8 | system.memory.read(0x00FFE8) as u16;
    let native_nmi   = (system.memory.read(0x00FFEB) as u16) << 8 | system.memory.read(0x00FFEA) as u16;
    let native_reset = (system.memory.read(0x00FFED) as u16) << 8 | system.memory.read(0x00FFEC) as u16;
    let native_irq   = (system.memory.read(0x00FFEF) as u16) << 8 | system.memory.read(0x00FFEE) as u16;
    
    let emu_cop   = (system.memory.read(0x00FFF5) as u16) << 8 | system.memory.read(0x00FFF4) as u16;
    let emu_abort = (system.memory.read(0x00FFF9) as u16) << 8 | system.memory.read(0x00FFF8) as u16;
    let emu_nmi   = (system.memory.read(0x00FFFB) as u16) << 8 | system.memory.read(0x00FFFA) as u16;
    let emu_reset = (system.memory.read(0x00FFFD) as u16) << 8 | system.memory.read(0x00FFFC) as u16;
    let emu_irq   = (system.memory.read(0x00FFFF) as u16) << 8 | system.memory.read(0x00FFFE) as u16;
    
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
    
    let current_reset = (system.memory.read(0x00FFFD) as u32) << 8 | system.memory.read(0x00FFFC) as u32;
    println!("\n🎯 Reset Vector Atual: ${:04X}", current_reset);
    
    let reset_valid = emu_reset != 0x0000 && emu_reset != 0xFFFF;
    println!("✅ Reset Vector Válido: {}", if reset_valid { "SIM" } else { "❌ NÃO" });
    
    if !reset_valid {
        println!("⚠️ PROBLEMA: Reset vector inválido! ROM pode estar corrompida.");
    }
    
    println!("\n=== DUMP DA MEMÓRIA NO RESET VECTOR ===");
    if emu_reset != 0x0000 && emu_reset < 0xFFFF {
        println!("Primeiros 16 bytes em ${:04X}:", emu_reset);
        for i in 0..16 {
            let addr = emu_reset.wrapping_add(i);
            let byte = system.memory.read(addr as u32);
            print!("${:04X}: {:02X} ", addr, byte);
            if (i + 1) % 8 == 0 { println!(); }
        }
        println!();
    }
}

#[test]
fn test_super_mario_world() {
    let result = execute_rom_test("Super.smc", 10000, false);
    println!("\nResultado Super Mario World:");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}

#[test]
fn test_super_mario_world_with_frames() {
    let result = execute_rom_test("Super.smc", 50000, true);
    println!("\n=== RESULTADO SUPER MARIO WORLD COM FRAMES ===");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
    println!("Estado final: {}", result.final_state);
    
    if let Some(error) = result.error_message {
        println!("Erro: {}", error);
    }
    
    println!("\n💡 DICA: Verifique os arquivos frame_Super_*.ppm gerados!");
    
    assert!(result.instructions_executed > 100, "Poucas instruções executadas");
}

#[test]
fn test_super_mario_world_quick() {
    let result = execute_rom_test("Super.smc", 10000, false);
    println!("\nResultado Super Mario World (rápido):");
    println!("Sucesso: {}", result.success);
    println!("Instruções executadas: {}", result.instructions_executed);
    println!("Frames gerados: {}", result.frames_generated);
    
    assert!(result.instructions_executed > 0, "Nenhuma instrução foi executada");
}
