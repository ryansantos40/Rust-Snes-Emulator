use snes_emulator::System;
use snes_emulator::opcodes;
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
    
    println!("📂 Carregando ROM: {}", rom_path);
    
    let rom_data = match fs::read(&rom_path) {
        Ok(mut data) => {
            if data.len() % 1024 == 512 {
                println!("🔧 Removendo header SMC...");
                data.drain(0..512);
            }
            data
        },
        Err(e) => {
            eprintln!("❌ Erro ao carregar ROM: {}", e);
            return;
        }
    };
    
    let mut system = System::new(rom_data);
    
    // Configura reset vector
    let reset_low = system.memory.read(0x00FFFC) as u32;
    let reset_high = system.memory.read(0x00FFFD) as u32;
    system.cpu.pc = (reset_high << 8) | reset_low;
    
    println!("\n╔═══════════════════════════════════════╗");
    println!("║       INFORMAÇÕES DA ROM              ║");
    println!("╠═══════════════════════════════════════╣");
    println!("║ Título: {:<29} ║", system.memory.get_rom_title());
    println!("║ Tipo: {:?}                       ║", system.memory.rom_type);
    println!("║ SRAM: {} bytes                      ║", system.memory.sram_size);
    println!("║ Reset Vector: ${:04X}                 ║", system.cpu.pc);
    println!("╚═══════════════════════════════════════╝\n");
    
    let max_instructions = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100000);
    
    let mut frames = 0;
    let mut instructions_this_frame = 0;
    let mut last_scanline = 0;
    let verbose = args.contains(&"--verbose".to_string());
    
    println!("🎮 Iniciando emulação... (max {} instruções)\n", max_instructions);
    
    for i in 0..max_instructions {
        let current_pc = system.cpu.pc;
        let opcode = system.memory.read(current_pc);
        
        // ✅ VERIFICA OPCODE NÃO IMPLEMENTADO
        if opcodes::get_opcode_info(opcode).is_none() {
            println!("\n❌ ═══════════════════════════════════════════════");
            println!("❌ OPCODE NÃO IMPLEMENTADO: ${:02X}", opcode);
            println!("❌ ═══════════════════════════════════════════════");
            println!("📍 PC: ${:06X} | Instrução #{}", current_pc, i + 1);
            println!("📊 CPU: {}", system.get_cpu_state());
            println!("🎨 PPU: Scanline {} Cycle {} VBlank:{}", 
                     system.get_scanline(), 
                     system.get_ppu().cycle,
                     system.is_vblank());
            
            // Contexto de memória
            print!("📄 Memória: ");
            for offset in -4i32..=4 {
                let addr = (current_pc as i32 + offset) as u32;
                let byte = system.memory.read(addr);
                if offset == 0 {
                    print!("→[{:02X}]← ", byte);
                } else {
                    print!("{:02X} ", byte);
                }
            }
            println!("\n💡 Adicione o opcode ${:02X} em opcodes.rs", opcode);
            println!("❌ ═══════════════════════════════════════════════\n");
            break;
        }
        
        // 🔇 SILENCIA BRK (apenas conta)
        let is_brk = opcode == 0x00;
        
        // Log verbose apenas se solicitado
        if verbose && i < 50 {
            let old_state = system.get_cpu_state();
            let cycles = system.step();
            println!("{:4}: ${:04X} {:02X} {} → {} ({}c)", 
                     i + 1, current_pc, opcode, old_state, system.get_cpu_state(), cycles);
        } else {
            system.step();
        }
        
        instructions_this_frame += 1;
        
        // 📺 DETECTA MUDANÇA DE SCANLINE (sem spam)
        let current_scanline = system.get_scanline();
        if current_scanline != last_scanline && current_scanline % 60 == 0 {
            if verbose {
                println!("  📺 Scanline {}", current_scanline);
            }
            last_scanline = current_scanline;
        }
        
        // 🎬 FRAME COMPLETO
        if system.frame_ready() {
            frames += 1;
            println!("🎬 Frame #{:2} │ {} instruções │ {} ciclos │ PC: ${:06X}", 
                     frames, 
                     instructions_this_frame,
                     system.cpu.cycles,
                     system.cpu.pc);
            
            instructions_this_frame = 0;
            
            if frames >= 30000 {
                println!("\n✅ Limite de {} frames atingido", frames);
                break;
            }
        }
        
        // 🔁 DETECTA LOOP INFINITO
        if !is_brk && system.cpu.pc == current_pc {
            println!("\n🔁 Loop infinito em ${:04X} (instrução #{})", current_pc, i + 1);
            println!("   (Programa entrou em wait loop)");
            break;
        }
    }
    
    // ═══════════════════════════════════════════════════════
    // ESTATÍSTICAS FINAIS
    // ═══════════════════════════════════════════════════════
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║          ESTATÍSTICAS FINAIS                      ║");
    println!("╠═══════════════════════════════════════════════════╣");
    println!("║ 🎮 Frames renderizados: {:25} ║", frames);
    println!("║ ⏱️  Ciclos CPU: {:34} ║", system.cpu.cycles);
    println!("║ 📊 Estado CPU: {:33} ║", system.get_cpu_state());
    println!("║ 📍 PC final: ${:04X}                              ║", system.cpu.pc);
    println!("╠═══════════════════════════════════════════════════╣");
    println!("║ PPU:                                              ║");
    println!("║   📺 Scanline: {:35} ║", system.get_scanline());
    println!("║   🔄 Cycle: {:38} ║", system.get_ppu().cycle);
    println!("║   🌑 VBlank: {:37} ║", system.is_vblank());
    println!("║   🎨 Video Mode: {:?}                          ║", system.get_ppu().video_mode);
    println!("║   💡 Brightness: {:32} ║", system.get_ppu().brightness);
    println!("║   ⚡ NMI Enabled: {:31} ║", system.get_ppu().nmi_enabled);
    println!("╠═══════════════════════════════════════════════════╣");
    
    // Análise de timing
    let total_ppu_cycles = system.cpu.cycles * 4;
    let expected_frames = total_ppu_cycles / (341 * 262);
    println!("║ 📈 Timing:                                        ║");
    println!("║   • PPU cycles: ~{:31} ║", total_ppu_cycles);
    println!("║   • Frames esperados: ~{:25} ║", expected_frames);
    
    // Status
    if system.get_ppu().brightness == 0 {
        println!("║   ⚠️  Tela desligada (brightness = 0)            ║");
    }
    if !system.get_ppu().nmi_enabled {
        println!("║   ⚠️  NMI desabilitado (não está no loop)        ║");
    }
    
    println!("╚═══════════════════════════════════════════════════╝\n");
}