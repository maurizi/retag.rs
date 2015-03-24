#![feature(old_io)]
#![feature(io)]
#![feature(std_misc)]
#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate docopt;
extern crate glob;
extern crate notify;
extern crate tempdir;
extern crate "rustc-serialize" as rustc_serialize;

use std::env;
use std::path::{PathBuf, Path};

mod watcher;

docopt!(Args derive Debug, "
Usage: retags [options] [TAGFILE]
Watches the current directory for changes and updates a ctags TAGFILE
TAGFILE is, by default 'tags'

Options:
  -h, --help        Show this message.
  --tag-cmd=<cmd>   The tag program to use [default: ctags].
                    Pass 'etags' to generate an Emacs compatible tag file
");

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Could not determine current directory: {}", e.description())
    };

    let tag: &str = if args.arg_TAGFILE != "" {
        &args.arg_TAGFILE
    } else {
        "tags"
    };
    let tag_file = if Path::new(tag).is_relative() {
        current_dir.join(tag)
    } else {
        PathBuf::new(tag)
    };

    watcher::watch_project(&current_dir, &tag_file, &args.flag_tag_cmd);
}
