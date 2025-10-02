use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

pub struct System {
    pub cpu: Cpu,
    pub ppu: Rc<RefCell<Ppu>>,
    pub memory: Memory,
}

impl System {
    pub fn new(rom: Vec<u8>) -> Self {
        let ppu = Rc::new(RefCell::new(Ppu::new()));

        System {
            cpu: Cpu::new(),
            memory: Memory::new(rom, Rc::clone(&ppu)),
            ppu,
        }
    }

    pub fn step(&mut self) -> u8 {
        let opcode = self.memory.read(self.cpu.pc);
        self.cpu.pc += 1;

        let cycles = self.cpu.execute_instruction(opcode, &mut self.memory);
        self.cpu.cycles += cycles as u64;

        let mut nmi_triggered = false;
        for _ in 0..(cycles * 4) {
            if self.ppu.borrow_mut().step(&mut self.memory) {
                nmi_triggered = true;
            }
        }

        if nmi_triggered && self.ppu.borrow().nmi_enabled && !self.cpu.get_flag(Cpu::FLAG_IRQ) {
            self.cpu.handle_nmi(&mut self.memory);
        }

        cycles
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
    }

    pub fn frame_ready(&self) -> bool {
        self.ppu.borrow_mut().frame_ready()
    }

    pub fn get_framebuffer(&self) -> Vec<u32> {
        self.ppu.borrow().get_framebuffer().to_vec()
    }

    pub fn get_ppu(&self) -> std::cell::Ref<Ppu> {
        self.ppu.borrow()
    }

    pub fn get_ppu_mut(&self) -> std::cell::RefMut<Ppu> {
        self.ppu.borrow_mut()
    }

    pub fn get_cpu_state(&self) -> String {
        self.cpu.get_register_state()
    }

    pub fn is_vblank(&self) -> bool {
        self.ppu.borrow().vblank
    }

    pub fn get_scanline(&self) -> u16 {
        self.ppu.borrow().scanline
    }
}