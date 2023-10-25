use anyhow::{anyhow, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{alpha1, alphanumeric1, hex_digit1, multispace0},
    error::ErrorKind,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Inst {
        inst_type: InstType,
        mnemonic: String,
        rd: String,
        rs: String,
        imm: String,
        symbol: String,
        address: u16,
    },
    Const {
        val: String,
        address: u16,
    },
    Label {
        name: String,
        address: u16,
    },
}

#[derive(Debug, PartialEq)]
pub enum InstType {
    R,
    I5,
    I8,
    C1,
    C2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
    pub address: u16,
}

pub fn parse(text: String, base_address: u16) -> Result<(Vec<Expr>, Vec<Label>)> {
    let mut exprs: Vec<Expr> = Vec::new();
    let mut label_table: Vec<Label> = Vec::new();
    let mut address: u16 = base_address;

    for (line_num, line) in text.lines().enumerate() {
        // delete empty line and comment
        if line.trim().is_empty() || line.trim().starts_with("//") {
            continue;
        }
        match parse_line(line) {
            Ok((_, expr)) => match expr {
                Expr::Label { name, .. } => {
                    label_table.push(Label { name, address });
                }
                Expr::Inst {
                    inst_type,
                    mnemonic,
                    rd,
                    rs,
                    imm,
                    symbol,
                    ..
                } => {
                    exprs.push(Expr::Inst {
                        inst_type,
                        mnemonic,
                        rd,
                        rs,
                        imm,
                        symbol,
                        address,
                    });
                    address += 2;
                }
                Expr::Const { val, .. } => {
                    exprs.push(Expr::Const { val, address });
                    address += 2;
                }
            },
            Err(_) => {
                return Err(anyhow!("Syntax Error : line {}:{}", line_num + 1, line));
            }
        }
    }
    Ok((exprs, label_table))
}

fn parse_line(line: &str) -> IResult<&str, Expr> {
    let (line, _) = multispace0(line)?;
    let result = alt((parse_inst, parse_word, parse_label))(line)?;
    Ok(result)
}

fn parse_inst(line: &str) -> IResult<&str, Expr> {
    fn till_space(s: &str) -> IResult<&str, &str> {
        take_till(|c| c == ' ')(s)
    }
    fn till_atmark(s: &str) -> IResult<&str, &str> {
        take_till(|c| c == '@')(s)
    }

    let (line, mnemonic) = alpha1(line)?;
    match mnemonic {
        // R Instruction
        "mov" | "add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" => {
            let (line, _) = multispace0(line)?;
            let (line, rd) = alphanumeric1(line)?;
            let (line, _) = multispace0(line)?;
            let (line, _) = tag(",")(line)?;
            let (line, _) = multispace0(line)?;
            let (line, rs) = alphanumeric1(line)?;
            Ok((
                line,
                Expr::Inst {
                    inst_type: InstType::R,
                    mnemonic: mnemonic.to_string(),
                    rd: rd.to_string(),
                    rs: rs.to_string(),
                    imm: "".to_string(),
                    symbol: "".to_string(),
                    address: 0,
                },
            ))
        }
        // I5 Instruction
        "addi" | "subi" | "beq" | "bnq" | "blt" | "bge" | "bltu" | "bgeu" | "jalr" | "lw"
        | "sw" => {
            let (line, _) = multispace0(line)?;
            let (line, rd) = alphanumeric1(line)?;
            let (line, _) = multispace0(line)?;
            let (line, _) = tag(",")(line)?;
            let (line, _) = multispace0(line)?;
            let (line, rs) = alphanumeric1(line)?;
            let (line, _) = multispace0(line)?;
            let (line, _) = tag(",")(line)?;
            let (line, _) = multispace0(line)?;
            let (line, imm) = till_space(line)?;

            Ok((
                line,
                Expr::Inst {
                    inst_type: InstType::I5,
                    mnemonic: mnemonic.to_string(),
                    rd: rd.to_string(),
                    rs: rs.to_string(),
                    imm: imm.to_string(),
                    symbol: "".to_string(),
                    address: 0,
                },
            ))
        }

        // I8 Instruction
        "jal" | "lil" | "lih" => {
            let (line, _) = multispace0(line)?;
            let (line, rd) = alphanumeric1(line)?;
            let (line, _) = multispace0(line)?;
            let (line, _) = tag(",")(line)?;
            let (line, _) = multispace0(line)?;
            if line.contains('@') {
                let (line, imm) = till_atmark(line)?;
                let (line, _) = tag("@")(line)?;
                let (line, symbol) = till_space(line)?;
                Ok((
                    line,
                    Expr::Inst {
                        inst_type: InstType::I8,
                        mnemonic: mnemonic.to_string(),
                        rd: rd.to_string(),
                        rs: "".to_string(),
                        imm: imm.to_string(),
                        symbol: symbol.to_string(),
                        address: 0,
                    },
                ))
            } else {
                let (line, imm) = till_space(line)?;

                Ok((
                    line,
                    Expr::Inst {
                        inst_type: InstType::I8,
                        mnemonic: mnemonic.to_string(),
                        rd: rd.to_string(),
                        rs: "".to_string(),
                        imm: imm.to_string(),
                        symbol: "".to_string(),
                        address: 0,
                    },
                ))
            }
        }

        // C1 Instruction
        "push" | "pop" | "rpc" | "rsp" | "rpsr" | "rtlr" | "rthr" | "wsp" | "wpsr" | "wtlr"
        | "wthr" => {
            let (line, _) = multispace0(line)?;
            let (line, rd) = alphanumeric1(line)?;
            Ok((
                line,
                Expr::Inst {
                    inst_type: InstType::C1,
                    mnemonic: mnemonic.to_string(),
                    rd: rd.to_string(),
                    rs: "".to_string(),
                    imm: "".to_string(),
                    symbol: "".to_string(),
                    address: 0,
                },
            ))
        }

        // C2 Instruction
        "rfi" | "rtr" | "wtr" => Ok((
            line,
            Expr::Inst {
                inst_type: InstType::C2,
                mnemonic: mnemonic.to_string(),
                rd: "".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
        )),
        _ => {
            let err = nom::error::Error::new(line, ErrorKind::Fail);
            Err(nom::Err::Error(err))
        }
    }
}

fn parse_word(line: &str) -> IResult<&str, Expr> {
    let (line, _) = multispace0(line)?;
    let (line, _) = tag(".")(line)?;
    let (line, directive) = alpha1(line)?;
    match directive {
        "word" => {
            let (line, _) = multispace0(line)?;
            let (line, _) = tag("0x")(line)?;
            let (line, hex) = hex_digit1(line)?;
            Ok((
                line,
                Expr::Const {
                    val: hex.to_string(),
                    address: 0,
                },
            ))
        }
        _ => {
            let err = nom::error::Error::new(line, ErrorKind::Fail);
            Err(nom::Err::Error(err))
        }
    }
}

fn parse_label(line: &str) -> IResult<&str, Expr> {
    let (line, name) = alphanumeric1(line)?;
    let (line, _) = tag(":")(line)?;
    Ok((
        line,
        Expr::Label {
            name: name.to_string(),
            address: 0,
        },
    ))
}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::{BufReader, Read};

    #[test]
    fn can_parse_r_inst() -> Result<()> {
        let file = std::fs::File::open("asm/r_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_label_table: Vec<Label> = vec![
            Label {
                name: "start".to_string(),
                address: 0,
            },
            Label {
                name: "end".to_string(),
                address: 18,
            },
        ];
        let expect_exprs: Vec<Expr> = vec![
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "mov".to_string(),
                rd: "x0".to_string(),
                rs: "zero".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "add".to_string(),
                rd: "x1".to_string(),
                rs: "ra".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "sub".to_string(),
                rd: "x2".to_string(),
                rs: "fp".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 4,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "and".to_string(),
                rd: "x3".to_string(),
                rs: "a0".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 6,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "or".to_string(),
                rd: "x4".to_string(),
                rs: "a1".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 8,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "xor".to_string(),
                rd: "x5".to_string(),
                rs: "a2".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 10,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "sll".to_string(),
                rd: "x6".to_string(),
                rs: "t0".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 12,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "srl".to_string(),
                rd: "x7".to_string(),
                rs: "t1".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 14,
            },
            Expr::Inst {
                inst_type: InstType::R,
                mnemonic: "sra".to_string(),
                rd: "x0".to_string(),
                rs: "x1".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 16,
            },
        ];
        assert_eq!(result_label_table, expect_label_table);
        assert_eq!(result_exprs, expect_exprs);

        Ok(())
    }

    #[test]
    fn can_parse_i5_inst() -> Result<()> {
        let file = std::fs::File::open("asm/i5_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_label_table: Vec<Label> = vec![
            Label {
                name: "start".to_string(),
                address: 0,
            },
            Label {
                name: "end".to_string(),
                address: 22,
            },
        ];
        let expect_exprs: Vec<Expr> = vec![
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "addi".to_string(),
                rd: "x0".to_string(),
                rs: "zero".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "subi".to_string(),
                rd: "x1".to_string(),
                rs: "ra".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "beq".to_string(),
                rd: "x2".to_string(),
                rs: "fp".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 4,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "bnq".to_string(),
                rd: "x3".to_string(),
                rs: "a0".to_string(),
                imm: "-1".to_string(),
                symbol: "".to_string(),
                address: 6,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "blt".to_string(),
                rd: "x4".to_string(),
                rs: "a1".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 8,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "bge".to_string(),
                rd: "x5".to_string(),
                rs: "a2".to_string(),
                imm: "-1".to_string(),
                symbol: "".to_string(),
                address: 10,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "bltu".to_string(),
                rd: "x6".to_string(),
                rs: "t0".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 12,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "bgeu".to_string(),
                rd: "x7".to_string(),
                rs: "t1".to_string(),
                imm: "-1".to_string(),
                symbol: "".to_string(),
                address: 14,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "jalr".to_string(),
                rd: "x0".to_string(),
                rs: "x1".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 16,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "lw".to_string(),
                rd: "x2".to_string(),
                rs: "x3".to_string(),
                imm: "-1".to_string(),
                symbol: "".to_string(),
                address: 18,
            },
            Expr::Inst {
                inst_type: InstType::I5,
                mnemonic: "sw".to_string(),
                rd: "x4".to_string(),
                rs: "x5".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 20,
            },
        ];
        assert_eq!(result_label_table, expect_label_table);
        assert_eq!(result_exprs, expect_exprs);

        Ok(())
    }

    #[test]
    fn can_parse_i8_inst() -> Result<()> {
        let file = std::fs::File::open("asm/i8_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_label_table: Vec<Label> = vec![
            Label {
                name: "start".to_string(),
                address: 0,
            },
            Label {
                name: "end".to_string(),
                address: 6,
            },
        ];
        let expect_exprs: Vec<Expr> = vec![
            Expr::Inst {
                inst_type: InstType::I8,
                mnemonic: "jal".to_string(),
                rd: "zero".to_string(),
                rs: "".to_string(),
                imm: "1".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::I8,
                mnemonic: "lil".to_string(),
                rd: "ra".to_string(),
                rs: "".to_string(),
                imm: "0x1".to_string(),
                symbol: "l".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::I8,
                mnemonic: "lih".to_string(),
                rd: "fp".to_string(),
                rs: "".to_string(),
                imm: "0x1".to_string(),
                symbol: "h".to_string(),
                address: 4,
            },
        ];
        assert_eq!(result_label_table, expect_label_table);
        assert_eq!(result_exprs, expect_exprs);

        Ok(())
    }

    #[test]
    fn can_parse_c1_inst() -> Result<()> {
        let file = std::fs::File::open("asm/c1_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_label_table: Vec<Label> = vec![
            Label {
                name: "start".to_string(),
                address: 0,
            },
            Label {
                name: "end".to_string(),
                address: 22,
            },
        ];
        let expect_exprs: Vec<Expr> = vec![
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "push".to_string(),
                rd: "zero".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "pop".to_string(),
                rd: "ra".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "rpc".to_string(),
                rd: "fp".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 4,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "rsp".to_string(),
                rd: "a0".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 6,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "rpsr".to_string(),
                rd: "a1".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 8,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "rtlr".to_string(),
                rd: "a2".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 10,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "rthr".to_string(),
                rd: "t0".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 12,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "wsp".to_string(),
                rd: "t1".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 14,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "wpsr".to_string(),
                rd: "x0".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 16,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "wtlr".to_string(),
                rd: "x1".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 18,
            },
            Expr::Inst {
                inst_type: InstType::C1,
                mnemonic: "wthr".to_string(),
                rd: "x2".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 20,
            },
        ];
        assert_eq!(result_label_table, expect_label_table);
        assert_eq!(result_exprs, expect_exprs);

        Ok(())
    }

    #[test]
    fn can_parse_c2_inst() -> Result<()> {
        let file = std::fs::File::open("asm/c2_inst_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_label_table: Vec<Label> = vec![
            Label {
                name: "start".to_string(),
                address: 0,
            },
            Label {
                name: "end".to_string(),
                address: 6,
            },
        ];
        let expect_exprs: Vec<Expr> = vec![
            Expr::Inst {
                inst_type: InstType::C2,
                mnemonic: "rfi".to_string(),
                rd: "".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::C2,
                mnemonic: "rtr".to_string(),
                rd: "".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::C2,
                mnemonic: "wtr".to_string(),
                rd: "".to_string(),
                rs: "".to_string(),
                imm: "".to_string(),
                symbol: "".to_string(),
                address: 4,
            },
        ];
        assert_eq!(result_label_table, expect_label_table);
        assert_eq!(result_exprs, expect_exprs);

        Ok(())
    }

    #[test]
    fn can_parse_word() -> Result<()> {
        let file = std::fs::File::open("asm/word_test.asm").unwrap();
        let mut reader = BufReader::new(file);
        let mut text = String::new();
        reader.read_to_string(&mut text).unwrap();

        let (result_exprs, result_label_table) = parse(text, 0)?;
        let expect_exprs: Vec<Expr> = vec![
            Expr::Const {
                val: "ffff".to_string(),
                address: 0,
            },
            Expr::Inst {
                inst_type: InstType::I8,
                mnemonic: "lil".to_string(),
                rd: "x1".to_string(),
                rs: "".to_string(),
                imm: "word".to_string(),
                symbol: "l".to_string(),
                address: 2,
            },
            Expr::Inst {
                inst_type: InstType::I8,
                mnemonic: "lih".to_string(),
                rd: "x1".to_string(),
                rs: "".to_string(),
                imm: "word".to_string(),
                symbol: "h".to_string(),
                address: 4,
            },
        ];
        let expect_label_table: Vec<Label> = vec![Label {
            name: "word".to_string(),
            address: 0,
        }];
        assert_eq!(result_exprs, expect_exprs);
        assert_eq!(result_label_table, expect_label_table);

        Ok(())
    }
}
