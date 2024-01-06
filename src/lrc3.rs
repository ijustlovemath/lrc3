
#[derive(Debug, Copy, Clone)]
pub enum RegisterName {
    R0, R1, R2, R3, R4, R5, R6, R7,
    PC, IR, MDR, MAR
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterContents(u16);

impl RegisterContents {
    fn init() -> Self {
        Self::new(0)
    }

    fn new(data: u16) -> Self {
        Self{ 0: data }
    }

    //fn from_instruction(
}

impl RegisterName {
    pub fn index(&self) -> usize {
        match self {
            Self::R0 => 0,
            Self::R1 => 1,
            Self::R2 => 2,
            Self::R3 => 3,
            Self::R4 => 4,
            Self::R5 => 5,
            Self::R6 => 6,
            Self::R7 => 7,
            _ => panic!("unhandled case for register indexing, self={:?}")
        }
    }

}

#[derive(Debug)]
pub struct Register {
    content: RegisterContents,
    id: RegisterName
}

struct Memory {
    memory: [RegisterContents; 65536]
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [RegisterContents::init(); 65536];
        memory[0x3000] = RegisterContents::new(0xfe00);
        Self {
            memory: memory
        }
    }
}

impl Register {
    pub fn new(content: RegisterContents, id: RegisterName) -> Self {
        Self {
            content, id
        }
    }

    pub fn ir_from_bits(bits: u16) -> Self {
        Self::new(RegisterContents::new(bits), RegisterName::IR)
    }
}

#[derive(Debug)]
pub struct Regfile {
    registers: [Register; 8]
}

impl Regfile {
    pub fn contents_of(&self, reg: RegisterName) -> RegisterContents {
        self.registers[reg.index()].content
    }

    pub fn set_contents_of(&mut self, reg: RegisterName, data: RegisterContents) {
        self.registers[reg.index()] = Register::new(data, reg);
    }

    pub fn new() -> Self {
        Self { 
            registers: [
                Register::new(RegisterContents::init(), RegisterName::R0),
                Register::new(RegisterContents::init(), RegisterName::R1),
                Register::new(RegisterContents::init(), RegisterName::R2),
                Register::new(RegisterContents::init(), RegisterName::R3),
                Register::new(RegisterContents::init(), RegisterName::R4),
                Register::new(RegisterContents::init(), RegisterName::R5),
                Register::new(RegisterContents::init(), RegisterName::R6),
                Register::new(RegisterContents::init(), RegisterName::R7),
            ]
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Opcode {
    ADD, AND, NOT,
    LD, LDI, LDR, LEA, ST, STR, STI,
    BR, JSR, JMP, RTI,
    TRAP
}

impl Opcode {
    pub fn from_ir_bits(ir_bits: u16) -> Self {
        match (ir_bits >> 12) & 0xf {
            0b1001 => Self::NOT,
            0b0001 => Self::ADD,
            0b0101 => Self::AND,
            0b0010 => Self::LD ,
            0b0011 => Self::ST ,
            0b1010 => Self::LDI,
            0b1011 => Self::STI,
            0b0110 => Self::LDR,


            _ => panic!("opcode {:?} not implemented", ir_bits)
        }
    }

    pub fn from_ir_content(ir_content: &RegisterContents) -> Self {
        Self::from_ir_bits(ir_content.0)
    }

    pub fn from_ir(ir: &Register) -> Self {
        match ir.id {
            RegisterName::IR => Self::from_ir_content(&ir.content),
            _ => panic!("Not allowed to build opcode from register ({:?}) that isn't IR")
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,

}
impl Instruction {
    pub fn decode_bits(bits: u16) -> Self {
        Self {
            opcode: Opcode::from_ir_bits(bits)
        }
    }

    pub fn decode_ir(ir: &Register) -> Self {
        Self {
            opcode: Opcode::from_ir(ir)
        }
    }

}
