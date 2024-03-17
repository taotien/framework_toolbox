use anyhow::Context;
use strum::AsRefStr;

use crate::LedColor;

#[derive(AsRefStr)]
pub enum Ectool {
    FwChargeLimit(u8),
    #[strum(serialize = "fwchargelimit")]
    FwChargeOnce(u8),
    AutoFanCtrl,
    FanDuty(u8),
    Led(LedSide),
}

#[derive(AsRefStr)]
pub enum LedSide {
    Power(LedColor),
    Left(LedColor),
    Right(LedColor),
}

impl Ectool {
    pub fn call(&self) -> anyhow::Result<()> {
        let mut ectool = std::process::Command::new("ectool");
        ectool.arg(self.as_ref());

        match self {
            // Self::FwChargeLimit(value) => ectool.arg(self.as_ref()).arg(value.to_string()).output(),
            Self::FwChargeLimit(value) | Self::FanDuty(value) => {
                ectool.arg(&value.to_string());
            }
            Self::FwChargeOnce(value) => {
                ectool.args([&value.to_string(), "once"]);
            }
            Self::AutoFanCtrl => { // no extra arguments needed
            }
            Self::Led(side) => {
                ectool.args([
                    side.as_ref(),
                    match side {
                        LedSide::Power(color) | LedSide::Left(color) | LedSide::Right(color) => {
                            color.as_ref()
                        }
                    },
                ]);
            }
        };

        // TODO check if output matches that of requested
        ectool.output().context("couldn't call ectool")?;

        Ok(())
    }
}
