use std::time::Duration;

use anyhow::Result;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    time::{sleep, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut input = reader.lines();

    let mut lastbatt = String::new();

    let minute = Duration::from_secs(5);
    let mut minago = Instant::now();

    loop {
        tokio::select! {
            _ = sleep(minute) => {
                // likely slept/hiber if this diffs too much
                if minago.elapsed() >= Duration::from_secs(10) {
                    ectool(&lastbatt);
                }
                minago = Instant::now();
            }

            line = input.next_line() => {
                let line = line?;
                if let Some(l) = line {
                    // TODO remove this hack
                    if l.contains("exit") {
                        std::process::exit(0);
                    } else {
                    ectool(&l);
                    if l.contains("fwchargelimit") {
                        lastbatt = l;
                    }}
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
