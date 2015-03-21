#![feature(old_io)]
#![feature(io)]
#![feature(std_misc)]

extern crate glob;
extern crate notify;
extern crate tempdir;

use glob::Pattern;
use notify::{RecommendedWatcher, Watcher};
use notify::Error as NotifyError;
use tempdir::TempDir;

use std::env;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::old_io::timer;
use std::time::duration::Duration;

macro_rules! ctags_fail {
    ($cause:expr) => (panic!("Failed to start ctags: {}", $cause));
}

macro_rules! pattern(($r:expr) => ({
    match Pattern::new($r) {
        Ok(r) => r,
        Err(e) => panic!("Couldn't parse glob: {}", e)
    }
}));

fn main() {
    let current_dir = &env::current_dir().unwrap();
    // TODO: Get tag file location from cmd line parameter
    let tag_file = current_dir.join(".git").join("tags");

    let mut ctags = Command::new("ctags");
    let mut cmd = ctags
        .arg("-f").arg(tag_file.to_str().unwrap())
        .arg("--recurse")
        .arg(current_dir.to_str().unwrap());

    println!("Running {:?}", cmd);
    let status = cmd.status().unwrap_or_else(|e| {
        ctags_fail!(e);
    });

    if ! status.success() {
        ctags_fail!(status);
    }

    let (file_change_tx, file_change_rx) = channel();
    let w: Result<RecommendedWatcher, NotifyError> = Watcher::new(file_change_tx);
    match w {
        Ok(mut watcher) => {
            match watcher.watch(current_dir) {
                Ok(_) => (),
                Err(_) => ctags_fail!("Could not start file watcher")
            };

            while let Ok(e) = file_change_rx.recv() {
                if let Some(path) = e.path {
                    if ! ignored(&path, &tag_file) {
                        // Sleep for a little bit, then collect all queued file notifications
                        // This should allow us to only regenerate the tag file once for a group of
                        // file changes, e.g. whena git operation happens
                        //
                        // TODO: Vary the sleep time based on how long the initial tag generation is
                        timer::sleep(Duration::seconds(1));

                        let mut changed_files = HashSet::new();
                        changed_files.insert(path);

                        while let Ok(e) = file_change_rx.try_recv() {
                            if let Some(path) = e.path {
                                if ! ignored(&path, &tag_file) {
                                    changed_files.insert(path);
                                }
                            }
                        }

                        match regenerate_tags(&changed_files, &tag_file) {
                            Ok(_) => println!("Rebuilt tag file for: {:?}", changed_files),
                            Err(e) => println!("Failed to rebuild tags, error {}", e.description())
                        }
                    }
                }
            }
        },
        Err(_) => ctags_fail!("Could not start file watcher")
    }
}

fn ignored(f: &Path, tag_file: &Path) -> bool {
    let ignored = vec![
        pattern!(r"**/.git/**"),
        pattern!(r"**/.hg/**"),
        pattern!(r"**/.svn/**"),
    ];
    for pattern in ignored.iter() {
        if pattern.matches_path(f) {
            return true;
        }
    }
    // Always ignore changes to the tag file
    f == tag_file
}

fn regenerate_tags(changed_files: &HashSet<PathBuf>, tag_path: &Path) -> Result<(), Error> {
    let tmp_dir = try!(TempDir::new("retag"));

    let path_strs = paths_to_strs(changed_files);

    let tmp_tag = try!(filter_tagfile_into_temp(&tmp_dir, &path_strs, tag_path));

    let mut ctags = Command::new("ctags");
    let mut cmd = ctags
        .arg("-f").arg(tmp_tag.to_str().unwrap())
        .arg("--append");

    for path in path_strs.iter() {
        cmd.arg(path);
    }

    println!("Running {:?}", cmd);
    let status = try!(cmd.status());

    if ! status.success() {
        let detail = match status.code() {
            Some(code) => Some(format!("Ctags exited with error code: {}", code)),
            None => None
        };
        return Err(Error::new(ErrorKind::Other, "Ctags exited with a non-zero error code", detail));
    }

    try!(fs::rename(&tmp_tag, tag_path));

    Ok(())
}

fn filter_tagfile_into_temp(tmp_dir: &TempDir, path_strs: &HashSet<&str>, tag_path: &Path) -> Result<PathBuf, Error> {
    // First, filter the tag file into a temp file excluding the changed files
    // This is done to prevent duplicate tags, as ctags does not remove tags
    // from your existing tag file when you use '--append'
    //
    // We use a temp file to avoid interfering with usage from the existing tag file
    let tmp_tag = tmp_dir.path().join("tags.temp");

    let cur_tag_file = BufReader::new(try!(File::open(&tag_path)));
    let mut tmp_tag_file = BufWriter::new(try!(File::create(&tmp_tag)));

    println!("{:?}", path_strs);

    // Copy lines that do not reference changed files into the temporary tag file
    for line in cur_tag_file.lines() {
        if let Ok(line) = line {
            if path_strs.iter().all(|&p| !line.contains(p)) {
                try!(tmp_tag_file.write_fmt(format_args!("{}\n", line)));
            }
        }
    }

    Ok(tmp_tag)
}

fn paths_to_strs(paths: &HashSet<PathBuf>) -> HashSet<&str> {
    let mut path_strs = HashSet::new();
    for file in paths.iter() {
        if let Some(path) = file.to_str(){
            path_strs.insert(path);
        }
    }

    path_strs
}
