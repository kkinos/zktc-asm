use crate::parse::{Expr, InstType, Label};
use anyhow::{anyhow, Context, Result};

pub fn gen(expr: Vec<Expr>, label_table: Vec<Label>) -> Result<Vec<u16>> {
    let mut words: Vec<u16> = Vec::new();

    'outer: for e in expr {
        match e {
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
                    words.push(word);
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
                        words.push(word);
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
                        words.push(word);
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
                                let imm = label.address as i32 - address as i32 - 4;
                                if !(-128..=127).contains(&imm) {
                                    return Err(anyhow!("could not jump to {}", label.name));
                                }
                                let word: u16 = opcode & 0x001F
                                    | (rd & 0x0007) << 5
                                    | (imm as u16 & 0x00FF) << 8;
                                words.push(word);
                                continue 'outer;
                            }
                        }
                        let imm = imm
                            .parse::<i8>()
                            .with_context(|| format!("could not parse {}", imm))?;
                        let word: u16 =
                            opcode & 0x001F | (rd & 0x0007) << 5 | (imm as u16 & 0x00FF) << 8;
                        words.push(word);
                    } else if imm.starts_with("0x") {
                        let imm = u16::from_str_radix(imm.trim_start_matches("0x"), 16)
                            .with_context(|| format!("could not parse {}", imm))?;
                        if symbol.as_str() == "l" {
                            let word: u16 =
                                opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0x00FF) << 8;
                            words.push(word);
                            continue 'outer;
                        } else if symbol.as_str() == "h" {
                            let word: u16 = opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0xFF00);
                            words.push(word);
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
                                    words.push(word);
                                    continue 'outer;
                                } else if symbol.as_str() == "h" {
                                    let word: u16 =
                                        opcode & 0x001F | (rd & 0x0007) << 5 | (imm & 0xFF00);
                                    words.push(word);
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
                        "wsp" => 0b01000,
                        "wpsr" => 0b01001,
                        "wtlr" => 0b01010,
                        "wthr" => 0b01011,
                        _ => unreachable!(),
                    };
                    let rd = gen_reg(rd)?;
                    let word: u16 = 0x001E | (rd & 0x0007) << 5 | (func & 0x001F) << 11;
                    words.push(word);
                }
                InstType::C2 => {
                    let func: u16 = match mnemonic.as_str() {
                        "rfi" => 0b00001,
                        "rtr" => 0b00010,
                        "wtr" => 0b00011,
                        _ => unreachable!(),
                    };
                    let word: u16 = 0x001F | (func & 0x001F) << 11;
                    words.push(word);
                }
            },
            Expr::Const { val, .. } => {
                let word: u16 = u16::from_str_radix(&val, 16)
                    .with_context(|| format!("could not parse 0x{}", val))?;
                words.push(word);
            }
            _ => unreachable!(),
        }
    }

    Ok(words)
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
    fn can_gen_test_asms() -> Result<()> {
        let file = std::fs::File::open("asm/all_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (expr, label_table) = parse(text)?;
        gen(expr, label_table)?;

        let file = std::fs::File::open("asm/comment_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (expr, label_table) = parse(text)?;
        gen(expr, label_table)?;

        let file = std::fs::File::open("asm/reg_name_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (expr, label_table) = parse(text)?;
        gen(expr, label_table)?;

        let file = std::fs::File::open("asm/word_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (expr, label_table) = parse(text)?;
        gen(expr, label_table)?;

        Ok(())
    }
}
