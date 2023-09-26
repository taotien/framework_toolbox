use std::sync::mpsc::channel;

use anyhow::Result;
use dirs::config_dir;
use notify::{
    event::{AccessKind, AccessMode},
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use strum::AsRefStr;

use framework_toolbox::{LedColor, Toolbox, ToolboxDiff};

fn main() -> Result<()> {
    let mut conf_path = config_dir().unwrap();
    conf_path.push("fwtb.toml");
    let mut conf_diff: ToolboxDiff;
    let mut conf_old = Toolbox::default();

    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(&conf_path, RecursiveMode::Recursive).unwrap();
    for res in rx {
        match res {
            Ok(event) => {
                println!("event: {:?}", event);
                if let EventKind::Access(AccessKind::Close(mode)) = event.kind {
                    if mode == AccessMode::Write {
                        let conf_new = Toolbox::parse().unwrap();
                        conf_diff = conf_old.diff(&conf_new);

                        {
                            if let Some(limit) = conf_diff.battery_limit {
                                Ectool::FwChargeLimit(limit).call();
                            }
                            if let Some(duty) = conf_diff.fan_duty {
                                Ectool::FanDuty(duty).call();
                            }
                            if let Some(auto) = conf_diff.fan_auto {
                                if auto {
                                    Ectool::AutoFanCtrl.call();
                                }
                            }
                            if let Some(Some(color)) = conf_diff.led_power {
                                Ectool::Led(LedSide::Power(color)).call();
                            }
                            if let Some(Some(color)) = conf_diff.led_left {
                                Ectool::Led(LedSide::Left(color)).call();
                            }
                            if let Some(Some(color)) = conf_diff.led_right {
                                Ectool::Led(LedSide::Right(color)).call();
                            }
                        }

                        conf_old = conf_new;
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

#[derive(AsRefStr)]
#[rustfmt::skip]
enum Ectool {
    FwChargeLimit(u8),
    #[strum(serialize = "fwchargelimit")]
    FwChargeOnce(u8),
    AutoFanCtrl,
    FanDuty(u8),
    Led(LedSide),
}

impl Ectool {
    fn call(&self) {
        let mut ectool = std::process::Command::new("ectool");
        match self {
            Self::FwChargeLimit(v) => ectool.args([self.as_ref(), &format!("{}", v)]),
            Self::FwChargeOnce(v) => ectool.args([self.as_ref(), &format!("{}", v), "once"]),
            Self::AutoFanCtrl => ectool.arg(self.as_ref()),
            Self::FanDuty(v) => ectool.args([self.as_ref(), &format!("{}", v)]),
            Self::Led(v) => ectool.args([
                self.as_ref(),
                v.as_ref(),
                match v {
                    LedSide::Power(i) => i.as_ref(),
                    LedSide::Left(i) => i.as_ref(),
                    LedSide::Right(i) => i.as_ref(),
                },
            ]),
        };
        ectool.output().unwrap();
    }
}

#[derive(AsRefStr)]
enum LedSide {
    Power(LedColor),
    Left(LedColor),
    Right(LedColor),
}
