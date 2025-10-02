use snes_emulator::System;
use snes_emulator::opcodes;
use std::{env, fs};

fn detect_boot_phase(system: &System) -> &'static str {
    let pc = system.cpu.pc;
    let brightness = system.get_ppu().brightness;
    let nmi_enabled = system.get_ppu().nmi_enabled;
    
    match (pc, brightness, nmi_enabled) {
        (0x8000..=0x804A, 0, false) => "🔧 Inicialização da WRAM",
        (0x804B..=0x8200, 0, false) => "⚙️  Inicialização do hardware",
        (_, 0, false) => "📺 Configurando vídeo",
        (_, 1..=15, false) => "🎨 Carregando gráficos",
        (_, _, true) => "🎮 Loop principal (NMI ativo)",
        _ => "❓ Fase desconhecida"
    }
}

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
    
    let mut system = System::new(rom_data);
    
    // Configura reset vector
    let reset_low = system.memory.read(0x00FFFC) as u32;
    let reset_high = system.memory.read(0x00FFFD) as u32;
    system.cpu.pc = (reset_high << 8) | reset_low;
    
    println!("=== INFORMAÇÕES DA ROM ===");
    println!("Título: {}", system.memory.get_rom_title());
    println!("Tipo: {:?}", system.memory.rom_type);
    println!("SRAM: {} bytes", system.memory.sram_size);
    println!("Reset Vector: ${:04X}", system.cpu.pc);
    println!("Estado inicial CPU: {}", system.get_cpu_state());
    println!("Estado inicial PPU: Scanline {}, Cycle {}, VBlank: {}", 
             system.get_scanline(), 
             system.get_ppu().cycle,
             system.is_vblank());
    
    // Mostra vetores de interrupção
    let brk_vector = (system.memory.read(0x00FFE7) as u16) << 8 | system.memory.read(0x00FFE6) as u16;
    let nmi_vector = (system.memory.read(0x00FFEB) as u16) << 8 | system.memory.read(0x00FFEA) as u16;
    println!("\n=== VETORES DE INTERRUPÇÃO ===");
    println!("BRK Vector: ${:04X}", brk_vector);
    println!("NMI Vector: ${:04X}", nmi_vector);
    
    println!("\n=== EXECUÇÃO ===");
    
    let max_instructions = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    
    let mut frames = 0;
    let mut instructions = 0;
    let mut last_scanline = 0;
    let mut last_phase = "";
    
    for i in 0..max_instructions {
        let current_pc = system.cpu.pc;
        let opcode = system.memory.read(current_pc);
        
        // ✅ VERIFICA SE O OPCODE É VÁLIDO ANTES DE EXECUTAR
        if opcodes::get_opcode_info(opcode).is_none() {
            println!("\n❌ ============================================");
            println!("❌ OPCODE NÃO IMPLEMENTADO DETECTADO!");
            println!("❌ ============================================");
            println!("📍 Endereço: ${:06X}", current_pc);
            println!("🔢 Opcode: ${:02X}", opcode);
            println!("📊 Estado CPU: {}", system.get_cpu_state());
            println!("🖼️  Estado PPU: Scanline {}, Cycle {}", system.get_scanline(), system.get_ppu().cycle);
            println!("📈 Instruções executadas: {}", i);
            println!("⏱️  Ciclos totais: {}", system.cpu.cycles);
            
            // Mostra contexto (bytes ao redor)
            println!("\n📄 Contexto da memória:");
            print!("   ${:06X}: ", current_pc.saturating_sub(4));
            for offset in -4i32..=4 {
                let addr = (current_pc as i32 + offset) as u32;
                let byte = system.memory.read(addr);
                if offset == 0 {
                    print!("[{:02X}] ", byte); // Destaca o opcode problemático
                } else {
                    print!("{:02X} ", byte);
                }
            }
            println!();
            
            println!("\n💡 DICA: Implemente o opcode ${:02X} no arquivo opcodes.rs", opcode);
            println!("❌ ============================================\n");
            break;
        }
        
        let old_state = system.get_cpu_state();
        
        // Detecta BRK ANTES de executar
        if opcode == 0x00 {
            println!("\n🚨 ============================================");
            println!("🚨 BRK (SOFTWARE INTERRUPT) DETECTADO!");
            println!("🚨 ============================================");
            println!("📍 Endereço do BRK: ${:06X}", current_pc);
            println!("📊 Estado CPU antes: {}", system.get_cpu_state());
            println!("🎯 BRK Vector: ${:04X}", brk_vector);
            
            // Contexto
            print!("📄 Contexto: ");
            for offset in -2i32..=2 {
                let addr = (current_pc as i32 + offset) as u32;
                let byte = system.memory.read(addr);
                if offset == 0 {
                    print!("[{:02X}] ", byte);
                } else {
                    print!("{:02X} ", byte);
                }
            }
            println!();
        }
        
        // Executa uma instrução (CPU + PPU)
        let cycles = system.step();
        instructions += 1;
        
        // Se foi um BRK, mostra o estado depois
        if opcode == 0x00 {
            println!("📊 Estado CPU depois: {}", system.get_cpu_state());
            println!("📍 Novo PC: ${:06X}", system.cpu.pc);
            println!("🚨 ============================================\n");
        }
        
        // Detecta mudança de fase
        let current_phase = detect_boot_phase(&system);
        if current_phase != last_phase {
            println!("\n🔄 ════════ MUDANÇA DE FASE ════════");
            println!("   {} → {}", last_phase, current_phase);
            println!("   PC: ${:06X} | Instrução: {}", system.cpu.pc, i + 1);
            println!("════════════════════════════════════\n");
            last_phase = current_phase;
        }
        
        // Log detalhado das primeiras instruções
        if i < 50 {
            println!("{:4}: ${:04X}: {:02X} {} -> {} ({}c) | PPU: L{:3} C{:3}", 
                     i + 1, 
                     current_pc, 
                     opcode, 
                     old_state, 
                     system.get_cpu_state(), 
                     cycles,
                     system.get_scanline(),
                     system.get_ppu().cycle);
        } else if i % 100 == 0 {
            // Log esporádico com fase
            println!("{:4}: ${:04X}: {:02X} - {} | {} | PPU: L{:3} C{:3}", 
                     i + 1, 
                     current_pc, 
                     opcode, 
                     system.get_cpu_state(),
                     current_phase,
                     system.get_scanline(),
                     system.get_ppu().cycle);
        }
        
        // Detecta mudança de scanline (para ver se PPU está funcionando)
        let current_scanline = system.get_scanline();
        if current_scanline != last_scanline {
            if i < 50 || current_scanline % 50 == 0 {
                println!("  └─ PPU: Scanline {} | VBlank: {}", 
                         current_scanline,
                         system.is_vblank());
            }
            last_scanline = current_scanline;
        }
        
        // Detecta frames completos
        if system.frame_ready() {
            frames += 1;
            println!("\n  ╔════════════════════════════════════════╗");
            println!("  ║  FRAME #{} COMPLETO                     ║", frames);
            println!("  ║  Instruções: {}                       ║", instructions);
            println!("  ║  CPU Cycles: {}                    ║", system.cpu.cycles);
            println!("  ╚════════════════════════════════════════╝\n");
            
            instructions = 0;
            
            // Para após alguns frames para não ficar infinito
            if frames >= 3 {
                println!("Limite de frames atingido. Encerrando...");
                break;
            }
        }
        
        // Detecta entrada em VBlank
        if system.is_vblank() && !system.get_ppu().frame_complete {
            if i < 100 || i % 200 == 0 {
                println!("  └─ VBlank iniciado na instrução {}", i + 1);
            }
        }
        
        // Detecta loop infinito
        if current_pc == system.cpu.pc && opcode != 0x00 {  // Ignora BRK
            println!("\n🔁 Loop infinito detectado em ${:04X}", current_pc);
            println!("   Isso é normal se o programa entrou em loop de espera.");
            break;
        }
    }
    
    println!("\n=== ESTATÍSTICAS FINAIS ===");
    println!("Fase final: {}", detect_boot_phase(&system));
    println!("Estado final CPU: {}", system.get_cpu_state());
    println!("Ciclos totais CPU: {}", system.cpu.cycles);
    println!("Frames completos: {}", frames);
    println!("Estado final PPU:");
    println!("  - Scanline: {}", system.get_scanline());
    println!("  - Cycle: {}", system.get_ppu().cycle);
    println!("  - VBlank: {}", system.is_vblank());
    println!("  - Video Mode: {:?}", system.get_ppu().video_mode);
    println!("  - Brightness: {}", system.get_ppu().brightness);
    println!("  - NMI Enabled: {}", system.get_ppu().nmi_enabled);
    
    // Análise de progresso
    if system.cpu.pc >= 0x8000 && system.cpu.pc <= 0x8048 {
        let y_reg = system.cpu.y;
        if y_reg <= 1021 {
            let progress = ((1021 - y_reg as i32) as f32 / 1021.0) * 100.0;
            println!("\n📊 Progresso do clear WRAM: {:.1}%", progress);
            println!("   Bytes limpos: ~{}", 1021 - y_reg);
            println!("   Bytes restantes: ~{}", y_reg);
        }
    }
    
    if system.get_ppu().brightness == 0 {
        println!("\n⚠️  Brightness = 0 (tela escura)");
        println!("   A ROM ainda não ligou a tela.");
    }
    
    if !system.get_ppu().nmi_enabled {
        println!("\n⚠️  NMI desabilitado");
        println!("   O jogo ainda não está no loop principal.");
    }
    
    // Estatísticas de timing
    let total_ppu_cycles = system.cpu.cycles * 4;
    let expected_scanlines = total_ppu_cycles / 341;
    println!("\nTiming:");
    println!("  - Total PPU cycles: ~{}", total_ppu_cycles);
    println!("  - Scanlines esperadas: ~{}", expected_scanlines);
    println!("  - Frames esperados: ~{}", expected_scanlines / 262);
    
    println!("\n=== ANÁLISE FINAL ===");
    if system.cpu.pc == brk_vector.into() {
        println!("✅ BRK tratado corretamente (saltou para BRK handler)");
    } else if system.cpu.pc > 0x8048 {
        println!("✅ Programa passou da inicialização da WRAM");
    } else {
        println!("⚠️  Programa ainda na inicialização");
    }
}