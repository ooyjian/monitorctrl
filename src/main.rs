use clap::{arg, command};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::fs::{write, File};
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use std::str;

#[derive(Deserialize, Serialize, Debug)]
struct MonitorSettings {
    brightness: i32,
    contrast: i32,
}

struct Args {
    lvl: u32,
    read_config: bool,
    enable_presets: bool,
}

fn parse_input() -> Args {
    let matches = command!()
        .arg(
            arg!([lvl] "level of brightness & contrast")
                .value_parser(clap::value_parser!(u32))
                .default_value("0"),
        )
        .arg(arg!(--read_config "read brightness & contrast from config file"))
        .arg(arg!(--enable_presets "use preset values base on the workspace"))
        .get_matches();

    let lvl = *matches
        .get_one::<u32>("lvl")
        .expect("lvl is required as i32");
    let read_config = if let Some(r) = matches.get_one::<bool>("read_config") {
        *r
    } else {
        false
    };
    let enable_presets = if let Some(p) = matches.get_one::<bool>("enable_presets") {
        *p
    } else {
        false
    };

    Args {
        lvl,
        read_config,
        enable_presets,
    }
}

fn read_config(config_path: &Path) -> Result<MonitorSettings, Box<dyn Error>> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);

    let settings = serde_json::from_reader(reader)?;
    Ok(settings)
}

fn write_config(config_path: &Path, settings: &MonitorSettings) {
    let values = json!(settings);
    write(config_path, values.to_string()).expect("unable to write to config");
}

fn exec_ddcutil(settings: &MonitorSettings) {
    Command::new("ddcutil")
        .args(["setvcp", "10", &settings.brightness.to_string()])
        .output()
        .expect("failed to execute ddcutil brightness");
    Command::new("ddcutil")
        .args(["setvcp", "12", &settings.contrast.to_string()])
        .output()
        .expect("failed to execute ddcutil contrast");
}

fn main() {
    let config_path = Path::new("/home/yj/.local/state/monitor-control/config.json");

    let args = parse_input();

    let game_mode = {
        let output = Command::new("hyprctl")
            .arg("activewindow")
            .arg("-j")
            .output()
            .unwrap();
        let activewindow_json = str::from_utf8(&output.stdout).unwrap();
        let v: Value = serde_json::from_str(activewindow_json).unwrap();
        v["workspace"]["name"] == "special:steam"
    };

    let (settings, write_to_config) = {
        if args.enable_presets && game_mode {
            (
                MonitorSettings {
                    brightness: 50,
                    contrast: 50,
                },
                false,
            )
        } else if args.read_config {
            match read_config(&config_path) {
                Ok(s) => (s, false),
                Err(_) => (
                    MonitorSettings {
                        brightness: 0,
                        contrast: 0,
                    },
                    false,
                ),
            }
        } else {
            let tmp_settings: MonitorSettings;
            match args.lvl {
                0 => {
                    tmp_settings = MonitorSettings {
                        brightness: 0,
                        contrast: 0,
                    }
                }
                1 => {
                    tmp_settings = MonitorSettings {
                        brightness: 8,
                        contrast: 7,
                    }
                }
                2 => {
                    tmp_settings = MonitorSettings {
                        brightness: 23,
                        contrast: 15,
                    }
                }
                3 => {
                    tmp_settings = MonitorSettings {
                        brightness: 50,
                        contrast: 50,
                    }
                }
                _ => {
                    tmp_settings = MonitorSettings {
                        brightness: 0,
                        contrast: 0,
                    }
                }
            }
            (tmp_settings, true)
        }
    };

    exec_ddcutil(&settings);
    if write_to_config {
        write_config(config_path, &settings);
    }
}
