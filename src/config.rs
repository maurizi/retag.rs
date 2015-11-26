extern crate toml;

use {Args};

use toml::Table;

use std::env;
use std::fs::File;
use std::io::{Read, BufReader, ErrorKind};


static DEFAULT_TAG: &'static str = "tags";
static DEFAULT_CMD: &'static str = "ctags";


pub fn read_config(args: &mut Args) {
    if let Some(config) = get_config() {
        read_setting(&config, &mut args.arg_TAGFILE, "tagfile");
        read_setting(&config, &mut args.flag_tag_cmd, "cmd");
    }
    if args.arg_TAGFILE == "" {
        args.arg_TAGFILE = DEFAULT_TAG.to_string();
    }
    if args.flag_tag_cmd == "" {
        args.flag_tag_cmd = DEFAULT_CMD.to_string();
    }
}

fn read_setting(config: &Table, setting: &mut String, key: &str) {
    if *setting == "" {
        if let Some(tagfile_config) = config.get(key) {
            if let Some(tagfile_path) = tagfile_config.as_str() {
                *setting = tagfile_path.to_string();
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

        match File::open(&option_path) {
            Ok(file) => {
                let mut toml_contents = String::new();
                if BufReader::new(file).read_to_string(&mut toml_contents).is_ok() {
                    return toml::Parser::new(&toml_contents).parse();
                }
            },
            Err(ref e) if e.kind() != ErrorKind::NotFound => {
                println!("Could not read {:?}, cause: {}", option_path, e);
            },
            _ => {}
        };
    }
    None
}
