extern crate toml;

use {Args};

use toml::Table;

use std::env;
use std::fs::{File, PathExt};
use std::io::{Read, BufReader};


static DEFAULT_TAG: &'static str = "tags";
static DEFAULT_CMD: &'static str = "ctags";


pub fn read_config(args: &mut Args) {
    if let Some(config) = get_config() {
        read_setting(&config, &mut args.arg_TAGFILE, "tagfile");
        read_setting(&config, &mut args.flag_tag_cmd, "cmd");
    }
    if args.arg_TAGFILE == "" {
        args.arg_TAGFILE = String::from_str(DEFAULT_TAG);
    }
    if args.flag_tag_cmd == "" {
        args.flag_tag_cmd = String::from_str(DEFAULT_CMD);
    }
}

fn read_setting(config: &Table, setting: &mut String, key: &str) {
    if *setting == "" {
        if let Some(tagfile_config) = config.get(key) {
            if let Some(tagfile_path) = tagfile_config.as_str() {
                *setting = String::from_str(tagfile_path);
            } else {
                println!("Invalid setting for option: {}", key);
            }
        }
    }
}

// TODO: Use more appropriate directory on Windows
fn get_config() -> Option<Table> {
    if let Some(home) = env::home_dir() {
        let option_path = home.join(".config").join("retag.toml");
        if option_path.exists() {
            match File::open(&option_path) {
                Ok(file) => {
                    let mut toml_contents = String::new();
                    if BufReader::new(file).read_to_string(&mut toml_contents).is_ok() {
                        return toml::Parser::new(&toml_contents).parse();
                    }
                },
                Err(e) => {
                    println!("Could not read {:?}, cause: {}", option_path, e);
                }
            };
        }
    }
    None
}
