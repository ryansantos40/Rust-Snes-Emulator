use crate::memory::Memory;

#[derive(Debug, Clone, Copy)]
pub enum VideoMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7,
}

pub struct Ppu {
    //Timing
    pub scanline: u16,
    pub cycle: u16,
    pub frame_complete: bool,
    pub vblank: bool,
    pub hblank: bool,

    pub video_mode: VideoMode,
    pub brightness: u8,
    pub forced_blank: bool,

    pub bg_enabled: [bool; 4],
    pub bg_mode: [u8; 4],
    pub bg_priority: [u8; 4],
    pub bg_size: [bool; 4],

    pub sprites_enabled: bool,
    pub sprite_size: u8,

    pub bg_hscroll: [u16; 4],
    pub bg_vscroll: [u16; 4],

    pub vram_addr: u16,
    pub vram_increment: u16,

    pub oam_addr: u16,

    pub cgram_addr: u16,

    pub framebuffer: Vec<u32>,
    pub line_buffer: [u8; 256],

    pub nmi_enabled: bool,
    pub nmi_flag: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            scanline: 0,
            cycle: 0,
            frame_complete: false,
            vblank: false,
            hblank: false,

            video_mode: VideoMode::Mode0,
            brightness: 0,
            forced_blank: true,

            bg_enabled: [false; 4],
            bg_mode: [0; 4],
            bg_priority: [0; 4],
            bg_size: [false; 4],

            sprites_enabled: false,
            sprite_size: 0,

            bg_hscroll: [0; 4],
            bg_vscroll: [0; 4],

            vram_addr: 0,
            vram_increment: 1,

            oam_addr: 0,
            cgram_addr: 0,

            framebuffer: vec![0; 256 * 224],
            line_buffer: [0; 256],

            nmi_enabled: false,
            nmi_flag: false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn step(&mut self, memory: &mut Memory) -> bool {
        let mut nmi_triggered = false;

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            match self.scanline {
                0..=223 => {
                    if !self.forced_blank {
                        self.render_scanline(memory);
                    }
                    self.vblank = false;
                }

                224 => {
                    self.vblank = true;
                    self.frame_complete = true;

                    if self.nmi_enabled {
                        self.nmi_flag = true;
                        nmi_triggered = true;
                    }
                }

                225..=261 => {
                    self.vblank = true;
                }

                262 => {
                    self.scanline = 0;
                    self.frame_complete = false;
                    self.nmi_flag = false;
                }

                _ => {}
            }
        }

        self.hblank = self.cycle >= 256;

        nmi_triggered
    }

    fn render_scanline(&mut self, memory: &mut Memory) {
        self.line_buffer.fill(0);

        match self.video_mode {
            VideoMode::Mode0 => {
                for bg in 0..4 {
                    if self.bg_enabled[bg] {
                        self.render_bg_mode0(memory, bg);
                    }
                }
            }

            _ => {
                // Other modes not implemented yet
            }
        }

        if self.sprites_enabled {
            self.render_sprites(memory);
        }

        for x in 0..256 {
            let color_index = self.line_buffer[x];
            let rgb_color = self.get_color_from_cgram(memory, color_index);
            let fb_index = (self.scanline as usize) * 256 + x;
            if fb_index < self.framebuffer.len() {
                self.framebuffer[fb_index] = rgb_color;
            }
        }
    }

    fn render_bg_mode0(&mut self, memory: &Memory, bg_layer: usize) {
        let scroll_x = self.bg_hscroll[bg_layer];
        let scroll_y = self.bg_vscroll[bg_layer];

        let y_pos = (self.scanline as u16 + scroll_y) % 256;
        let tile_y = y_pos / 8;
        let pixel_y = y_pos % 8;

        for tile_x in 0..32 {
            let x_pos = (tile_x * 8 + scroll_x) % 256;

            let tile_index = self.get_bg_tile_index(memory, bg_layer, tile_x as u16, tile_y);
            let tile_data = self.get_tile_data(memory, tile_index, pixel_y);

            for pixel_x in 0..8 {
                let screen_x = ((x_pos + pixel_x) % 256) as usize;
                
                if screen_x < 256 {
                    let color_index = (tile_data >> (pixel_x * 2)) & 0x03;
                    if color_index != 0 {
                        self.line_buffer[screen_x] = color_index as u8;
                    }
                }
            }
        }
    }

    fn get_bg_tile_index(&self, memory: &Memory, bg_layer: usize, tile_x: u16, tile_y: u16) -> u16 {
        let tilemap_addr = 0x0000 + (bg_layer * 0x800);
        let tile_addr = tilemap_addr + ((tile_y * 32 + tile_x) * 2) as usize;

        if tile_addr < memory.vram.len() {
            let low = memory.vram[tile_addr] as u16;
            let high = memory.vram[tile_addr + 1] as u16;
            (high << 8) | low
        } else {
            0
        }
    }

    fn get_tile_data(&self, memory: &Memory, tile_index: u16, pixel_row: u16) -> u32 {
        let tile_addr = (tile_index * 32 + pixel_row * 4) as usize;

        if tile_addr + 3 < memory.vram.len() {
            let plane0 = memory.vram[tile_addr] as u32;
            let plane1 = memory.vram[tile_addr + 1] as u32;
            let plane2 = memory.vram[tile_addr + 2] as u32;
            let plane3 = memory.vram[tile_addr + 3] as u32;

            let mut pixel_data = 0;
            for bit in 0..8 {
                let color = ((plane0 >> bit) & 1) |
                            ((plane1 >> bit) & 1) << 1 |
                            ((plane2 >> bit) & 1) << 2 |
                            ((plane3 >> bit) & 1) << 3;
                pixel_data |= color << (bit * 4);      
            }

            pixel_data
        } else {
            0
        }
    }

    fn render_sprites(&mut self, memory: &Memory) {
        for sprite in 0..128 {
            let oam_addr = sprite * 4;

            if oam_addr + 3 < memory.oam.len() {
                let x = memory.oam[oam_addr] as u16;
                let y = memory.oam[oam_addr + 1] as u16;
                let tile = memory.oam[oam_addr + 2] as u16;
                let attr = memory.oam[oam_addr + 3];

                if y <= self.scanline && self.scanline < y + 8 {
                    let sprite_y = self.scanline - y;
                    let sprite_data = self.get_sprite_data(memory, tile, sprite_y);

                    for pixel_x in 0..8 {
                        let screen_x = (x + pixel_x) as usize;

                        if screen_x < 256 {
                            let color_index = (sprite_data >> (pixel_x * 4)) & 0x0F;
                            if color_index != 0 {
                                self.line_buffer[screen_x] = color_index as u8 + 16;
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_sprite_data(&self, memory: &Memory, tile_index: u16, pixel_row: u16) -> u32 {
        let tile_addr = (0x4000 + tile_index * 32 + pixel_row * 4) as usize;

        if tile_addr + 3 < memory.vram.len() {
            let plane0 = memory.vram[tile_addr] as u32;
            let plane1 = memory.vram[tile_addr + 1] as u32;
            let plane2 = memory.vram[tile_addr + 2] as u32;
            let plane3 = memory.vram[tile_addr + 3] as u32;

            let mut pixel_data = 0;
            for bit in 0..8 {
                let color = ((plane0 >> bit) & 1) |
                            ((plane1 >> bit) & 1) << 1 |
                            ((plane2 >> bit) & 1) << 2 |
                            ((plane3 >> bit) & 1) << 3;
                pixel_data |= color << (bit * 4);      
            }
            pixel_data

        } else {
            0
        }
    }

    fn get_color_from_cgram(&self, memory: &Memory, color_index: u8) -> u32 {
        if color_index == 0 {
            return 0x00000000;
        }

        let cgram_addr = (color_index as usize * 2) % memory.cgram.len();
        if cgram_addr + 1 < memory.cgram.len() {
            let low = memory.cgram[cgram_addr] as u16;
            let high = memory.cgram[cgram_addr + 1] as u16;
            let color_15bit = (high << 8) | low;

            let r = ((color_15bit & 0x1F) << 3) as u32;
            let g = (((color_15bit >> 5) & 0x1F) << 3) as u32;
            let b = (((color_15bit >> 10) & 0x1F) << 3) as u32;

            (r << 16) | (g << 8) | b

        } else {
            0x00000000
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x2100 => {
                self.brightness = value & 0x0F;
                self.forced_blank = (value & 0x80) != 0;
            }

            0x2101 => {
                self.sprite_size = value & 0x07;
            }

            0x2105 => {
                self.video_mode = match value & 0x07 {
                    0 => VideoMode::Mode0,
                    1 => VideoMode::Mode1,
                    2 => VideoMode::Mode2,
                    3 => VideoMode::Mode3,
                    4 => VideoMode::Mode4,
                    5 => VideoMode::Mode5,
                    6 => VideoMode::Mode6,
                    7 => VideoMode::Mode7,
                    _ => VideoMode::Mode0,
                };

                self.bg_size[0] = (value & 0x10) != 0;
                self.bg_size[1] = (value & 0x20) != 0;
                self.bg_size[2] = (value & 0x40) != 0;
                self.bg_size[3] = (value & 0x80) != 0;
            }

            0x212C => {
                self.bg_enabled[0] = (value & 0x01) != 0;
                self.bg_enabled[1] = (value & 0x02) != 0;
                self.bg_enabled[2] = (value & 0x04) != 0;
                self.bg_enabled[3] = (value & 0x08) != 0;
                self.sprites_enabled = (value & 0x10) != 0;
            }

            0x4200 => {
                self.nmi_enabled = (value & 0x80) != 0;
            }

            _ => {}
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0x2137 => {
                0
            }

            0x213E => {
                let mut status = 0;
                if self.vblank { status |= 0x80; }
                if self.hblank { status |= 0x40; }
                status
            }

            0x213F => {
                let mut status = 0;
                if self.nmi_flag { status |= 0x80; }
                status
            }

            _ => 0
        }
    }

    pub fn get_framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    pub fn frame_ready(&mut self) -> bool {
        if self.frame_complete {
            self.frame_complete = false;
            true
        } else {
            false
        }
    }
}