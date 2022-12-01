use std::io;
use std::process::Command;

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        if buffer.is_empty() {
            break;
        };
        let buffer = buffer.split_whitespace();
        Command::new("ectool").args(buffer).output()?;
    }
    Ok(())
}
