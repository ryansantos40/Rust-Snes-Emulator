use snes_emulator::memory::{Memory, RomType};

fn create_test_rom() -> Vec<u8> {
        let mut rom = vec![0; 0x10000]; // 64KB ROM
        
        // Simular header LoROM válido em $7FC0
        let header_start = 0x7FC0;
        
        // Nome do jogo
        let title = b"TEST ROM             ";
        rom[header_start..header_start + 21].copy_from_slice(title);
        
        // Checksum válido (simplificado)
        rom[header_start + 0x1C] = 0x34; // Checksum low
        rom[header_start + 0x1D] = 0x12; // Checksum high
        rom[header_start + 0x1E] = 0xCB; // Complement low
        rom[header_start + 0x1F] = 0xED; // Complement high
        
        // SRAM size (32KB)
        rom[0x7FD8] = 0x03;
        
        rom
    }

    #[test]
    fn test_memory_creation() {
        let rom = create_test_rom();
        let memory = Memory::new(rom);
        
        assert_eq!(memory.sram_size, 0x8000); // 32KB
        assert!(matches!(memory.rom_type, RomType::LoRom));
        assert_eq!(memory.wram.len(), 0x20000); // 128KB
        assert_eq!(memory.vram.len(), 0x10000); // 64KB
    }

    #[test]
    fn test_wram_read_write() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste WRAM mirror (banco 0)
        memory.write(0x001000, 0xAB);
        assert_eq!(memory.read(0x001000), 0xAB);
        
        // Teste WRAM direto (banco 7E)
        memory.write(0x7E1000, 0xCD);
        assert_eq!(memory.read(0x7E1000), 0xCD);
        
        // Teste WRAM alto (banco 7F)
        memory.write(0x7F1000, 0xEF);
        assert_eq!(memory.read(0x7F1000), 0xEF);
    }

    #[test]
    fn test_rom_read() {
        let mut rom = create_test_rom();
        rom[0x8000] = 0x12; // Primeiro byte da ROM
        rom[0x8001] = 0x34;
        
        let memory = Memory::new(rom);
        
        // Teste leitura ROM em banco 0
        assert_eq!(memory.read(0x008000), 0x12);
        assert_eq!(memory.read(0x008001), 0x34);
        
        // Teste leitura ROM em banco 80 (mirror)
        assert_eq!(memory.read(0x808000), 0x12);
        assert_eq!(memory.read(0x808001), 0x34);
    }

    #[test]
    fn test_rom_write_readonly() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // ROM deve ser read-only
        memory.write(0x008000, 0xFF);
        assert_eq!(memory.read(0x008000), 0x00); // Deve permanecer 0
    }

    #[test]
    fn test_sram_read_write() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste SRAM
        memory.write(0x006000, 0x55);
        assert_eq!(memory.read(0x006000), 0x55);
        
        memory.write(0x807000, 0xAA);
        assert_eq!(memory.read(0x807000), 0xAA);
    }

    #[test]
    fn test_vram_access() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste acesso direto à VRAM
        memory.write_vram(0x1000, 0x42);
        assert_eq!(memory.read_vram(0x1000), 0x42);
        
        // Teste acesso via registradores PPU
        memory.write(0x002116, 0x00); // VRAM addr low
        memory.write(0x002117, 0x10); // VRAM addr high = 0x1000
        memory.write(0x002118, 0x33); // VRAM data write
        
        assert_eq!(memory.read_vram(0x1000), 0x33);
    }

    #[test]
    fn test_oam_access() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste acesso direto ao OAM
        memory.write_oam(0x100, 0x77);
        assert_eq!(memory.read_oam(0x100), 0x77);
        
        // Teste acesso via registradores PPU
        memory.write(0x002102, 0x00); // OAM addr low
        memory.write(0x002103, 0x01); // OAM addr high = 0x100
        memory.write(0x002104, 0x88); // OAM data write
        
        assert_eq!(memory.read_oam(0x100), 0x88);
    }

    #[test]
    fn test_cgram_access() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste acesso direto ao CGRAM
        memory.write_cgram(0x50, 0x99);
        assert_eq!(memory.read_cgram(0x50), 0x99);
        
        // Teste acesso via registradores PPU
        memory.write(0x002121, 0x50); // CGRAM addr
        memory.write(0x002122, 0xBB); // CGRAM data write
        
        assert_eq!(memory.read_cgram(0x50), 0xBB);
    }

    #[test]
    fn test_rom_title() {
        let rom = create_test_rom();
        let memory = Memory::new(rom);
        
        let title = memory.get_rom_title();
        assert_eq!(title, "TEST ROM");
    }

    #[test]
    fn test_sram_size_detection() {
        let mut rom = vec![0; 0x10000];
        
        // Teste diferentes tamanhos de SRAM
        rom[0x7FD8] = 0x01; // 2KB
        let memory = Memory::new(rom.clone());
        assert_eq!(memory.sram_size, 0x800);
        
        rom[0x7FD8] = 0x02; // 8KB
        let memory = Memory::new(rom.clone());
        assert_eq!(memory.sram_size, 0x2000);
        
        rom[0x7FD8] = 0x03; // 32KB
        let memory = Memory::new(rom.clone());
        assert_eq!(memory.sram_size, 0x8000);
    }

    #[test]
    fn test_bounds_checking() {
        let rom = create_test_rom();
        let mut memory = Memory::new(rom);
        
        // Teste leitura além dos limites - deve retornar 0
        assert_eq!(memory.read(0xFF0000), 0); // Banco não mapeado
        assert_eq!(memory.read_vram(0xFFFF), 0); // VRAM além do limite
        assert_eq!(memory.read_oam(0x300), 0); // OAM além do limite
        assert_eq!(memory.read_cgram(0x300), 0); // CGRAM além do limite
        
        // Teste escrita além dos limites - não deve crashar
        memory.write_vram(0xFFFF, 0xFF);
        memory.write_oam(0x300, 0xFF);
        memory.write_cgram(0x300, 0xFF);
    }