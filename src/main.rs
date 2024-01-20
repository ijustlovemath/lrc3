mod lrc3;
//#use lrc3::*;

fn main() {
    for bits in u16::MIN..=u16::MAX {
        match lrc3::Instruction::decode_bits(bits) {
            Ok(ins) => {
                println!("{:016b}: {}", bits, ins);
            }
            _ => {}
        }
    }

    println!("{}", lrc3::Imm5::new(0x1f));
    println!("{:016b}", lrc3::sext16(0b111,3)); 
}
// Templates:
//impl Display for TwoSourceArithArgs {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        write!()
//    }
//}
