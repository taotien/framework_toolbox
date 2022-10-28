use std::io;
use std::io::{Error, Write};
use std::process::{Command, Output};

fn main() {
    let stdin = io::stdin();
    loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).expect("couldn't read stdin");
        print!("{}", &buffer);
        match buffer.trim() {
            "charge" => {
                let mut buffer = String::new();
                stdin.read_line(&mut buffer).expect("couldn't read stdin");
                print!("{}", &buffer);
                charge(buffer.trim().to_string()).expect("fail");
            }
            "fan" => {
                let mut buffer = String::new();
                stdin.read_line(&mut buffer).expect("couldn't read stdin");
                print!("{}", &buffer);
                fan(buffer.trim().to_string()).expect("fail");
            }
            "autofan" => {
                autofan().expect("fail");
            }
            "backlight" => {
                let mut buffer = String::new();
                stdin.read_line(&mut buffer).expect("couldn't read stdin");
                print!("{}", &buffer);
                backlight(buffer.trim().to_string()).expect("fail");
            }
            _ => break,
        };
    }
}

fn charge(lim: String) -> Result<Output, Error> {
    Command::new("ectool")
        .arg("fwchargelimit")
        .arg(lim)
        .output()
}

#[rustfmt::skip]
fn fan(duty: String) -> Result<Output, Error> {
    Command::new("ectool")
        .arg("fanduty")
        .arg(duty)
        .output()
}

#[rustfmt::skip]
fn autofan() -> Result<Output, Error> {
    Command::new("ectool")
        .arg("autofanctrl")
        .output()
}

fn backlight(val: String) -> Result<(), Error> {
    let mut f = std::fs::File::create("/sys/class/backlight/intel_backlight/brightness")
        .expect("couldn't open backlight device file");
    f.write_all(val.as_bytes())
}
