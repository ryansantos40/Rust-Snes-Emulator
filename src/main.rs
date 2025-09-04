use std::fs;
use std::io::Read;

mod cpu;
mod memoty;

fn main (){
    let mut file = fs::File::open("teste.teste").expect("Erro ao abrir o arquivo");
    let mut rom_data = Vec::new();
    file.read_to_end(&mut rom_data).expect("Erro ao ler o arquivo");

    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::Memory::new(rom_data);

    loop {
        cpu.step(&mut memory);
    }
}