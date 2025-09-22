use std::collections::HashMap;

pub struct Memory {
    pub wram: [u8; 0x20000], // 128KB WRAM
    pub rom: Vec<u8>,     // ROM data
    pub vram: [u8; 0x10000], // 64KB VRAM
    pub oam: [u8; 0x220], // 512B OAM + 32B Padding
    pub cgram: [u8; 0x200], // 512B CGRAM
    pub sram: Vec<u8>, // Save Ram
    pub registers: HashMap<u16, u8>, // Registradores(To-DO)
    pub rom_type: RomType, // Tipo de mapeamento (LoRom, HiRom)
    pub sram_size: usize, // Tamanho do SRAM
}

#[derive(Debug, Clone, Copy)]
pub enum RomType {
    LoRom,
    HiRom,
}

impl Memory{
    pub fn new(rom: Vec<u8>) -> Self {
        let rom_type = Self::detect_rom_type(&rom);
        let sram_size = Self::detect_sram_size(&rom);

        Memory {
            wram: [0; 0x20000],
            rom,
            vram: [0; 0x10000],
            oam: [0; 0x220],
            cgram: [0; 0x200],
            sram: vec![0; sram_size],
            registers: HashMap::new(),
            rom_type,
            sram_size,
        }
    }

    fn detect_rom_type(rom: &[u8]) -> RomType {
        if rom.len() < 0x8000 {
            return RomType::LoRom; // ROM muito pequena para ser HiRom
        }

        let lorom_header = 0x7FC0;
        let hirom_header = 0xFFC0;

        if rom.len() > hirom_header + 0x20 {
            let hirom_checksum = (rom[hirom_header + 0x1C] as u16) | ((rom[hirom_header + 0x1D] as u16) << 8);
            let hirom_complement = (rom[hirom_header + 0x1E] as u16) | ((rom[hirom_header + 0x1F] as u16) << 8);

            if hirom_checksum.wrapping_add(hirom_complement) == 0xFFFF {
                return RomType::HiRom;
            }
        }

        if rom.len() > lorom_header + 0x20 {
            let lorom_checksum = (rom[lorom_header + 0x1C] as u16) | ((rom[lorom_header + 0x1D] as u16) << 8);
            let lorom_complement = (rom[lorom_header + 0x1E] as u16) | ((rom[lorom_header + 0x1F] as u16) << 8);

            if lorom_checksum.wrapping_add(lorom_complement) == 0xFFFF {
                return RomType::LoRom;
            }
        }

        RomType::LoRom // Padrão para LoRom
    }

    fn detect_sram_size(rom: &[u8]) -> usize {
        if rom.len() < 0x7FD8 {
            return 0;
        }

        let sram_byte = rom[0x7FD8];
        match sram_byte {
            0x00 => 0, // sem SRAM
            0x01 => 0x800, // 2KB
            0x02 => 0x2000, // 8KB
            0x03 => 0x8000, //32KB
            0x04 => 0x20000, // 128KB
            _ => 0x8000, // Padrão 32KB
        }
    }

    pub fn read(&self, addr: u32) -> u8 {
        let bank = (addr >> 16) as u8;
        let offset = (addr & 0xFFFF) as u16;

        match bank {
            // Bancos 00-3F: Sistema + LoRom
            0x00..=0x3F => {
                match offset {
                    0x0000..=0x1FFF => self.wram[offset as usize],
                    0x2000..=0x20FF => self.wram[offset as usize],
                    0x2100..=0x21FF => self.read_ppu_registers(offset),
                    0x2200..=0x3FFF => self.wram[offset as usize],
                    0x4000..=0x4015 => self.read_apu_registers(offset),
                    0x4016..=0x4017 => self.registers.get(&offset).copied().unwrap_or(0), // Input
                    0x4018..=0x401F => self.read_apu_registers(offset),
                    0x4020..=0x41FF => self.read_apu_registers(offset),
                    0x4200..=0x44FF => self.read_dma_registers(offset),
                    0x4500..=0x5FFF => self.wram[offset as usize],
                    0x6000..=0x7FFF => { // SRAM Area para LoRom
                        if self.sram_size > 0 {
                            let sram_addr = (offset - 0x6000) as usize;
                            if sram_addr < self.sram.len(){
                                self.sram[sram_addr]
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                    //LoRom Area
                    0x8000..=0xFFFF => {
                        let rom_addr = ((bank as usize) * 0x8000) + ((offset - 0x8000) as usize);
                        if rom_addr < self.rom.len() {
                            self.rom[rom_addr]
                        } else {
                            0
                        }
                    }
                }
            }

            0x40..=0x6F => {
                if offset >= 0x8000 {
                    let rom_addr = ((bank as usize) * 0x8000) + ((offset - 0x8000) as usize);
                    if rom_addr < self.rom.len(){
                        self.rom[rom_addr]
                    } else {
                        0
                    }
                } else {
                    0 // Areas não mapeadas
                }
            }

            0x7E => self.wram[offset as usize], // WRAM (primeiros 64KB)

            0x7F => self.wram[0x10000_usize + offset as usize], // WRAM (últimos 64KB)

            0x80..=0xBF => {
                match offset {
                    0x0000..=0x1FFF => self.wram[offset as usize],
                    0x2000..=0x20FF => self.wram[offset as usize],
                    0x2100..=0x21FF => self.read_ppu_registers(offset),
                    0x2200..=0x3FFF => self.wram[offset as usize],
                    0x4000..=0x4015 => self.read_apu_registers(offset),
                    0x4016..=0x4017 => self.registers.get(&offset).copied().unwrap_or(0), // Input
                    0x4018..=0x401F => self.read_apu_registers(offset),
                    0x4020..=0x41FF => self.read_apu_registers(offset),
                    0x4200..=0x44FF => self.read_dma_registers(offset),
                    0x4500..=0x5FFF => self.wram[offset as usize],
                    0x6000..=0x7FFF => { // SRAM Area para LoRom
                        if self.sram_size > 0 {
                            let sram_addr = (offset - 0x6000) as usize;
                            if sram_addr < self.sram.len(){
                                self.sram[sram_addr]
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                    0x8000..=0xFFFF => {
                        let mapped_bank = bank - 0x80;
                        let rom_addr = ((mapped_bank as usize) * 0x8000) + ((offset - 0x8000) as usize);
                        if rom_addr < self.rom.len() {
                            self.rom[rom_addr]
                        } else {
                            0
                        }
                    }
                }
            }

            //HiRom area ou continuação do LoRom
            0xC0..=0xFF => {
                match self.rom_type {
                    RomType::HiRom => {
                        let rom_addr = (((bank - 0xC0) as usize) << 16) | (offset as usize);
                        if rom_addr < self.rom.len() {
                            self.rom[rom_addr]
                        } else {
                            0
                        }
                    }
                    RomType::LoRom => {
                        //unmapped area
                        0
                    }
                }
            }

            _ => 0, // Unmapped area
        }
    }

    pub fn write(&mut self, addr: u32, value: u8) {
        let bank = (addr >> 16) as u8;
        let offset = (addr & 0xFFFF) as u16;

        match bank {
            0x00..=0x3F => {
                match offset {
                    //WRAM mirror
                    0x0000..=0x1FFF => self.wram[offset as usize] = value,
                    0x2000..=0x20FF => self.wram[offset as usize] = value,
                    0x2100..=0x21FF => self.write_ppu_registers(offset, value),
                    0x2200..=0x3FFF => self.wram[offset as usize] = value,
                    0x4000..=0x4015 => self.write_apu_registers(offset, value),
                    0x4016..=0x4017 => { self.registers.insert(offset, value); }, // Input
                    0x4018..=0x401F => self.write_apu_registers(offset, value),
                    0x4020..=0x41FF => self.write_apu_registers(offset, value),
                    0x4200..=0x44FF => self.write_dma_registers(offset, value),
                    0x4500..=0x5FFF => self.wram[offset as usize] = value,
                    0x6000..=0x7FFF => { // SRAM write
                        if self.sram_size > 0 {
                            let sram_addr = (offset - 0x6000) as usize;
                            if sram_addr < self.sram.len() {
                                self.sram[sram_addr] = value;
                            }
                        }
                    } // Input
                    0x8000..=0xFFFF => {} // Rom area (read-only)
                }
            }

            0x7E => self.wram[offset as usize] = value, // WRAM (first 64KB)
            0x7F => self.wram[0x10000_usize + offset as usize] = value, // WRAM (last 64KB)

            0x80..=0xBF => {
                match offset {
                    0x0000..=0x1FFF => self.wram[offset as usize] = value,
                    0x2000..=0x20FF => self.wram[offset as usize] = value,
                    0x2100..=0x21FF => self.write_ppu_registers(offset, value),
                    0x2200..=0x3FFF => self.wram[offset as usize] = value,
                    0x4000..=0x4015 => self.write_apu_registers(offset, value),
                    0x4016..=0x4017 => { self.registers.insert(offset, value); }, // Input
                    0x4018..=0x401F => self.write_apu_registers(offset, value),
                    0x4020..=0x41FF => self.write_apu_registers(offset, value),
                    0x4200..=0x44FF => self.write_dma_registers(offset, value),
                    0x4500..=0x5FFF => self.wram[offset as usize] = value,
                    0x6000..=0x7FFF => { // SRAM write
                        if self.sram_size > 0 {
                            let sram_addr = (offset - 0x6000) as usize;
                            if sram_addr < self.sram.len() {
                                self.sram[sram_addr] = value;
                            }
                        }
                    }
                    0x8000..=0xFFFF => {} // Rom area (read-only)
                }
            }

            _ => {} // Unmapped area
        }
    }

    fn read_ppu_registers(&self, addr: u16) -> u8 {
        match addr {
            0x2134..=0x2136 => self.registers.get(&addr).copied().unwrap_or(0), // VRAM read
            0x2137 => 0, // SLHV
            0x2138 => 0, // OAM READ
            0x2139 => 0, // VRAM low read
            0x213A => 0, // VRAM high read
            0x213B => 0, // CGRAM read
            0x213C => 0, // H/V counter
            0x213D => 0, // ppu status
            0x213E => 0, // ppu status
            0x213F => 0, // ppu status
            _ => self.registers.get(&addr).copied().unwrap_or(0), // Outros registradores PPU
        }
    }

    fn write_ppu_registers(&mut self, addr: u16, value: u8) {
        match addr {
            // VRAM access
            0x2116 => { // VRAM address low
                self.registers.insert(0x2116, value);
            }
            0x2117 => { // VRAM address high
                self.registers.insert(0x2117, value);
            }
            0x2118 => { // VRAM data write low
                let addr_low = self.registers.get(&0x2116).copied().unwrap_or(0);
                let addr_high = self.registers.get(&0x2117).copied().unwrap_or(0);
                let vram_addr = ((addr_high as u16) << 8) | (addr_low as u16);
                if (vram_addr as usize) < self.vram.len() {
                    self.vram[vram_addr as usize] = value;
                }
            }
            0x2119 => { // VRAM data write high
                let addr_low = self.registers.get(&0x2116).copied().unwrap_or(0);
                let addr_high = self.registers.get(&0x2117).copied().unwrap_or(0);
                let vram_addr = ((addr_high as u16) << 8) | (addr_low as u16);
                if (vram_addr as usize + 1) < self.vram.len() {
                    self.vram[vram_addr as usize + 1] = value;
                }
            }
            
            // OAM access
            0x2102 => { self.registers.insert(addr, value); }, // OAM address low
            0x2103 => { self.registers.insert(addr, value); }, // OAM address high
            0x2104 => { // OAM data write
                let addr_low = self.registers.get(&0x2102).copied().unwrap_or(0);
                let addr_high = self.registers.get(&0x2103).copied().unwrap_or(0);
                let oam_addr = ((addr_high as u16) << 8) | (addr_low as u16);
                if (oam_addr as usize) < self.oam.len() {
                    self.oam[oam_addr as usize] = value;
                }
            }
            
            // CGRAM access
            0x2121 => { self.registers.insert(addr, value); }, // CGRAM address
            0x2122 => { // CGRAM data write
                let cgram_addr = self.registers.get(&0x2121).copied().unwrap_or(0);
                if (cgram_addr as usize) < self.cgram.len() {
                    self.cgram[cgram_addr as usize] = value;
                }
            }
            
            _ => { self.registers.insert(addr, value); }
        }
    }

    pub fn save_sram(&self, path: &str) -> std::io::Result<()> {
        if self.sram_size > 0 {
            std::fs::write(path, &self.sram[..self.sram_size])?;
        }
        Ok(())
    }

    pub fn load_sram(&mut self, path: &str) -> std::io::Result<()> {
        if self.sram_size > 0 {
            let sram_data = std::fs::read(path)?;
            let copy_size = std::cmp::min(sram_data.len(), self.sram_size);
            self.sram[..copy_size].copy_from_slice(&sram_data[..copy_size]);
        }
        Ok(())
    }

    pub fn get_rom_title(&self) -> String {
        if self.rom.len() < 0x7FC0 + 21 {
            return "Unknown".to_string();
        }

        let title_bytes = &self.rom[0x7FC0..0x7FC0 + 21];
        String::from_utf8_lossy(title_bytes).trim().to_string()
    }

    // APU Registers ($4000-$41FF)
    fn read_apu_registers(&self, addr: u16) -> u8 {
        self.registers.get(&addr).copied().unwrap_or(0)
    }

    fn write_apu_registers(&mut self, addr: u16, value: u8) {
        self.registers.insert(addr, value);
    }

    // DMA/HDMA Registers ($4200-$44FF)
    fn read_dma_registers(&self, addr: u16) -> u8 {
        self.registers.get(&addr).copied().unwrap_or(0)
    }

    fn write_dma_registers(&mut self, addr: u16, value: u8) {
        self.registers.insert(addr, value);
    }

    // Métodos auxiliares para VRAM, OAM, CGRAM
    pub fn read_vram(&self, addr: u16) -> u8 {
        if (addr as usize) < self.vram.len() {
            self.vram[addr as usize]
        } else {
            0
        }
    }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        if (addr as usize) < self.vram.len() {
            self.vram[addr as usize] = value;
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        if (addr as usize) < self.oam.len() {
            self.oam[addr as usize]
        } else {
            0
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        if (addr as usize) < self.oam.len() {
            self.oam[addr as usize] = value;
        }
    }

    pub fn read_cgram(&self, addr: u16) -> u8 {
        if (addr as usize) < self.cgram.len() {
            self.cgram[addr as usize]
        } else {
            0
        }
    }

    pub fn write_cgram(&mut self, addr: u16, value: u8) {
        if (addr as usize) < self.cgram.len() {
            self.cgram[addr as usize] = value;
        }
    }
}