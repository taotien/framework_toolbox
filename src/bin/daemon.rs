use std::time::Duration;

use anyhow::Result;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    time::{sleep, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    let minute = Duration::from_secs(60);
    let mut minago = Instant::now();

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut input = reader.lines();

    let mut lastargs = String::new();

    loop {
        tokio::select! {
            _ = sleep(minute) => {
                // likely slept/hiber if this diffs too much
                if minago.elapsed() >= Duration::from_secs(69) {
                    ectool(&lastargs);
                }
                minago = Instant::now();
            }

            line = input.next_line() => {
                let line = line?;
                match line {
                    Some(l) =>{
                        ectool(&l);
                        lastargs = l;
                        }
                    None => {panic!()}
                }
            }
        }
    }
}

fn ectool(s: &str) {
    let a = s.split_whitespace();
    std::process::Command::new("ectool")
        .args(a)
        .output()
        .unwrap();
}
