use crate::parse::{ConstType, Expr, InstType, Label};
use anyhow::{anyhow, Context, Result};

pub fn gen(exprs: Vec<Expr>, label_table: Vec<Label>) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    'outer: for expr in exprs {
        match expr {
            Expr::Inst {
                inst_type,
                mnemonic,
                rd,
                rs,
                imm,
                symbol,
                address,
            } => match inst_type {
                InstType::R => {
                    let func: u16 = match mnemonic.as_str() {
                        "mov" => 0b00001,
                        "add" => 0b00010,
                        "sub" => 0b00011,
                        "and" => 0b00100,
                        "or" => 0b00101,
                        "xor" => 0b00110,
                        "sll" => 0b00111,
                        "srl" => 0b01000,
                        "sra" => 0b01001,
                        _ => unreachable!(),
                    };
                    let rd = gen_reg(rd)?;
                    let rs = gen_reg(rs)?;
                    let word: u16 = (rd & 0x0007) << 5 | (rs & 0x0007) << 8 | (func & 0x001F) << 11;
                    bytes.push((word & 0x00FF) as u8);
                    bytes.push(((word & 0xFF00) >> 8) as u8);
                }

                InstType::I5 => {
                    let opcode: u16 = match mnemonic.as_str() {
                        "addi" => 0b00001,
                        "subi" => 0b00010,
                        "beq" => 0b00011,
                        "bnq" => 0b00100,
                        "blt" => 0b00101,
                        "bge" => 0b00110,
                        "bltu" => 0b00111,
                        "bgeu" => 0b01000,
                        "jalr" => 0b01001,
                        "lw" => 0b01010,
                        "sw" => 0b01011,
                        _ => unreachable!(),
                    };
                    let rd = gen_reg(rd)?;
                    let rs = gen_reg(rs)?;
                    if mnemonic.as_str() == "addi" || mnemonic.as_str() == "subi" {
                        let imm = imm
                            .parse::<u8>()
                            .with_context(|| format!("could not parse {}", imm))?;
                        if !(0..=31).contains(&imm) {
                            return Err(anyhow!("invalid immediate {}", imm));
                        }
                        let word: u16 = opcode & 0x001F
                            | (rd & 0x0007) << 5
                            | (rs & 0x0007) << 8
                            | (imm as u16 & 0x001F) << 11;
                        bytes.push((word & 0x00FF) as u8);
                        bytes.push(((word & 0xFF00) >> 8) as u8);
                    } else {
                        let imm = imm
                            .parse::<i8>()
                            .with_context(|| format!("could not parse {}", imm))?;
                        if !(-16..=15).contains(&imm) {
                            return Err(anyhow!("invalid immediate {}", imm));
                        }
                        let word: u16 = opcode & 0x001F
                            | (rd & 0x0007) << 5
                            | (rs & 0x0007) << 8
                            | (imm as u16 & 0x001F) << 11;
                        bytes.push((word & 0x00FF) as u8);
                        bytes.push(((word & 0xFF00) >> 8) as u8);
                    }
                }
                InstType::I8 => {
                    let opcode: u16 = match mnemonic.as_str() {
                        "jal" => 0b01100,
                        "lil" => 0b01101,
                        "lih" => 0b01110,
                        _ => unreachable!(),
                    };
                    let rd = gen_reg(rd)?;

                    if mnemonic.as_str() == "jal" {
                        for label in &label_table {
                            if imm == label.name {
                                let imm = label.address as i32 - address as i32;
                                if !(-128..=127).contains(&imm) {
                                    return Err(anyhow!("could not jump to {}", label.name));
                                }
                                let word: u16 = opcode & 0x001F
                                    | (rd & 0x0007) << 5
                                    | (imm as u16 & 0x00FF) << 8;
                                bytes.push((word & 0x00FF) as u8);
                                bytes.push(((word & 0xFF00) >> 8) as u8);

                                continue 'outer;
                            }
                        }
                        let imm = imm
                            .parse::<i8>()
                            .with_context(|| format!("could not parse {}", imm))?;
                        let word: u16 =
                            opcode & 0x001F | (rd & 0x0007) << 5 | (imm as u16 & 0x00FF) << 8;
                        bytes.push((word & 0x00FF) as u8);
                        bytes.push(((word & 0xFF00) >> 8) as u8);
                    } else if imm.starts_with("0x") {
                        let imm = u16::from_str_radix(imm.trim_start_matches("0x"), 16)
                            .with_context(|| format!("could not parse {}", imm))?;
                        if symbol.as_str() == "l" {
                            let word: u16 =
                                opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0x00FF) << 8;
                            bytes.push((word & 0x00FF) as u8);
                            bytes.push(((word & 0xFF00) >> 8) as u8);
                            continue 'outer;
                        } else if symbol.as_str() == "h" {
                            let word: u16 = opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0xFF00);
                            bytes.push((word & 0x00FF) as u8);
                            bytes.push(((word & 0xFF00) >> 8) as u8);
                            continue 'outer;
                        } else {
                            return Err(anyhow!("Unknown symbol {}", symbol));
                        }
                    } else {
                        for label in &label_table {
                            if imm == label.name {
                                let imm = label.address;
                                if symbol.as_str() == "l" {
                                    let word: u16 =
                                        opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0x00FF) << 8;
                                    bytes.push((word & 0x00FF) as u8);
                                    bytes.push(((word & 0xFF00) >> 8) as u8);
                                    continue 'outer;
                                } else if symbol.as_str() == "h" {
                                    let word: u16 =
                                        opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0xFF00);
                                    bytes.push((word & 0x00FF) as u8);
                                    bytes.push(((word & 0xFF00) >> 8) as u8);
                                    continue 'outer;
                                } else {
                                    return Err(anyhow!("Unknown symbol {}", symbol));
                                }
                            }
                        }
                        return Err(anyhow!("Unknown label {}", imm));
                    }
                }
                InstType::C1 => {
                    let func: u16 = match mnemonic.as_str() {
                        "push" => 0b00001,
                        "pop" => 0b00010,
                        "rpc" => 0b00011,
                        "rsp" => 0b00100,
                        "rpsr" => 0b00101,
                        "rtlr" => 0b00110,
                        "rthr" => 0b00111,
                        "rppc" => 0b01000,
                        "rppsr" => 0b01001,
                        "wsp" => 0b01010,
                        "wpsr" => 0b01011,
                        "wtlr" => 0b01100,
                        "wthr" => 0b01101,
                        "wppc" => 0b01110,
                        "wppsr" => 0b01111,
                        _ => unreachable!(),
                    };
                    let rd = gen_reg(rd)?;
                    let word: u16 = 0x001E | (rd & 0x0007) << 5 | (func & 0x001F) << 11;
                    bytes.push((word & 0x00FF) as u8);
                    bytes.push(((word & 0xFF00) >> 8) as u8);
                }
                InstType::C2 => {
                    let func: u16 = match mnemonic.as_str() {
                        "rfi" => 0b00001,
                        "rtr" => 0b00010,
                        "wtr" => 0b00011,
                        _ => unreachable!(),
                    };
                    let word: u16 = 0x001F | (func & 0x001F) << 11;
                    bytes.push((word & 0x00FF) as u8);
                    bytes.push(((word & 0xFF00) >> 8) as u8);
                }
                InstType::Trap => {
                    let word: u16 = 0xFFFF;
                    bytes.push((word & 0x00FF) as u8);
                    bytes.push(((word & 0xFF00) >> 8) as u8);
                }
            },
            Expr::Const {
                val, const_type, ..
            } => {
                if const_type == ConstType::Word {
                    let word: u16 = u16::from_str_radix(&val, 16)
                        .with_context(|| format!("could not parse 0x{}", val))?;
                    bytes.push((word & 0x00FF) as u8);
                    bytes.push(((word & 0xFF00) >> 8) as u8);
                } else {
                    let byte: u8 = u8::from_str_radix(&val, 16)
                        .with_context(|| format!("could not parse 0x{}", val))?;
                    bytes.push(byte);
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(bytes)
}

fn gen_reg(reg: String) -> Result<u16> {
    match reg.as_str() {
        "x0" | "zero" => Ok(0b000),
        "x1" | "ra" => Ok(0b001),
        "x2" | "fp" => Ok(0b010),
        "x3" | "a0" => Ok(0b011),
        "x4" | "a1" => Ok(0b100),
        "x5" | "a2" => Ok(0b101),
        "x6" | "t0" => Ok(0b110),
        "x7" | "t1" => Ok(0b111),
        _ => Err(anyhow!("unknown register {}", reg)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::parse;
    use std::io::{BufReader, Read};

    #[test]
    fn can_gen_r_inst() -> Result<()> {
        let text = load_test_asm("test/asm/r_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b0000_0000,
            0b0000_1000,
            0b0010_0000,
            0b0001_0001,
            0b0100_0000,
            0b0001_1010,
            0b0110_0000,
            0b0010_0011,
            0b1000_0000,
            0b0010_1100,
            0b1010_0000,
            0b0011_0101,
            0b1100_0000,
            0b0011_1110,
            0b1110_0000,
            0b0100_0111,
            0b0000_0000,
            0b0100_1001,
        ];

        assert_eq!(result_bytes, expect_bytes);

        Ok(())
    }

    #[test]
    fn can_gen_i5_inst() -> Result<()> {
        let text = load_test_asm("test/asm/i5_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b0000_0001,
            0b0000_1000,
            0b0010_0010,
            0b0000_1001,
            0b0100_0011,
            0b0000_1010,
            0b0110_0100,
            0b1111_1011,
            0b1000_0101,
            0b0000_1100,
            0b1010_0110,
            0b1111_1101,
            0b1100_0111,
            0b0000_1110,
            0b1110_1000,
            0b1111_1111,
            0b0000_1001,
            0b0000_1001,
            0b0100_1010,
            0b1111_1011,
            0b1000_1011,
            0b0000_1101,
        ];

        assert_eq!(result_bytes, expect_bytes);

        Ok(())
    }

    #[test]
    fn can_gen_i8_inst() -> Result<()> {
        let text = load_test_asm("test/asm/i8_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b0000_1100,
            0b0000_0001,
            0b0010_1101,
            0b0000_0001,
            0b0100_1110,
            0b0000_0000,
        ];

        assert_eq!(result_bytes, expect_bytes);
        Ok(())
    }

    #[test]
    fn can_gen_c1_inst() -> Result<()> {
        let text = load_test_asm("test/asm/c1_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b0001_1110,
            0b0000_1000,
            0b0011_1110,
            0b0001_0000,
            0b0101_1110,
            0b0001_1000,
            0b0111_1110,
            0b0010_0000,
            0b1001_1110,
            0b0010_1000,
            0b1011_1110,
            0b0011_0000,
            0b1101_1110,
            0b0011_1000,
            0b1111_1110,
            0b0100_0000,
            0b0001_1110,
            0b0100_1000,
            0b0011_1110,
            0b0101_0000,
            0b0101_1110,
            0b0101_1000,
            0b0111_1110,
            0b0110_0000,
            0b1001_1110,
            0b0110_1000,
            0b1011_1110,
            0b0111_0000,
            0b1101_1110,
            0b0111_1000,
        ];

        assert_eq!(result_bytes, expect_bytes);

        Ok(())
    }

    #[test]
    fn can_gen_c2_inst() -> Result<()> {
        let text = load_test_asm("test/asm/c2_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b0001_1111,
            0b0000_1000,
            0b0001_1111,
            0b0001_0000,
            0b0001_1111,
            0b0001_1000,
        ];

        assert_eq!(result_bytes, expect_bytes);
        Ok(())
    }

    #[test]
    fn can_gen_trap_inst() -> Result<()> {
        let text = load_test_asm("test/asm/trap_inst_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![0b1111_1111, 0b1111_1111];

        assert_eq!(result_bytes, expect_bytes);
        Ok(())
    }

    #[test]
    fn can_gen_direcitve() -> Result<()> {
        let text = load_test_asm("test/asm/directive_test.asm");

        let (exprs, label_table) = parse(text, 0)?;
        let result_bytes = gen(exprs, label_table)?;
        let expect_bytes: Vec<u8> = vec![
            0b1111_1111,
            0b1111_1111,
            0b1111_0000,
            0b0010_1101,
            0b0000_0000,
            0b0010_1110,
            0b0000_0000,
        ];

        assert_eq!(result_bytes, expect_bytes);
        Ok(())
    }

    fn load_test_asm(path: &str) -> String {
        let file = std::fs::File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();
        text
    }
}
