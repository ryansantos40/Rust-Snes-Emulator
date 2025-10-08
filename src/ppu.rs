use crate::memory::Memory;
use crate::ppu_registers::*;

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

    pub inidisp: u8,
    pub obsel: u8,
    pub oamaddl: u8,
    pub oamaddh: u8,
    pub oamdata: u8,

    pub bg_mode_reg: u8,
    pub mosaic: u8,

    pub bg_tilemap_addr: [u16; 4],
    pub bg_char_addr: [u16; 4],

    pub mode7_settings: u8,
    pub mode7_matrix: [i16; 4],
    pub mode7_center: [i16; 2],

    pub window_settings: [u8; 3],
    pub window_positions: [u8; 4],
    pub window_logic: [u8; 2],

    pub main_screen_enabled: u8,
    pub sub_screen_enabled: u8,
    pub main_window_mask: u8,
    pub sub_window_mask: u8,

    pub color_math_control_a: u8,
    pub color_math_control_b: u8,
    pub fixed_color_data: u8,
    pub screen_mode: u8,

    pub vmain: u8,
    pub vmadd: u8,

    pub vram_read_buffer: u16,
    pub open_bus: u8,
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

            inidisp: 0x80,
            obsel: 0,
            oamaddl: 0,
            oamaddh: 0,
            oamdata: 0,
            bg_mode_reg: 0,
            mosaic: 0,

            bg_tilemap_addr: [0; 4],
            bg_char_addr: [0; 4],
            
            mode7_settings: 0,
            mode7_matrix: [0; 4],
            mode7_center: [0; 2],
            
            window_settings: [0; 3],
            window_positions: [0; 4],
            window_logic: [0; 2],
            
            main_screen_enabled: 0,
            sub_screen_enabled: 0,
            main_window_mask: 0,
            sub_window_mask: 0,
            
            color_math_control_a: 0,
            color_math_control_b: 0,
            fixed_color_data: 0,
            screen_mode: 0,

            vmain: 0,
            vmadd: 0,

            vram_read_buffer: 0,
            open_bus: 0,
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

        let mut priority_buffer = [0u8; 256];

        match self.video_mode {
            VideoMode::Mode0 => {
                for priority in (0..=1).rev(){
                    for bg in 0..4 {
                        if self.bg_enabled[bg] {
                            self.render_bg_mode0(memory, bg, priority, &mut priority_buffer);
                        }
                    }
                }
            }

            VideoMode::Mode1 => {
                for priority in (0..=1).rev() {
                    if self.bg_enabled[2] {
                        self.render_bg_mode1_2bpp(memory, 2, priority, &mut priority_buffer);
                    }

                    if self.bg_enabled[0] {
                        self.render_bg_mode1_4bpp(memory, 0, priority, &mut priority_buffer);
                    }

                    if self.bg_enabled[1] {
                        self.render_bg_mode1_4bpp(memory, 1, priority, &mut priority_buffer);
                    }
                }
            }

            _ => {
                // Other modes not implemented yet
            }
        }

        if self.sprites_enabled {
            self.render_sprites(memory, &mut priority_buffer);
        }

        for x in 0..256 {
            let color_index = self.line_buffer[x];
            let rgb_color = self.get_color_from_cgram(memory, color_index);
            let fb_index = (self.scanline as usize) * 256 + x;
            if fb_index < self.framebuffer.len() {
                self.framebuffer[fb_index] = self.apply_brightness(rgb_color);
            }
        }
    }

    fn render_bg_mode0(&mut self, memory: &Memory, bg_layer: usize, target_priority: u8, priority_buffer: &mut [u8; 256]) {
        let scroll_x = self.bg_hscroll[bg_layer];
        let scroll_y = self.bg_vscroll[bg_layer];

        let y_pos = (self.scanline as u16 + scroll_y) & 0x1FF;
        let tile_y = y_pos / 8;
        let pixel_y = y_pos % 8;

        let tilemap_width = if self.bg_size[bg_layer] { 64 } else { 32 };

        for tile_x_screen in 0..33 {
            let x_pos = (tile_x_screen * 8 + scroll_x) & 0x1FF;
            let tile_x = (x_pos / 8) % tilemap_width;

            let tile_info = self.get_bg_tile_index(memory, bg_layer, tile_x, tile_y);
            let tile_number = tile_info & 0x03FF;
            let palette = ((tile_info >> 10) & 0x07) as u8;
            let tile_priority = ((tile_info >> 13) & 0x01) as u8;
            let flip_x = ((tile_info >> 14) & 0x01) != 0;
            let flip_y = ((tile_info >> 15) & 0x01) != 0;

            if tile_priority != target_priority {
                continue;
            }

            let actual_pixel_y = if flip_y { 7 - pixel_y } else { pixel_y };
            let tile_data = self.get_tile_data_2bpp(memory, bg_layer, tile_number, actual_pixel_y);
            
            for pixel_x in 0..8 {
                let screen_x = ((x_pos + pixel_x) & 0x1FF) as usize;

                if screen_x < 256 {
                    let actual_pixel_x = if flip_x { 7 - pixel_x } else { pixel_x };
                    let color_index = ((tile_data >> (actual_pixel_x * 2)) & 0x03) as u8;

                    if color_index != 0 {
                        let current_priority = priority_buffer[screen_x];
                        if tile_priority >= current_priority {
                            let cgram_index = (bg_layer as u8 * 32) + (palette * 4) + color_index;
                            self.line_buffer[screen_x] = cgram_index;
                            priority_buffer[screen_x] = tile_priority;
                        }
                    }
                }
            }
        }
    }

    fn render_bg_mode1_4bpp(&mut self, memory: &Memory, bg_layer: usize, target_priority: u8, priority_buffer: &mut [u8; 256]) {
        let scroll_x = self.bg_hscroll[bg_layer];
        let scroll_y = self.bg_vscroll[bg_layer];

        let y_pos = (self.scanline as u16 + scroll_y) & 0x1FF;
        let tile_y = y_pos / 8;
        let pixel_y = y_pos % 8;

        let tilemap_width = if self.bg_size[bg_layer] { 64 } else { 32 };

        for tile_x_screen in 0..33 {
            let x_pos = (tile_x_screen * 8 + scroll_x) & 0x1FF;
            let tile_x = (x_pos / 8) % tilemap_width;

            let tile_info = self.get_bg_tile_index(memory, bg_layer, tile_x, tile_y);
            let tile_number = tile_info & 0x03FF;
            let palette = ((tile_info >> 10) & 0x07) as u8;
            let tile_priority = ((tile_info >> 13) & 0x01) as u8;
            let flip_x = ((tile_info >> 14) & 0x01) != 0;
            let flip_y = ((tile_info >> 15) & 0x01) != 0;

            if tile_priority != target_priority {
                continue;
            }

            let actual_pixel_y = if flip_y { 7 - pixel_y } else { pixel_y };

            let tile_data = self.get_tile_data_4bpp(memory, bg_layer,tile_number, actual_pixel_y);

            for pixel_x in 0..8 {
                let screen_x = ((x_pos + pixel_x) & 0x1FF) as usize;

                if screen_x < 256 {
                    let actual_pixel_x = if flip_x { 7 - pixel_x } else { pixel_x };
                    let color_index = ((tile_data >> (actual_pixel_x * 4)) & 0x0F) as u8;
                    if color_index != 0 {
                        let current_priority = priority_buffer[screen_x];

                        if tile_priority >= current_priority {
                            let cgram_index = (palette * 16) + color_index;
                            self.line_buffer[screen_x] = cgram_index;
                            priority_buffer[screen_x] = tile_priority;
                        }
                    }
                }
            }

        }
    }

    fn render_bg_mode1_2bpp(&mut self, memory: &Memory, bg_layer: usize, target_priority: u8, priority_buffer: &mut [u8; 256]) {
        let scroll_x = self.bg_hscroll[bg_layer];
        let scroll_y = self.bg_vscroll[bg_layer];

        let y_pos = (self.scanline as u16 + scroll_y) & 0x1FF;
        let tile_y = y_pos / 8;
        let pixel_y = y_pos % 8;

        let tilemap_width = if self.bg_size[bg_layer] { 64 } else { 32 };

        for tile_x_screen in 0..33 {
            let x_pos = (tile_x_screen * 8 + scroll_x) & 0x1FF;
            let tile_x = (x_pos / 8) % tilemap_width;

            let tile_info = self.get_bg_tile_index(memory, bg_layer, tile_x, tile_y);
            let tile_number = tile_info & 0x03FF;
            let palette = ((tile_info >> 10) & 0x07) as u8;
            let tile_priority = ((tile_info >> 13) & 0x01) as u8;
            let flip_x = ((tile_info >> 14) & 0x01) != 0;
            let flip_y = ((tile_info >> 15) & 0x01) != 0;

            if tile_priority != target_priority {
                continue;
            }

            let actual_pixel_y = if flip_y { 7 - pixel_y } else { pixel_y };
            let tile_data = self.get_tile_data(memory, tile_number, actual_pixel_y);

            for pixel_x in 0..8 {
                let screen_x = ((x_pos + pixel_x) & 0x1FF) as usize;

                if screen_x < 256 {
                    let actual_pixel_x = if flip_x { 7 - pixel_x } else { pixel_x };
                    let color_index = ((tile_data >> (actual_pixel_x * 2)) & 0x03) as u8;

                    if color_index != 0 {
                        let current_priority = priority_buffer[screen_x];

                        if tile_priority >= current_priority {
                            let cgram_index = 128 + (palette * 4) + color_index;
                            self.line_buffer[screen_x] = cgram_index;
                            priority_buffer[screen_x] = tile_priority;
                        }
                    }
                }
            }
        }
    }

    fn get_bg_tile_info(&self, memory: &Memory, bg_layer: usize, tile_x: u16, tile_y: u16) -> u16 {
        let tilemap_addr = self.bg_tilemap_addr[bg_layer] as usize;
        let tilemap_width = if self.bg_size[bg_layer] { 64 } else { 32 };

        let block_x = tile_x / 32;
        let block_y = tile_y / 32;
        let local_x = tile_x % 32;
        let local_y = tile_y % 32;

        let block_offset = match (block_x, block_y) {
            (0, 0) => 0x0000,
            (1, 0) => 0x0400,
            (0, 1) => 0x0800,
            (1, 1) => 0x0C00,
            _ => 0x0000,
        };

        let tile_offset = (local_y * 32 + local_x) * 2;
        let tile_addr = tilemap_addr + block_offset + tile_offset as usize;

        if tile_addr + 1 < memory.vram.len() {
            let low = memory.vram[tile_addr] as u16;
            let high = memory.vram[tile_addr + 1] as u16;
            (high << 8) | low

        } else {
            0
        }
    }

    fn get_tile_data_4bpp(&self, memory: &Memory, bg_layer: usize, tile_number: u16, pixel_row: u16) -> u64 {
        let char_base = self.bg_char_addr[bg_layer] as usize;

        let tile_addr = char_base + (tile_number as usize * 32) + (pixel_row as usize * 2);

        if tile_addr + 24 < memory.vram.len() {
            let plane0 = memory.vram[tile_addr] as u64;
            let plane1 = memory.vram[tile_addr + 1] as u64;
            let plane2 = memory.vram[tile_addr + 16] as u64;
            let plane3 = memory.vram[tile_addr + 17] as u64;

            let mut pixel_data = 0u64;
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

    fn get_tile_data_2bpp(&self, memory: &Memory, bg_layer: usize, tile_number: u16, pixel_row: u16) -> u32 {
        let char_base = self.bg_char_addr[bg_layer] as usize;
        let tile_addr = char_base + (tile_number as usize * 16) + (pixel_row as usize * 2);

        if tile_addr + 1 < memory.vram.len() {
            let plane0 = memory.vram[tile_addr] as u32;
            let plane1 = memory.vram[tile_addr + 1] as u32;

            let mut pixel_data = 0u32;
            for bit in 0..8 {
                let color = ((plane0 >> bit) & 1) |
                            ((plane1 >> bit) & 1) << 1;
                pixel_data |= color << (bit * 2);
            }
            pixel_data

        } else {
            0
        }
    }

    fn apply_brightness(&self, color: u32) -> u32 {
        if self.forced_blank {
            return 0x00000000;
        }

        if self.brightness == 0 {
            return 0x00000000;
        }

        if self.brightness == 15 {
            return color;
        }

        let r = ((color >> 16) & 0xFF) as u32;
        let g = ((color >> 8) & 0xFF) as u32;
        let b = (color & 0xFF) as u32;

        let brightness_factor = (self.brightness as u32 + 1) * 17;
        let r_adjusted = (r * brightness_factor) / 255;
        let g_adjusted = (g * brightness_factor) / 255;
        let b_adjusted = (b * brightness_factor) / 255;

        (r_adjusted << 16) | (g_adjusted << 8) | b_adjusted
    }

    fn get_bg_tile_index(&self, memory: &Memory, bg_layer: usize, tile_x: u16, tile_y: u16) -> u16 {
        let tilemap_addr = self.bg_tilemap_addr[bg_layer] as usize;
        let tile_addr = tilemap_addr + ((tile_y * 32 + tile_x) * 2) as usize;

        if tile_addr + 1 < memory.vram.len() {
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

    fn render_sprites(&mut self, memory: &Memory, priority_buffer: &mut [u8; 256]) {
        for sprite in 0..128 {
            let oam_addr = sprite * 4;

            if oam_addr + 3 < memory.oam.len() {
                let x = memory.oam[oam_addr] as u16;
                let y = memory.oam[oam_addr + 1] as u16;
                let tile = memory.oam[oam_addr + 2] as u16;
                let attr = memory.oam[oam_addr + 3];

                let sprite_priority = ((attr >> 4) & 0x03) +2;

                if y <= self.scanline && self.scanline < y + 8 {
                    let sprite_y = self.scanline - y;
                    let sprite_data = self.get_sprite_data(memory, tile, sprite_y);

                    for pixel_x in 0..8 {
                        let screen_x = (x + pixel_x) as usize;

                        if screen_x < 256 {
                            let color_index = (sprite_data >> (pixel_x * 4)) & 0x0F;
                            if color_index != 0 {
                                let current_priority = priority_buffer[screen_x];
                                if sprite_priority >= current_priority {
                                    self.line_buffer[screen_x] = color_index as u8 + 16;
                                    priority_buffer[screen_x] = sprite_priority;
                                }
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
            INIDISP => {
                self.brightness = value & 0x0F;
                self.forced_blank = (value & 0x80) != 0;
                self.inidisp = value;
            }

            OBSEL => {
                self.sprite_size = value & 0x07;
                self.obsel = value;
            }

            BGMODE => {
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
                self.bg_mode_reg = value;
            }

            MOSAIC => {
                self.mosaic = value;
            }

            BG1SC => {
                self.bg_tilemap_addr[0] = ((value as u16) & 0xFC) << 8;
            }

            BG2SC => {
                self.bg_tilemap_addr[1] = ((value as u16) & 0xFC) << 8;
            }

            BG3SC => {
                self.bg_tilemap_addr[2] = ((value as u16) & 0xFC) << 8;
            }

            BG4SC => {
                self.bg_tilemap_addr[3] = ((value as u16) & 0xFC) << 8;
            }

            BG12NBA => {
                self.bg_char_addr[0] = ((value & 0x0F) as u16) << 12;
                self.bg_char_addr[1] = ((value & 0xF0) as u16) << 8;
            }

            BG34NBA => {
                self.bg_char_addr[2] = ((value & 0x0F) as u16) << 12;
                self.bg_char_addr[3] = ((value & 0xF0) as u16) << 8;
            }

            M7SEL => {
                self.mode7_settings = value;
            }

            W12SEL => {
                self.window_settings[0] = value;
            }

            W34SEL => {
                self.window_settings[1] = value;
            }

            WOBJSEL => {
                self.window_settings[2] = value;
            }

            WH0 => {
                self.window_positions[0] = value;
            }

            WH1 => {
                self.window_positions[1] = value;
            }

            WH2 => {
                self.window_positions[2] = value;
            }

            WH3 => {
                self.window_positions[3] = value;
            }

            WBGLOG => {
                self.window_logic[0] = value;
            }

            WOBJLOG => {
                self.window_logic[1] = value;
            }

            TM => {
                self.bg_enabled[0] = (value & 0x01) != 0;
                self.bg_enabled[1] = (value & 0x02) != 0;
                self.bg_enabled[2] = (value & 0x04) != 0;
                self.bg_enabled[3] = (value & 0x08) != 0;
                self.sprites_enabled = (value & 0x10) != 0;
                self.main_screen_enabled = value;
            }

            TS => {
                self.sub_screen_enabled = value;
            }

            TMW => {
                self.main_window_mask = value;
            }

            TSW => {
                self.sub_window_mask = value;
            }

            CGWSEL => {
                self.color_math_control_a = value;
            }

            CGADSUB => {
                self.color_math_control_b = value;
            }

            COLDATA => {
                self.fixed_color_data = value;
            }

            SETINI => {
                self.screen_mode = value;
            }

            VMAIN => {
                self.vmain = value;
                self.vram_increment = match value & 0x03 {
                    0 => 1,
                    1 => 32,
                    2 => 128,
                    3 => 128,
                    _ => 1,
                };
            }

            VMADDL => {
                self.vram_addr = (self.vram_addr & 0xFF00) | (value as u16);
            }

            VMADDH => {
                self.vram_addr = (self.vram_addr & 0x00FF) | ((value as u16) << 8);
            }

            CGADD => {
                self.cgram_addr = (value as u16) & 0x1FF;
            }

            _ => {}
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {

            SLHV => {
                0 //placeholder
            }

            OPHCT => {
                (self.cycle & 0xFF) as u8
            }

            OPVCT => {
                (self.scanline & 0xFF) as u8
            }

            STAT77 => {
                let mut status = 0x01;
                if self.vblank { status |= 0x80; }
                if self.hblank { status |= 0x40; }
                status
            }

            STAT78 => {
                let mut status = 0x03;
                if self.nmi_flag { status |= 0x80; }
                self.nmi_flag = false;
                status
            }

            0x4210 => {
                let mut value = 0x02;

                if self.nmi_enabled { value |= 0x80; }

                self.nmi_flag = false;

                value
            }

            0x4212 => {
                let mut status = 0;

                if self.vblank { status |= 0x80; }
                if self.hblank { status |= 0x40; }


                status
            }

            _ => {
                //placeholder
                self.open_bus
            }
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