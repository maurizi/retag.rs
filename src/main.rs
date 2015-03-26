#![feature(old_io)]
#![feature(io)]
#![feature(std_misc)]
#![feature(collections)]
#![feature(path_ext)]

extern crate docopt;
extern crate glob;
extern crate notify;
extern crate tempdir;
extern crate toml;
extern crate "rustc-serialize" as rustc_serialize;

use docopt::Docopt;

use std::env;
use std::path::{PathBuf, Path};

mod watcher;
mod config;

static USAGE: &'static str = "
Usage: retags [options] [TAGFILE]
Watches the current directory for changes and updates a ctags TAGFILE
TAGFILE is, by default 'tags'

Options:
  -h, --help        Show this message.
  --tag-cmd=<cmd>   The tag program to use, defaults to 'ctags'.
                    Pass 'etags' to generate an Emacs compatible tag file

You may specify also specify the following options in ~/.config/retag.toml:
 - tagfile
 - cmd
";

#[derive(RustcDecodable, Debug)]
#[allow(non_snake_case)]
pub struct Args {
    arg_TAGFILE: String,
    flag_tag_cmd: String
}

fn main() {
    let mut args: Args = Docopt::new(USAGE)
                                .and_then(|d| d.decode())
                                .unwrap_or_else(|e| e.exit());

    config::read_config(&mut args);

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Could not determine current directory: {}", e.description())
    };

    let tag = &args.arg_TAGFILE;
    let tag_file = if Path::new(tag).is_relative() {
        current_dir.join(tag)
    } else {
        PathBuf::new(tag)
    };

    watcher::watch_project(&current_dir, &tag_file, &args.flag_tag_cmd);
}
