
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

    pub fn from_bits(bits: u16) -> Self {
        match bits & 0x7 {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            _ => unreachable!(),
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

//trait SetCc{};
//trait PcOffset9{};
//trait PcOffset11{};

#[derive(Debug)]
struct PcOffset9(u16);

#[derive(Debug)]
struct PcOffset11(u16);

#[derive(Debug)]
struct Imm5(u16);

#[derive(Debug)]
struct Offset6(u16);

#[derive(Debug)]
struct TrapVect(u16);

#[derive(Debug)]
struct BranchFlag(bool);

#[derive(Debug)]
struct TwoSourceArithArgs {
    dr: RegisterName,
    sr1: RegisterName,
    sr2: RegisterName,
}

#[derive(Debug)]
struct OneSourceArithArgs {
    dr: RegisterName,
    sr: RegisterName,
}

#[derive(Debug)]
struct ImmedArithArgs {
    dr: RegisterName,
    sr1: RegisterName,
    imm5: Imm5,
}

#[derive(Debug)]
struct BranchArgs {
    n: BranchFlag,
    z: BranchFlag,
    p: BranchFlag,
    pcoffset9: PcOffset9,
}

#[derive(Debug)]
struct BaseRArgs {
    base_r: RegisterName
}

#[derive(Debug)]
struct TrapArgs {
    trapvect8: TrapVect
}

#[derive(Debug)]
struct JsrArgs {
    pcoffset11: PcOffset11
}

#[derive(Debug)]
struct DrOffsetArgs {
    dr: RegisterName,
    pcoffset9: PcOffset9,
}

#[derive(Debug)]
struct DrBaseROff6Args {
    dr: RegisterName,
    base_r: RegisterName,
    offset6: Offset6,
}

#[derive(Debug)]
struct SrBaseROff6Args {
    sr: RegisterName,
    base_r: RegisterName,
    offset6: Offset6,
}

#[derive(Debug)]
struct SrOff9Args {
    sr: RegisterName,
    offset9: PcOffset9,
}

#[derive(Debug)]
pub enum InstructionArgs {
    Add(TwoSourceArithArgs),
    Addi(ImmedArithArgs),
    And(TwoSourceArithArgs),
    Andi(ImmedArithArgs),

    Br(BranchArgs),
    Jmp(BaseRArgs),
    Jsr(JsrArgs),
    Jsrr(BaseRArgs),

    Ld(DrOffsetArgs),
    Ldi(DrOffsetArgs),
    Ldr(DrBaseROff6Args),

    Lea(DrOffsetArgs),

    Not(OneSourceArithArgs),
    Rti(),
    
    St(SrOff9Args),
    Sti(SrOff9Args),
    Str(SrBaseROff6Args),
    Trap(TrapArgs)

}

#[derive(Debug)]
pub struct Instruction {
    opcode: Opcode,
    args: Option<InstructionArgs>,

}
impl Instruction {
    pub fn decode_bits(bits: u16) -> Self {
        let opcode = Opcode::from_ir_bits(bits);
        
        let arg9to11 = bits >> 9;
        let arg6to8 = bits >> 6;
        let imm5 = Imm5(bits & 0x1f);
        let off6 = Offset6(bits & 0x3f);
        let off9 = PcOffset9(bits & 0x1ff);
        let off11 = PcOffset11(bits & 0x7ff);
        let trap8 = TrapVect(bits & 0xff);

        let reg9to11 = RegisterName::from_bits(arg9to11);
        let reg6to8 = RegisterName::from_bits(arg6to8);
        let reg0to2 = RegisterName::from_bits(bits);

        let n = BranchFlag((bits >> 11) & 0b1 == 0b1);
        let z = BranchFlag((bits >> 10) & 0b1 == 0b1);
        let p = BranchFlag((bits >>  9) & 0b1 == 0b1);

        let args = match opcode {
            Opcode::ADD => Some({
                match (bits >> 5) & 0b1 {
                    0b0 => {
                        if ((bits >> 3) & 0b11) != 0b0 {
                            panic!("Illegally encoded ADD instruction: IR[3:4] != 0")
                        }
                        InstructionArgs::Add(TwoSourceArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        })
                    },
                    0b1 => {
                        InstructionArgs::Addi(ImmedArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            imm5: imm5,
                        })
                    },
                    _ => unreachable!(),
                }
            }),
            Opcode::AND => Some({
                match (bits >> 5) & 0b1 {
                    0b0 => {
                        if ((bits >> 3) & 0b11) != 0b0 {
                            panic!("Illegally encoded AND instruction: IR[3:4] != 0")
                        }
                        InstructionArgs::And(TwoSourceArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        })
                    },
                    0b1 => {
                        InstructionArgs::Andi(ImmedArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            imm5: imm5,
                        })
                    },
                    _ => unreachable!(),
                }
            }),
            Opcode::BR => Some({
                InstructionArgs::Br(BranchArgs{
                    n: n,
                    z: z,
                    p: p,
                    pcoffset9: off9
                })
            }),
            Opcode::JMP => Some({
                if ((bits & 0x3f) != 0b0) {
                    panic!("Illegally encoded JMP instruction: IR[0:5] != 0")
                }
                if ((bits >> 9) & 0b111 != 0b0) {
                    panic!("Illegally encoded JMP instruction: IR[9:11] != 0")
                }
                InstructionArgs::Jmp(BaseRArgs{
                    base_r: reg6to8
                })
            }),
            Opcode::JSR => Some({
                if((bits >> 11) & 0b1 != 0b1) {
                    panic!("Illegally encoded JSR instruction: IR[11:11] != 1")
                }
                InstructionArgs::Jsr(JsrArgs{
                    pcoffset11: off11
                })
            }),
            Opcode::JSRR => Some({
                if ((bits & 0x3f) != 0b0) {
                    panic!("Illegally encoded JSRR instruction: IR[0:5] != 0")
                }
                if ((bits >> 9) & 0b111 != 0b0) {
                    panic!("Illegally encoded JSRR instruction: IR[9:11] != 0")
                }
                InstructionArgs::Jsrr(BaseRArgs{
                    base_r: reg6to8
                })
            }),
            Opcode::LD => Some({
                InstructionArgs::Ld(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            }),
            Opcode::LDI => Some({
                InstructionArgs::Ldi(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            }),
            Opcode::LDR => Some({
                InstructionArgs::Ldr(DrBaseROff6Args{
                    dr: reg9to11,
                    base_r: reg6to8,
                    offset6: off6,
                })
            }),
            Opcode::LEA => Some({
                InstructionArgs::Lea(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            }),
            Opcode::NOT => Some({
                // TODO: illegally encoded instruction
                InstructionArgs::Not(OneSourceArithArgs{
                    dr: reg9to11,
                    sr: reg6to8,
                })
            }),
            Opcode::ST => Some({
                InstructionArgs::St(SrOff9Args{
                    sr: reg9to11,
                    offset9: off9,
                })
            }),
            Opcode::STI => Some({
                InstructionArgs::Sti(SrOff9Args{
                    sr: reg9to11,
                    offset9: off9,
                })
            }),
            Opcode::STR => Some({
                InstructionArgs::Str(SrBaseROff6Args{
                    sr: reg9to11,
                    base_r: reg6to8,
                    offset6: off6,
                })
            }),
            Opcode::TRAP => Some({
                InstructionArgs::Trap(TrapArgs{
                    trapvect8: trap8,
                })
            }),
            _ => panic!("unhandled opcode {:?}", opcode),
        };
        Self {
            opcode: opcode,
            args: args
        }
    }

    pub fn decode_ir(ir: &Register) -> Self {
        let _ = Opcode::from_ir(ir);
        Self::decode_bits(ir.content.0)
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
