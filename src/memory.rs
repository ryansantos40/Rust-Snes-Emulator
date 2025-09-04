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
}

#[derive(Debug, Clone, Copy)]
pub enum RomType {
    LoRom,
    HiRom,
}

impl Memory{
    pub fn new(rom: Vec<u8>) -> Self {
        let rom_type = if rom.len() > 0x8000 {
            RomType::LoRom
        } else {
            RomType::LoRom // Placeholder
        };

        Memory {
            wram: [0; 0x20000],
            rom,
            vram: [0; 0x10000],
            oam: [0; 0x220],
            cgram: [0; 0x200],
            sram: vec![0; 0x8000], // 32KB SRAM
            registers: HashMap::new(),
            rom_type,
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
                    0x2100..=0x21FF => self.read_ppu_registers(offset),
                    0x4000..=0x41FF => self.read_apu_registers(offset),
                    0x4200..=0x44FF => self.read_dma_registers(offset),
                    0x4016..=0x4017 => self.registers.get(&offset).copied().unwrap_or(0), // Input
                    //LoRom Area
                    0x8000..=0xFFFF => {
                        let rom_addr = ((bank as usize) << 15) | ((offset - 0x8000) as usize);
                        self.rom.get(rom_addr).copied().unwrap_or(0)
                    }
                    _ => 0, // Areas não mapeadas
                }
            }

            0x40..=0x6F => {
                if offset >= 0x8000 {
                    let rom_addr = ((bank as usize) << 15) | ((offset - 0x8000) as usize);
                    self.rom.get(rom_addr).copied().unwrap_or(0)
                } else {
                    0 // Areas não mapeadas
                }
            }

            0x7E => self.wram[offset as usize], // WRAM (primeiros 64KB)

            0x7F => self.wram[(0x10000 + offset) as usize], // WRAM (últimos 64KB)

            0x80..=0xBF => {
                match offset {
                    0x0000..=0x1FFF => self.wram[offset as usize],
                    0x2100..=0x21FF => self.read_ppu_registers(offset),
                    0x4000..=0x41FF => self.read_apu_registers(offset),
                    0x4200..=0x44FF => self.read_dma_registers(offset),
                    0x4016..=0x4017 => self.registers.get(&offset).copied().unwrap_or(0), // Input
                    0x8000..=0xFFFF => {
                        let rom_addr = (((bank - 0x80) as usize) << 15) | ((offset - 0x8000) as usize);
                        self.rom.get(rom_addr).copied().unwrap_or(0)
                    }
                    _ => 0, // Areas não mapeadas
                }
            }

            //HiRom area ou continuação do LoRom
            0xC0..=0xFF => {
                match self.rom_type {
                    RomType::HiRom => {
                        let rom_addr = (((bank - 0xC0) as usize) << 16) | (offset as usize);
                        self.rom.get(rom_addr).copied().unwrap_or(0)
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
                    // Hardware Registers
                    0x2100..=0x21FF => self.write_ppu_registers(offset, value),
                    0x4000..=0x41FF => self.write_apu_registers(offset, value),
                    0x4200..=0x44FF => self.write_dma_registers(offset, value),
                    0x4016..=0x4017 => { self.registers.insert(offset, value); }, // Input
                    0x8000..=0xFFFF => {} // Rom area (read-only)
                    _ => {} // Unmapped area
                }
            }

            0x7E => self.wram[offset as usize] = value, // WRAM (first 64KB)
            0x7F => self.wram[(0x10000 + offset) as usize] = value, // WRAM (last 64KB)

            0x80..=0xBF => {
                match offset {
                    0x0000..=0x1FFF => self.wram[offset as usize] = value,
                    0x2100..=0x21FF => self.write_ppu_registers(offset, value),
                    0x4000..=0x41FF => self.write_apu_registers(offset, value),
                    0x4200..=0x44FF => self.write_dma_registers(offset, value),
                    0x4016..=0x4017 => { self.registers.insert(offset, value); }, // Input
                    0x8000..=0xFFFF => {} // Rom area (read-only)
                    _ => {} // Unmapped area
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
            0x2102 => self.registers.insert(addr, value), // OAM address low
            0x2103 => self.registers.insert(addr, value), // OAM address high
            0x2104 => { // OAM data write
                let addr_low = self.registers.get(&0x2102).copied().unwrap_or(0);
                let addr_high = self.registers.get(&0x2103).copied().unwrap_or(0);
                let oam_addr = ((addr_high as u16) << 8) | (addr_low as u16);
                if (oam_addr as usize) < self.oam.len() {
                    self.oam[oam_addr as usize] = value;
                }
            }
            
            // CGRAM access
            0x2121 => self.registers.insert(addr, value), // CGRAM address
            0x2122 => { // CGRAM data write
                let cgram_addr = self.registers.get(&0x2121).copied().unwrap_or(0);
                if (cgram_addr as usize) < self.cgram.len() {
                    self.cgram[cgram_addr as usize] = value;
                }
            }
            
            _ => { self.registers.insert(addr, value); }
        }
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
            self.cgram[cgram_addr as usize] = value;
        }
    }
}