//use core::marker::PhantomData;
use std::vec::Vec;
use core::ops::{Not, Add};

#[derive(Debug, Copy, Clone)]
pub enum RegisterName {
    R0, R1, R2, R3, R4, R5, R6, R7,
    PC, IR, MDR, MAR
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterContents(u16);

fn width_mask(lsb: usize, msb: usize) -> (usize, u16) {
    if lsb > msb {
        panic!("Programming error: least significant bit should always be less or equal to most significant bit")
    }

    let width = msb - lsb + 1;
    let mask = ((1 << width) - 1) << lsb;
    
    (width, mask)
}

fn mask_out(value: u16, lsb: usize, msb: usize) -> u16 {
    let (_, mask) = width_mask(lsb, msb);
    
    (value & mask) >> lsb
}

impl RegisterContents {
    fn init() -> Self {
        Self::new(0)
    }

    fn new(data: u16) -> Self {
        Self{ 0: data }
    }

    fn zext(self, lsb: usize, msb: usize) -> Self {

        // TODO get width and mask from a helper function
        let (_, mask) = width_mask(lsb, msb);
        Self::new(self.0 & mask)
    }

    fn sext(self, lsb: usize, msb: usize) -> Self {
        let (_, mask) = width_mask(msb + 1, 15);
        
        match (self.0 >> msb) & 0b1 {
            0b0 => {
                Self {
                    0: self.0 & !mask
                }
            },
            0b1 => {
                Self {
                    0: self.0 | mask
                }
            },
            _ => unreachable!(),
        }
        
    }
}

impl Add for RegisterContents {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            0: self.0 + other.0
        }
    }
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
            _ => panic!("unhandled case for register indexing, self={:?}", self)
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

#[derive(Debug, Copy, Clone)]
struct OpcodeAssumptionsViolation {
    msb: usize,
    lsb: usize,
    expected: RegisterContents,
    actual: RegisterContents,
}

impl OpcodeAssumptionsViolation {
    fn new(lsb: usize, msb: usize, expected: u16, actual: u16) -> Self {
        if lsb > msb {
            panic!("Programming error: least significant bit should always be less or equal to most significant bit")
        }

        let (_, mask) = width_mask(lsb, msb);

        Self {
            lsb: lsb,
            msb: msb,
            expected: RegisterContents::new(expected & mask),
            actual: RegisterContents::new(actual & mask),
        }
    }

//    fn check(&self) -> Option<Self> {
//        // Checks pass if this returns None
//        if self.expected == self.actual {
//            None
//        }
//
//        Some(self)
//    }
}

#[derive(Debug)]
enum Lc3Error {
    ProgrammingError(),
    IllegalOpcode(Vec<OpcodeAssumptionsViolation>),

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

    pub fn zeroed(id: RegisterName) -> Self {
        Self {
            content: RegisterContents::init(),
            id: id,
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

//trait SetCc{};
//trait PcOffset9{};
//trait PcOffset11{};

#[derive(Debug)]
struct GateFlag(bool);

impl Not for GateFlag {
    type Output = Self;

    fn not(self) -> Self {
        Self {
            0: !self.0
        }
    }
}

#[derive(Debug)]
struct LoadFlag(bool);

#[derive(Debug)]
struct OneBitMux(bool);

#[derive(Debug)]
struct TwoBitMux(u8);

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
pub enum Instruction {
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

impl Instruction {
    pub fn decode_bits(bits: u16) -> Self {
        /* The highest 4 bits of the instruction register,
         * IR[15:12], always contain the opcode.
         */
        let opcode = mask_out(bits, 12, 15);
        
        /* Legend -> instruction name, + if they set cc, 
         * (eg LD+ has instruction name LD, and
         * it sets the condition codes for jumps)
         *
         * This is followed by a colon, 
         * then all fields of instruction from 
         * highest bit to lowest of IR, separated by semicolon
         *
         * dr, sr1, sr2, sr, baser take 3 bits each
         * nzp all take 1 bit each
         * 
         * constants in the instruction, eg 0b1, are bits which are always set/cleared in a
         * valid instruction
         * 
         * other fields are labeled with the 
         * number of bits they take at the end, 
         * eg imm5 takes 5 bits
         */
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

        match opcode {
            /* opcode 0b0001: ADD/ADDi
             *
             * ADD+    : dr; sr1; 0b000; sr2
             * ADDi+   : dr; sr1; 0b1; imm5
             */
            0b0001 => { 
                match (bits >> 5) & 0b1 {
                    0b0 => {
                        if ((bits >> 3) & 0b11) != 0b0 {
                            panic!("Illegally encoded ADD instruction: IR[3:4] != 0")
                        }
                        Instruction::Add(TwoSourceArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        })
                    },
                    0b1 => {
                        Instruction::Addi(ImmedArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            imm5: imm5,
                        })
                    },
                    _ => unreachable!(),
                }
            },
            /* opcode 0b0101: AND/ANDi
             * AND+    : dr; sr1; 0b000; sr2
             * ANDi+   : dr; sr1; 0b1; imm5
             */
            0b0101 => {
                match mask_out(bits, 5, 5) {
                    0b0 => {
                        if mask_out(bits, 3, 4) != 0b0 {
                            panic!("Illegally encoded AND instruction: IR[3:4] != 0")
                        }
                        Instruction::And(TwoSourceArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        })
                    },
                    0b1 => {
                        Instruction::Andi(ImmedArithArgs{
                            dr: reg9to11,
                            sr1: reg6to8,
                            imm5: imm5,
                        })
                    },
                    _ => unreachable!(),
                }
            },
            /* opcode 0b0000: BR(branch) 
             * BR      : n; z; p; pcoffset9 */
            0b0000 => {
                Instruction::Br(BranchArgs{
                    n: n,
                    z: z,
                    p: p,
                    pcoffset9: off9
                })
            },
            /* opcode 0b1100: JMP
             * JMP     : 0b000; baser; 0b000000
             */
            0b1100 => {
                if mask_out(bits, 0, 5) != 0b0 {
                    panic!("Illegally encoded JMP instruction: IR[0:5] != 0")
                }
                if mask_out(bits, 9, 11) != 0b0 {
                    panic!("Illegally encoded JMP instruction: IR[9:11] != 0")
                }
                Instruction::Jmp(BaseRArgs{
                    base_r: reg6to8
                })
            },
            /* opcode 0b0100: JSR/JSRR
             * JSR      : 0b1; pcoffset11
             * JSRR     : 0b0; 0b00; baser; 0b000000
             */
            0b0100 => {
                match mask_out(bits, 11, 11) {
                    0b1 => {
                        Instruction::Jsr(JsrArgs{
                            pcoffset11: off11
                        })
                    },
                    0b0 => {
                        if mask_out(bits, 0, 5) != 0b0 {
                            panic!("Illegally encoded JSRR instruction: IR[0:5] != 0")
                        }
                        if mask_out(bits, 9, 10) != 0b0 {
                            panic!("Illegally encoded JSRR instruction: IR[9:11] != 0")
                        }
                        Instruction::Jsrr(BaseRArgs{
                            base_r: reg6to8
                        })
                    },
                    _ => unreachable!()
                }
            },
            /* opcode 0b0010: LD
             * LD+     : dr; pcoffset9
             */
            0b0010 => {
                Instruction::Ld(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            },
            /* opcode 0b1010: LDI
             * LDI+     : dr; pcoffset9
             */
            0b1010 => {
                Instruction::Ldi(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            },
            /* opcode 0b0110: LDR
             * LDR+     : dr; baser; offset6
             */
            0b0110 => {
                Instruction::Ldr(DrBaseROff6Args{
                    dr: reg9to11,
                    base_r: reg6to8,
                    offset6: off6,
                })
            },
            /* opcode 0b1110: LEA
             * LEA+    : dr; pcoffset9
             */
            0b1110 => {
                Instruction::Lea(DrOffsetArgs{
                    dr: reg9to11,
                    pcoffset9: off9,
                })
            },
            /* opcode 0b1001: NOT
             * NOT+    : dr; sr; 0b111111
             */
            0b1001 => {
                let (_, mask5) = width_mask(0, 5);
                if mask_out(bits, 0, 5) != mask5 {
                    panic!("Illegally encoded NOT instruction: IR[0:5] != 0b111111")
                }
                Instruction::Not(OneSourceArithArgs{
                    dr: reg9to11,
                    sr: reg6to8,
                })
            },
            /* opcode 0b0011: ST
             * ST      : sr; pcoffset9
             */
            0b0011 => {
                Instruction::St(SrOff9Args{
                    sr: reg9to11,
                    offset9: off9,
                })
            },
            /* opcode 0b1011: STI
             * STI     : sr; pcoffset9
             */
            0b1011 => {
                Instruction::Sti(SrOff9Args{
                    sr: reg9to11,
                    offset9: off9,
                })
            },
            /* opcode 0b0111: STR
             * STR     : sr; baser; offset6
             */
            0b0111 => {
                Instruction::Str(SrBaseROff6Args{
                    sr: reg9to11,
                    base_r: reg6to8,
                    offset6: off6,
                })
            },
            /* opcode 0b1111: TRAP
             * TRAP     : 0b0000; trapvect8
             */
            0b1111 => {
                if mask_out(bits, 8, 11) != 0b0 {
                    panic!("Illegally encoded TRAP instruction; IR[8:11] != 0")
                }
                Instruction::Trap(TrapArgs{
                    trapvect8: trap8,
                })
            },
            _ => panic!("unhandled opcode {:?}", opcode),
        }
    }

    pub fn decode_ir(ir: &Register) -> Self {
        match ir.id {
            RegisterName::IR => Self::decode_bits(ir.content.0),
            _ => panic!("Not allowed to build opcode from register ({:?}) that isn't IR", ir.id)
        }
    }

}

#[derive(Debug)]
pub struct Datapath {
    regfile: Regfile,
    
    ld_pc: LoadFlag,
    ld_ir: LoadFlag,
    ld_mar: LoadFlag,
    ld_reg: LoadFlag,
    ld_cc: LoadFlag,

    n: BranchFlag,
    z: BranchFlag,
    p: BranchFlag,

    // derive the bus value from gate signals
    gate_pc: GateFlag,
    gate_marmux: GateFlag,
    gate_alu: GateFlag,
 
    pc_mux: TwoBitMux,
    addr1_mux: OneBitMux,
    addr2_mux: TwoBitMux,
    sr2_mux: OneBitMux,
    mar_mux: OneBitMux,
    aluk: TwoBitMux,

    mar: Register,
    ir: Register,
    mdr: Register,
    pc: Register,

}

impl Datapath {
    fn new(starting_pc: RegisterContents) -> Self {
        Self {
            regfile: Regfile::new(),
            
            ld_pc: LoadFlag(false),
            ld_ir: LoadFlag(false),
            ld_mar: LoadFlag(false),
            ld_reg: LoadFlag(false),
            ld_cc: LoadFlag(false),

            n: BranchFlag(false),
            z: BranchFlag(false),
            p: BranchFlag(false),

            gate_pc: GateFlag(false),
            gate_marmux: GateFlag(false),
            gate_alu: GateFlag(false),

            addr1_mux: OneBitMux(false),
            addr2_mux: TwoBitMux(5),
            mar_mux: OneBitMux(false),
            sr2_mux: OneBitMux(false),
            aluk: TwoBitMux(5),
            pc_mux: TwoBitMux(5),

            mar: Register::zeroed(RegisterName::MAR),
            ir: Register::zeroed(RegisterName::IR),
            mdr: Register::zeroed(RegisterName::MDR),
            pc: Register::new(starting_pc, RegisterName::PC),
        }
        
    }

    fn mux_addr1(&self, instruction: &Instruction) -> RegisterContents {
        match self.addr1_mux.0 {
            false => {
                // Should be regfile.contents_of[sr1]
                panic!("incorrect implementation for addr1MUX")
                //RegisterContents::init()
            },
            true => {
                self.pc.content
            }
        }
    }

    fn mux_addr2(&self, instruction: &Instruction) -> RegisterContents {
        match self.addr2_mux.0 {
            0 => {self.ir.content.sext(0, 10)},
            1 => {self.ir.content.sext(0, 8)},
            2 => {self.ir.content.sext(0, 5)},
            3 => {RegisterContents::init()},
            _ => panic!("Invalid value for ADDR2MUX: {:?}", self.addr2_mux)
        }
    }

    fn bus(&self, instruction: &Instruction) -> RegisterContents {
        if self.gate_pc.0 && !self.gate_marmux.0 && !self.gate_alu.0 {
            return self.pc.content;
        }
        if !self.gate_pc.0 && self.gate_marmux.0 && !self.gate_alu.0 {
            match self.mar_mux.0 {
                false => {
                    return self.ir.content.zext(0, 7);
                },
                true => {
                    return self.mux_addr2(instruction) + self.mux_addr1(instruction);
                },
            }
        }
        panic!("unimplemented")
    }
}

struct Lrc3State18;
struct Lrc3State19;

struct Lrc3CpuState {
    datapath: Datapath,
    memory: Memory,
}

impl Lrc3CpuState {
    fn new(starting_pc: RegisterContents) -> Self {
        Self {
            datapath: Datapath::new(starting_pc),
            memory: Memory::new()
        }
    }
}

enum Lrc3State {
    S18,
    S19,
}

trait Lrc3Transition{
    fn transition(self, state: &mut Lrc3CpuState) -> Lrc3State;  
}

impl Lrc3Transition for Lrc3State18 {
    fn transition(self, _: &mut Lrc3CpuState) -> Lrc3State {
        Lrc3State::S19
    }
}

impl Lrc3Transition for Lrc3State19 {
    fn transition(self, _: &mut Lrc3CpuState) -> Lrc3State {
        Lrc3State::S18
    }
}

struct Lrc3Cpu {
    state: Lrc3State,
    data: Lrc3CpuState,
}

impl Lrc3Cpu {
    pub fn new() -> Self {
        Self {
            state: Lrc3State::S18,
            data: Lrc3CpuState::new(RegisterContents::new(0x3000)),
        }
    }
}

