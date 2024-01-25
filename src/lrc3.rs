//use core::marker::PhantomData;
//use std::vec::Vec;
use core::fmt::{Display, Error, Formatter};
use core::ops::{Add, BitAnd, Not};

#[derive(Debug, Copy, Clone)]
pub enum RegisterName {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    IR,
    MDR,
    MAR,
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

pub fn mask_out(value: u16, lsb: usize, msb: usize) -> u16 {
    let (_, mask) = width_mask(lsb, msb);

    (value & mask) >> lsb
}

pub fn zext16(bits: u16, lsb: usize, msb: usize) -> u16 {
    let (_, mask) = width_mask(lsb, msb);

    // Keep all the bits from lsb to msb as they are
    bits & mask
}

/// # Sign extension starting from msb, 0-indexed
pub fn sext16(bits: u16, msb: usize) -> u16 {
    let (_, mask) = width_mask(msb + 1, 15);

    if msb == 0 {
        panic!("don't know how to sign extend from 0th bit")
    }
    
    /*
     * sext16(0b011, 2) -> right shift 2 = 0b0 is the sign bit
     * add in a bunch of 0s from bit 3 (msb+1) to the end
     */
    match (bits >> msb) & 0b1 {
        0b0 => { bits & !mask }
        0b1 => { bits | mask}
        _ => unreachable!()
    }
}

#[test]
fn sext16test() {
    assert_eq!(sext16(0b111, 2), 0xffff);
    assert_eq!(sext16(0b111, 1), 0xffff);
    assert_eq!(sext16(0b111, 3), 0x7);
    assert_eq!(sext16(0b011, 2), 0b11);
    assert_eq!(sext16(0b100101101, 8), 0b1111_1111_0010_1101);
    assert_eq!(sext16(0b111, 2) as i16, -1);
    assert_eq!(sext16(0b0111, 3) as i16, 7);

}

impl RegisterContents {
    fn init() -> Self {
        Self::new(0)
    }

    fn new(data: u16) -> Self {
        Self { 0: data }
    }

    fn zext(self, lsb: usize, msb: usize) -> Self {
        Self::new(zext16(self.0, lsb, msb))
    }

    fn sext(self, msb: usize) -> Self {
        Self { 0: sext16(self.0, msb) }
    }
}

impl Add for RegisterContents {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            0: self.0 + other.0,
        }
    }
}

impl BitAnd<u16> for RegisterContents {
    type Output = u16;

    fn bitand(self, other: u16) -> Self::Output {
        other & self.0
    }
}

impl Display for RegisterContents {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Reg[{:04x}]", self.0)
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
            _ => panic!("unhandled case for register indexing, self={:?}", self),
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
pub struct OpcodeAssumptionsViolation {
    msb: usize,
    lsb: usize,
    width: usize,
    mask: u16,
    expected: RegisterContents,
    actual: RegisterContents,
    opcode: &'static str,
}

impl OpcodeAssumptionsViolation {
    fn new(lsb: usize, msb: usize, expected: u16, actual: u16, opcode_name: &'static str) -> Self {
        if lsb > msb {
            panic!("Programming error: least significant bit should always be less or equal to most significant bit")
        }

        let (width, mask) = width_mask(lsb, msb);

        Self {
            lsb: lsb,
            msb: msb,
            width: width,
            mask: mask,
            expected: RegisterContents::new(expected),
            actual: RegisterContents::new((actual & mask) >> lsb),
            opcode: opcode_name,
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

impl Display for OpcodeAssumptionsViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Illegally encoded {} instruction: IR[{}:{}] should be {}, but is {}",
            self.opcode,
            self.msb,
            self.lsb,
            self.expected,
            (self.actual & self.mask) >> self.lsb
        )
    }
}

#[derive(Debug)]
pub struct UnknownOpcodeArgs {
    opcode: u16,
    bits: u16,
}

#[derive(Debug)]
pub enum Lrc3Error {
    ProgrammingError(),
    IllegalOpcode(OpcodeAssumptionsViolation),
    UnknownOpcode(UnknownOpcodeArgs),
}

impl Display for Lrc3Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::ProgrammingError() => {
                write!(
                    f,
                    "LRC3 Error: A mistake was made in this implementation of the LRC3"
                )
            }
            Self::IllegalOpcode(o) => {
                write!(f, "LRC3 Error: {}", o)
            }
            Self::UnknownOpcode(o) => {
                write!(f, "LRC3 Error: {:?}", o)
            }
        }
    }
}

#[derive(Debug)]
pub struct Register {
    content: RegisterContents,
    id: RegisterName,
}

struct Memory {
    memory: [RegisterContents; 65536],
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [RegisterContents::init(); 65536];
        memory[0x3000] = RegisterContents::new(0xfe00);
        Self { memory: memory }
    }
}

impl Register {
    pub fn new(content: RegisterContents, id: RegisterName) -> Self {
        Self { content, id }
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
    registers: [Register; 8],
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
            ],
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
        Self { 0: !self.0 }
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

impl PcOffset9 {
    pub fn new(bits: u16) -> Self {
        Self{ 0: sext16(bits & 0x1ff, 8) }
    }

    pub fn masked(&self) -> u16 {
        self.0 & 0x1ff
    }
}

impl Display for PcOffset9 {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#{}", self.0 as i16)
    }
}

#[derive(Debug)]
struct PcOffset11(u16);

impl PcOffset11 {
    pub fn new(bits: u16) -> Self {
        Self{ 0: sext16(bits & 0x7ff, 10) }
    }

    pub fn masked(&self) -> u16 {
        self.0 & 0x7ff
    }
}

impl Display for PcOffset11 {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#{}", self.0 as i16)
    }
}

#[derive(Debug)]
pub struct Imm5(u16);

impl Imm5 {
    pub fn new(bits: u16) -> Self {
        Self{ 0: sext16(bits & 0x1f, 4) }
    }
}

impl Display for Imm5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#{}", self.0 as i16)
    }
}

#[test]
fn test_imm5_display() {
    assert_eq!(format!("{}", Imm5::new(0b111_111)), "#-1")
}

#[derive(Debug)]
struct Offset6(u16);

impl Offset6 {
    pub fn new(bits: u16) -> Self {
        Self{ 0: sext16(bits & 0x3f, 5) }
    }

    pub fn masked(&self) -> u16 {
        self.0 & 0x3f
    }
}

impl Display for Offset6 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
       write!(f, "#{}", self.0 as i16)
    }
}

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

impl Display for TwoSourceArithArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}, {:?} -> {:?}", self.sr1, self.sr2, self.dr)
    }
}

#[derive(Debug)]
struct OneSourceArithArgs {
    dr: RegisterName,
    sr: RegisterName,
}

impl Display for OneSourceArithArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?} -> {:?}", self.sr, self.dr)
    }
}

#[derive(Debug)]
struct ImmedArithArgs {
    dr: RegisterName,
    sr1: RegisterName,
    imm5: Imm5,
}

impl Display for ImmedArithArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}, {} -> {:?}", self.sr1, self.imm5, self.dr)
    }
}

#[derive(Debug)]
struct BranchArgs {
    n: BranchFlag,
    z: BranchFlag,
    p: BranchFlag,
    pcoffset9: PcOffset9,
}

impl Display for BranchArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}{}{} {}",
            if self.n.0 { "n" } else { "" },
            if self.z.0 { "z" } else { "" },
            if self.p.0 { "p" } else { "" },
            self.pcoffset9
        )
    }
}

#[derive(Debug)]
struct BaseRArgs {
    base_r: RegisterName,
}

impl Display for BaseRArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?} ; {:?} -> PC", self.base_r, self.base_r)
    }
}

#[derive(Debug)]
struct TrapArgs {
    trapvect8: TrapVect,
}

impl Display for TrapArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "0x{:02x}", self.trapvect8.0)
    }
}

#[derive(Debug)]
struct JsrArgs {
    pcoffset11: PcOffset11,
}

impl Display for JsrArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{} ; PC + SEXT(0b{:011b}) -> PC",
            self.pcoffset11, self.pcoffset11.masked()
        )
    }
}

#[derive(Debug)]
struct LdArgs {
    dr: RegisterName,
    pcoffset9: PcOffset9,
}

impl Display for LdArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "MEM[PC + {}] -> {:?}",
            self.pcoffset9, self.dr
        )
    }
}

#[derive(Debug)]
struct LdiArgs {
    dr: RegisterName,
    pcoffset9: PcOffset9,
}

impl Display for LdiArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "MEM[MEM[PC + SEXT(0b{:09b})]] -> {:?}",
            self.pcoffset9.0, self.dr
        )
    }
}

#[derive(Debug)]
struct LeaArgs {
    dr: RegisterName,
    pcoffset9: PcOffset9,
}

impl Display for LeaArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}, {} ; PC + SEXT(0b{:09b}) -> {:?}",
            self.dr, self.pcoffset9, self.pcoffset9.0, self.dr
        )
    }
}

#[derive(Debug)]
struct LdrArgs {
    dr: RegisterName,
    base_r: RegisterName,
    offset6: Offset6,
}

impl Display for LdrArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}, {:?}, {} ; MEM[{:?} + SEXT(0b{:06b})] -> {:?}",
            self.dr,
            self.base_r,
            self.offset6,
            self.base_r,
            self.offset6.masked(),
            self.dr
        )
    }
}

#[derive(Debug)]
struct StrArgs {
    sr: RegisterName,
    base_r: RegisterName,
    offset6: Offset6,
}

impl Display for StrArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}, {:?}, {} ; {:?} -> MEM[{:?} + SEXT(0b{:06b})]",
            self.sr, self.base_r, self.offset6, self.sr, self.base_r, self.offset6.masked()
        )
    }
}

#[derive(Debug)]
struct StArgs {
    sr: RegisterName,
    offset9: PcOffset9,
}

impl Display for StArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}, {} ; {:?} -> MEM[PC + SEXT(0b{:09b})]",
            self.sr, self.offset9, self.sr, self.offset9.masked(),
        )
    }
}

#[derive(Debug)]
struct StiArgs {
    sr: RegisterName,
    offset9: PcOffset9,
}

impl Display for StiArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}, {} ; {:?} -> MEM[MEM[PC + SEXT(0b{:09b})]]",
            self.sr, self.offset9, self.sr, self.offset9.masked(),
        )
    }
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

    Ld(LdArgs),
    Ldi(LdiArgs),
    Ldr(LdrArgs),

    Lea(LeaArgs),

    Not(OneSourceArithArgs),
    Rti(),

    St(StArgs),
    Sti(StiArgs),
    Str(StrArgs),
    Trap(TrapArgs),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Add(args) => {
                write!(f, "ADD {}", args)
            }
            Self::Addi(args) => {
                write!(f, "ADDi {}", args)
            }
            Self::And(args) => {
                write!(f, "AND {}", args)
            }
            Self::Andi(args) => {
                write!(f, "ANDi {}", args)
            }
            Self::Br(args) => {
                write!(f, "BR{}", args)
            }
            Self::Not(args) => {
                write!(f, "NOT {}", args)
            }
            Self::Jmp(args) => {
                write!(f, "JMP {}", args)
            }
            Self::Trap(args) => {
                write!(f, "TRAP {}", args)
            }
            Self::Jsr(args) => {
                write!(f, "JSR {}", args)
            }
            Self::Jsrr(_args) => {
                write!(f, "JSRR UNIMPLEMENTED")
            }
            Self::Ld(args) => {
                write!(f, "LD {}", args)
            }
            Self::Ldi(args) => {
                write!(f, "LDi {}", args)
            }
            Self::Ldr(args) => {
                write!(f, "LDR {}", args)
            }
            Self::Lea(args) => {
                write!(f, "LEA {}", args)
            }
            Self::Str(args) => {
                write!(f, "STR {}", args)
            }
            Self::St(args) => {
                write!(f, "ST {}", args)
            }
            Self::Sti(args) => {
                write!(f, "STI {}", args)
            }
            Self::Rti() => {
                write!(f, "RTI")
            }
        }
    }
}

impl Instruction {
    pub fn decode_bits(bits: u16) -> Result<Self, Lrc3Error> {
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
        let imm5 = Imm5::new(bits);
        let off6 = Offset6::new(bits);
        let off9 = PcOffset9::new(bits);
        let off11 = PcOffset11::new(bits);
        let trap8 = TrapVect(bits & 0xff);

        let reg9to11 = RegisterName::from_bits(arg9to11);
        let reg6to8 = RegisterName::from_bits(arg6to8);
        let reg0to2 = RegisterName::from_bits(bits);

        let n = BranchFlag((bits >> 11) & 0b1 == 0b1);
        let z = BranchFlag((bits >> 10) & 0b1 == 0b1);
        let p = BranchFlag((bits >> 9) & 0b1 == 0b1);

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
                            return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                                3, 4, 0b0, bits, "ADD",
                            )));
                            // panic!("Illegally encoded ADD instruction: IR[3:4] != 0")
                        }
                        Ok(Instruction::Add(TwoSourceArithArgs {
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        }))
                    }
                    0b1 => Ok(Instruction::Addi(ImmedArithArgs {
                        dr: reg9to11,
                        sr1: reg6to8,
                        imm5: imm5,
                    })),
                    _ => unreachable!(),
                }
            }
            /* opcode 0b0101: AND/ANDi
             * AND+    : dr; sr1; 0b000; sr2
             * ANDi+   : dr; sr1; 0b1; imm5
             */
            0b0101 => {
                match mask_out(bits, 5, 5) {
                    0b0 => {
                        if mask_out(bits, 3, 4) != 0b0 {
                            return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                                3, 4, 0b0, bits, "AND",
                            )));
                            // panic!("Illegally encoded AND instruction: IR[3:4] != 0")
                        }
                        Ok(Instruction::And(TwoSourceArithArgs {
                            dr: reg9to11,
                            sr1: reg6to8,
                            sr2: reg0to2,
                        }))
                    }
                    0b1 => Ok(Instruction::Andi(ImmedArithArgs {
                        dr: reg9to11,
                        sr1: reg6to8,
                        imm5: imm5,
                    })),
                    _ => unreachable!(),
                }
            }
            /* opcode 0b0000: BR(branch)
             * BR      : n; z; p; pcoffset9 */
            0b0000 => Ok(Instruction::Br(BranchArgs {
                n: n,
                z: z,
                p: p,
                pcoffset9: off9,
            })),
            /* opcode 0b1100: JMP
             * JMP     : 0b000; baser; 0b000000
             */
            0b1100 => {
                if mask_out(bits, 0, 5) != 0b0 {
                    return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                        0, 5, 0b0, bits, "JMP",
                    )));
                }
                if mask_out(bits, 9, 11) != 0b0 {
                    return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                        9, 11, 0b0, bits, "JMP",
                    )));
                }
                Ok(Instruction::Jmp(BaseRArgs { base_r: reg6to8 }))
            }
            /* opcode 0b0100: JSR/JSRR
             * JSR      : 0b1; pcoffset11
             * JSRR     : 0b0; 0b00; baser; 0b000000
             */
            0b0100 => match mask_out(bits, 11, 11) {
                0b1 => Ok(Instruction::Jsr(JsrArgs { pcoffset11: off11 })),
                0b0 => {
                    if mask_out(bits, 0, 5) != 0b0 {
                        return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                            0, 5, 0b0, bits, "JSRR",
                        )));
                    }
                    if mask_out(bits, 9, 10) != 0b0 {
                        return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                            9, 10, 0b0, bits, "JSRR",
                        )));
                    }
                    Ok(Instruction::Jsrr(BaseRArgs { base_r: reg6to8 }))
                }
                _ => unreachable!(),
            },
            /* opcode 0b0010: LD
             * LD+     : dr; pcoffset9
             */
            0b0010 => Ok(Instruction::Ld(LdArgs {
                dr: reg9to11,
                pcoffset9: off9,
            })),
            /* opcode 0b1010: LDI
             * LDI+     : dr; pcoffset9
             */
            0b1010 => Ok(Instruction::Ldi(LdiArgs {
                dr: reg9to11,
                pcoffset9: off9,
            })),
            /* opcode 0b0110: LDR
             * LDR+     : dr; baser; offset6
             */
            0b0110 => Ok(Instruction::Ldr(LdrArgs {
                dr: reg9to11,
                base_r: reg6to8,
                offset6: off6,
            })),
            /* opcode 0b1110: LEA
             * LEA+    : dr; pcoffset9
             */
            0b1110 => Ok(Instruction::Lea(LeaArgs {
                dr: reg9to11,
                pcoffset9: off9,
            })),
            /* opcode 0b1001: NOT
             * NOT+    : dr; sr; 0b111111
             */
            0b1001 => {
                let (_, mask5) = width_mask(0, 5);
                if mask_out(bits, 0, 5) != mask5 {
                    return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                        0, 5, 0b111111, bits, "NOT",
                    )));
                }
                Ok(Instruction::Not(OneSourceArithArgs {
                    dr: reg9to11,
                    sr: reg6to8,
                }))
            }
            /* opcode 0b0011: ST
             * ST      : sr; pcoffset9
             */
            0b0011 => Ok(Instruction::St(StArgs {
                sr: reg9to11,
                offset9: off9,
            })),
            /* opcode 0b1011: STI
             * STI     : sr; pcoffset9
             */
            0b1011 => Ok(Instruction::Sti(StiArgs {
                sr: reg9to11,
                offset9: off9,
            })),
            /* opcode 0b0111: STR
             * STR     : sr; baser; offset6
             */
            0b0111 => Ok(Instruction::Str(StrArgs {
                sr: reg9to11,
                base_r: reg6to8,
                offset6: off6,
            })),
            /* opcode 0b1111: TRAP
             * TRAP     : 0b0000; trapvect8
             */
            0b1111 => {
                if mask_out(bits, 8, 11) != 0b0 {
                    return Err(Lrc3Error::IllegalOpcode(OpcodeAssumptionsViolation::new(
                        8, 11, 0b0, bits, "JSRR",
                    )));
                }
                Ok(Instruction::Trap(TrapArgs { trapvect8: trap8 }))
            }
            _ => Err(Lrc3Error::UnknownOpcode(UnknownOpcodeArgs {
                opcode: opcode,
                bits: bits,
            })),
        }
    }

    pub fn decode_ir(ir: &Register) -> Self {
        match ir.id {
            RegisterName::IR => Self::decode_bits(ir.content.0).expect("unhandled decode ir"),
            _ => panic!(
                "Not allowed to build opcode from register ({:?}) that isn't IR",
                ir.id
            ),
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

    fn mux_addr1(&self, _instruction: &Instruction) -> RegisterContents {
        match self.addr1_mux.0 {
            false => {
                // Should be regfile.contents_of[sr1]
                panic!("incorrect implementation for addr1MUX")
                //RegisterContents::init()
            }
            true => self.pc.content,
        }
    }

    fn mux_addr2(&self, _instruction: &Instruction) -> RegisterContents {
        match self.addr2_mux.0 {
            0 => self.ir.content.sext(10),
            1 => self.ir.content.sext(8),
            2 => self.ir.content.sext(5),
            3 => RegisterContents::init(),
            _ => panic!("Invalid value for ADDR2MUX: {:?}", self.addr2_mux),
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
                }
                true => {
                    return self.mux_addr2(instruction) + self.mux_addr1(instruction);
                }
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
            memory: Memory::new(),
        }
    }
}

enum Lrc3State {
    S0_Branch,
    S18_Fetch_LdMar,
    S19_Fetch_IncPc,

}

trait Lrc3Transition {
    fn transition(self, state: &mut Lrc3CpuState) -> Lrc3State;
}

impl Lrc3Transition for Lrc3State18 {
    fn transition(self, state: &mut Lrc3CpuState) -> Lrc3State {
        
        Lrc3State::S19_Fetch_IncPc
    }
}

impl Lrc3Transition for Lrc3State19 {
    fn transition(self, _: &mut Lrc3CpuState) -> Lrc3State {
        Lrc3State::S18_Fetch_LdMar
    }
}

struct Lrc3Cpu {
    state: Lrc3State,
    data: Lrc3CpuState,
}

impl Lrc3Cpu {
    pub fn new() -> Self {
        Self {
            state: Lrc3State::S18_Fetch_LdMar,
            data: Lrc3CpuState::new(RegisterContents::new(0x3000)),
        }
    }
}
