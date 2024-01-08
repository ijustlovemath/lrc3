
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
    BR, JSR, JSRR, JMP, RET, RTI,
    TRAP
}

impl Opcode {
    pub fn from_ir_bits(ir_bits: u16) -> Self {
        match (ir_bits >> 12) & 0xf {
            /* The highest 4 bits of the instruction register,
             * IR[15:12], always contain the opcode.
             *
             * Legend -> instruction name, + if they set cc, 
             * (eg LD+ has instruction name LD, and
             * it sets the condition codes for jumps)
             * This is followed by a colon, 
             * then all fields of instruction from 
             * highest bit to lowest of IR, separated by semicolon
             *
             * dr, sr1, sr2, sr, baser take 3 bits each
             * 
             * constants in the instruction, eg 0b1, are bits which are always set/cleared in a
             * valid instruction
             * 
             * nzp all take 1 bit each
             * 
             * other fields are labeled with the 
             * number of bits they take at the end, 
             * eg imm5 takes 5 bits
             */
            0b0000 => Self::BR,   /* BR      : n; z; p; pcoffset9 */
            0b0001 => Self::ADD,  /* ADD+    : dr; sr1; 0b000; sr2 
                                   * ADDi+   : dr; sr1; 0b1; imm5
                                   */
            0b0010 => Self::LD ,  /* LD+     : dr; pcoffset9 */
            0b1010 => Self::LDI,  /* LDi+    : dr; pcoffset9 */
            0b0011 => Self::ST ,  /* ST      : sr; pcoffset9 */
            0b1011 => Self::STI,  /* STI     : sr; pcoffset9 */
            0b0101 => Self::AND,  /* AND+    : dr; sr1; 0b000; sr2 
                                   * ANDi+   : dr; sr1; 0b1; imm5
                                   */
            0b0110 => Self::LDR,  /* LDR+    : dr; baser; offset6 */
            0b1000 => Self::RTI,  /* RTI     : 0x000 */
            0b1001 => Self::NOT,  /* NOT+    : dr; sr; 0b111111 */
            0b1011 => Self::STI,  /* STI     : sr; pcoffset9 */
            0b1100 => Self::JMP,  /* JMP     : 0b000; baser; 0b000000 */
            0b1110 => Self::LEA,  /* LEA+    : dr; pcoffset9 */
            0b1111 => Self::TRAP, /* TRAP    : 0b0000; trapvect8 */

            _ => panic!("opcode {:?} not implemented", ir_bits)
        }
    }

    pub fn from_ir_content(ir_content: &RegisterContents) -> Self {
        Self::from_ir_bits(ir_content.0)
    }

    pub fn from_ir(ir: &Register) -> Self {
        match ir.id {
            RegisterName::IR => Self::from_ir_content(&ir.content),
            _ => panic!("Not allowed to build opcode from register ({:?}) that isn't IR", ir.id)
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    sr1: Option<RegisterName>,
    sr2: Option<RegisterName>,
    dr: Option<RegisterName>,
    baser: Option<RegisterName>,

}
impl Instruction {
    pub fn decode_bits(bits: u16) -> Self {
        Self {
            opcode: Opcode::from_ir_bits(bits),
            sr1: None,
            sr2: None,
            dr: None,
            baser: None,
        }
    }

    pub fn decode_ir(ir: &Register) -> Self {
        let opcode = Opcode::from_ir(ir);
        Self {
            opcode: opcode,
            sr1: None,
            sr2: None,
            dr: None,
            baser: None,
        }
    }

}

#[derive(Debug)]
pub struct Datapath {
    regfile: Regfile,
    
    ld_pc: bool,
    ld_ir: bool,
    ld_mar: bool,
    ld_reg: bool,
    ld_cc: bool,

    n: bool,
    z: bool,
    p: bool,

    gate_pc: bool,
    gate_marmux: bool,
    gate_alu: bool,
 
    pc_mux: u8,
    addr1_mux: bool,
    addr2_mux: u8,
    sr2_mux: bool,
    aluk: u8,
}

pub struct Lrc3 {
    datapath: Datapath,
    memory: Memory,
    mar: Register,
    ir: Register,
    mdr: Register,
}

impl Lrc3 {
    pub fn fsm(&mut self) {
        
    }
}
