use anyhow::{Context, Result};
use std::io::{BufReader, Read, Write};
mod gen;
mod parse;

use clap::Parser;

#[derive(Parser)]
#[clap(version = "0.1", author = "kinpoko", about = "ZKTC assembler")]
struct Args {
    /// .asm file path
    file_path: std::path::PathBuf,

    /// output file name
    #[arg(short = 'o', default_value = "a.mem")]
    output_file_name: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = std::fs::File::open(&args.file_path)
        .with_context(|| format!("could not read file '{}'", &args.file_path.display()))?;
    let mut reader = BufReader::new(file);
    let mut text = String::new();
    reader.read_to_string(&mut text)?;

    let (expr, label_table) = parse::parse(text)?;
    let words = gen::gen(expr, label_table)?;

    let mut output_file = std::fs::File::create(&args.output_file_name)
        .with_context(|| "could not create file".to_string())?;

    for word in words {
        writeln!(output_file, "{:02x}", word & 0x00FF)?;
        writeln!(output_file, "{:02x}", (word & 0xFF00) >> 8)?;
    }

    Ok(())
}
