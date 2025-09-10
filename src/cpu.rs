pub struct Cpu {
    pub a: u16, // Accumulator
    pub x: u16, // Index Register X
    pub y: u16, // Index Register Y
    pub sp: u16, // Stack Pointer
    pub pc: u16, // Program Counter
    pub dp: u16, // Direct Page Register
    pub db: u8,  // Data Bank Register
    pub pb: u8,  // Program Bank Register

    pub p: u8,   // Processor Status Register

    pub m_flag: bool, // Memory/Accumulator Flag (0 = 16-bit, 1 = 8-bit)
    pub x_flag: bool, // Index Register Flag (0 = 16-bit, 1 = 8-bit)
    pub e_flag: bool, // Emulation Mode Flag (1 = Emulation Mode, 0 = Native Mode)
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0x01FF, // Stack Pointer starts at 0x1FF in Emulation Mode
            pc: 0x008000,
            dp: 0x0000,
            db: 0x00,
            pb: 0x00,
            p: 0x34, // Default status register value in Emulation Mode
            m_flag: true, // Start in Emulation Mode (8-bit accumulator)
            x_flag: true, // Start in Emulation Mode (8-bit index registers)
            e_flag: true, // Start in Emulation Mode
        }
    }

    pub fn step(&mut self, memory: &mut crate::memory::Memory) {
        //Placeholder
        let opcode = memory.read(self.pc.into());
        self.pc += 1;

        //TODO: Implementar decoficação e execução de instruções
        match opcode {
            0xEA => {} // NOP
            _ => {
                println!("Opcode desconhecido: {:02X}", opcode);
                return;
            }   // Instrução desconhecida
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x008000;
        self.sp = 0x01FF;
        self.dp = 0x0000;
        self.db = 0x00;
        self.pb = 0x00;
        self.p = 0x34;
        self.m_flag = true;
        self.x_flag = true;
        self.e_flag = true;
    }
    
}