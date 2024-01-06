mod lrc3;
//#use lrc3::*;

fn main() {
    let mut rf = lrc3::Regfile::new();

    println!("{:?}", rf);

//    println!("instruction encoded is {:?}", lrc3::Opcode::from_ir_bits(0b1111_111111111111));

    println!("instruction from bits: {:?}", lrc3::Instruction::decode_ir(&lrc3::Register::ir_from_bits(0b0110_011_011_000000)));
}
