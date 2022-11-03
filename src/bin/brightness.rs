use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Seek},
    process::Command,
};

// check if output scale was requested
// adjust only if ambient goes over scale and manual adjustment
// reset if auto is toggled
//
// try brightnessctl,fallback to writing to intel driver

fn main() {
    // configs
    //
    let averaging = 5; // samples to retrieve before adjusting
    let sample_ms = 500; // time between samples collected
    let offset = 28800;
    let fps = 60;
    let transition_ms: u64 = 1000;

    let mut sensor = File::open("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")
        .expect("couldn't open illuminance sensor");
    let max = String::from_utf8(
        Command::new("brightnessctl")
            .arg("m")
            .output()
            .expect("couldn't call brightnessctl")
            .stdout,
    )
    .unwrap();
    let max = max.trim().parse::<i32>().unwrap();
    let scale = max / 3355; // scale of brightness to sensor
    let smooth = transition_ms / fps;
    let mut last_target: i32 = 0;
    let mut avg = VecDeque::with_capacity(averaging);
    let mut idx = 0;
    loop {
        let mut ambient = String::new();
        sensor.rewind().unwrap();
        sensor.read_to_string(&mut ambient).unwrap();
        let ambient = ambient.trim().parse::<i32>().unwrap();

        let current = String::from_utf8(
            Command::new("brightnessctl")
                .arg("g")
                .output()
                .expect("couldn't call brightnessctl")
                .stdout,
        )
        .unwrap();
        let current = current.trim().parse::<i32>().unwrap();
        // let extern_change = current - current_last;

        if idx < averaging {
            avg.pop_front();
            avg.push_back(ambient);
            idx += 1;
        } else {
            let ambient = Iterator::sum::<i32>(avg.iter()) / i32::try_from(avg.len()).unwrap();
            let target = ambient * scale + offset;

            let step = (target - current) / smooth as i32;

            if target != last_target {
                for _ in 0..smooth {
                    let mut set = Command::new("brightnessctl");
                    set.arg("s");

                    if step.is_positive() {
                        set.arg(format!("+{}", step.to_string()));
                    } else {
                        set.arg(format!("{}-", step.abs().to_string()));
                    }

                    set.output().unwrap();

                    std::thread::sleep(std::time::Duration::from_millis(
                        (transition_ms / smooth) as u64,
                    ));
                }
                last_target = target;
            }
            idx = 0;
        }

        // TODO change to non-blocking to allow collect more sensor data points?
        // will need to track sensor changes rather than spam reads
        std::thread::sleep(std::time::Duration::from_millis(sample_ms));
    }
}
