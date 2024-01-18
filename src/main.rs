mod lrc3;
//#use lrc3::*;

fn main() {
    let rf = lrc3::Regfile::new();

    println!("{:?}", rf);

    //    println!("instruction encoded is {:?}", lrc3::Opcode::from_ir_bits(0b1111_111111111111));

    if let Err(e) = lrc3::Instruction::decode_bits(0b0001_011_101_0_01_001) {
        println!("broken instruction: {}", e);
    }
    println!(
        "instruction from bits: {:?}",
        lrc3::Instruction::decode_ir(&lrc3::Register::ir_from_bits(0b0001_011_101_0_00_001))
    );
    println!(
        "instruction from bits: {:?}",
        lrc3::Instruction::decode_ir(&lrc3::Register::ir_from_bits(0b0001_011_101_1_01001))
    );

    for bits in u16::MIN..=u16::MAX {
        match lrc3::Instruction::decode_bits(bits) {
            Ok(ins) => {
                println!("{:016b}: {}", bits, ins);
            }
            _ => {}
        }
    }
}
// Templates:
//impl Display for TwoSourceArithArgs {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        write!()
//    }
//}
