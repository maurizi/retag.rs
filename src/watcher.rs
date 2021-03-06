use glob::Pattern;
use notify::{Event, RecommendedWatcher, Watcher};
use notify::Error as NotifyError;
use tempdir::TempDir;

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write, ErrorKind};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

macro_rules! pattern(($r:expr) => ({
    match Pattern::new($r) {
        Ok(r) => r,
        Err(e) => panic!("Couldn't parse glob: {}", e)
    }
}));

pub struct TagWatcher<'a> {
    project_dir: &'a Path,
    tag_path: PathBuf,
    tag_cmd: &'a str,
    tmp_dir: TempDir
}

impl <'a> TagWatcher<'a> {
    pub fn new(project_dir: &'a Path, tag: &str, tag_cmd: &'a str) -> TagWatcher<'a> {
        let tmp_dir = TempDir::new("retag").ok().expect("Could not create temp directory");

        // Using an absolute path to the tagfile will make Ctags use absolute paths for file
        // references, which makes it easier to filter out files later.
        let tag_path = if Path::new(tag).is_relative() {
            project_dir.join(tag)
        } else {
            PathBuf::from(tag)
        };

        TagWatcher {
            project_dir: project_dir,
            tag_path: tag_path,
            tag_cmd: tag_cmd,
            tmp_dir: tmp_dir
        }
    }

    pub fn watch_project(self) {
        let (file_change_tx, file_change_rx) = channel();
        let w: Result<RecommendedWatcher, NotifyError> = Watcher::new(file_change_tx);

        match w {
            Ok(mut watcher) => {
                watcher.watch(&self.project_dir).ok().expect("Could not start file watcher");

                self.create_tagfile().ok().expect("Could not build tag file");

                self.wait_for_files(|| file_change_rx.recv(), &mut |path| {
                    // Sleep for a little bit, then collect all queued file notifications
                    // This should allow us to only regenerate the tag file once for a group of
                    // file changes, e.g. whena git operation happens
                    //
                    // TODO: Vary the sleep time based on how long the initial tag generation is
                    thread::sleep_ms(500);

                    let mut changed_files = HashSet::new();
                    changed_files.insert(path);

                    self.wait_for_files(|| file_change_rx.try_recv(), &mut |path| {
                        changed_files.insert(path);
                    });

                    match self.regenerate_tags(&changed_files) {
                        Ok(_) => println!("Rebuilt tag file for: {:?}", changed_files),
                        Err(e) => println!("Failed to rebuild tags, error {}", e)
                    }
                });
            },
            Err(_) => panic!("Could not start file watcher")
        }
    }

    fn wait_for_files<R, B, E>(&self, receiver: R, body: &mut B)
        where R : Fn() -> Result<Event, E>, B : FnMut(PathBuf) {
        // Repeatedly calls `receiver` until it returns an `Err`
        // For each `Event`, call the `body` callback if it is not an ignored file
        while let Ok(e) = receiver() {
            if let Some(path) = e.path {
                if let Ok(canon_path) = path.canonicalize() {
                    if ! self.ignored(&canon_path) {
                        body(canon_path);
                    }
                }
            }
        }
    }

    fn ignored(&self, f: &Path) -> bool {
        let ignored = [
            pattern!(r"**/.git/**"),
            pattern!(r"**/.hg/**"),
            pattern!(r"**/.svn/**"),
        ];
        if let Ok(f_metadata) = fs::metadata(f){
            // Ignore directories, version control files, and always ignore changes to the tag file
            return f_metadata.is_dir() || f == self.tag_path.as_path() || ignored.iter().any(|p| p.matches_path(f));
        } else {
            // Also ignore if we couldn't read the file metadata (it likely does not exist)
            return true;
        }
    }

    fn create_tagfile(&self) -> Result<(), IoError> {
        let tmp_tag = self.get_tmp_tag();

        let project_dir_str = self.project_dir.to_str()
            .expect("Could not determine current directory");

        let tmp_tag_str = tmp_tag.to_str().expect("Could not load tag file path");

        let mut ctags = Command::new(self.tag_cmd);
        let mut cmd = ctags
            .arg("-f").arg(tmp_tag_str)
            .arg("--recurse")
            .arg(project_dir_str);

        try!(self.run_ctags(&mut cmd, &tmp_tag));

        Ok(())
    }

    fn regenerate_tags(&self, changed_files: &HashSet<PathBuf>) -> Result<(), IoError> {
        let path_strs = paths_to_strs(changed_files);

        let tmp_tag = &try!(self.filter_tagfile_into_temp(&path_strs));
        let tmp_tag_str = match tmp_tag.to_str() {
            Some(filename) => filename,
            None => {
                return Err(IoError::new(ErrorKind::Other, "Could not open temporary file"));
            }
        };

        let mut ctags = Command::new(self.tag_cmd);
        let mut cmd = ctags.arg("-f").arg(tmp_tag_str);

        for path in path_strs.iter() {
            cmd.arg("--append").arg(path);
        }

        try!(self.run_ctags(&mut cmd, &tmp_tag));

        Ok(())
    }

    fn filter_tagfile_into_temp(&self, path_strs: &HashSet<&str>) -> Result<PathBuf, IoError> {
        // First, filter the tag file into a temp file excluding the changed files
        // This is done to prevent duplicate tags, as ctags does not remove tags
        // from your existing tag file when you use '--append'
        let tmp_tag = self.get_tmp_tag();

        let cur_tag_file = BufReader::new(try!(File::open(&self.tag_path)));
        let mut tmp_tag_file = BufWriter::new(try!(File::create(&tmp_tag)));

        println!("{:?}", path_strs);

        // Copy lines that do not reference changed files into the temporary tag file
        for line in cur_tag_file.lines() {
            if let Ok(line) = line {
                if path_strs.iter().all(|&p| !line.contains(p)) {
                    try!(writeln!(&mut tmp_tag_file, "{}", line));
                }
            }
        }

        Ok(tmp_tag)
    }

    fn get_tmp_tag(&self) -> PathBuf {
        // We use a temp file to avoid interfering with usage from the existing tag file
        self.tmp_dir.path().join("tags.temp")
    }

    fn run_ctags(&self, cmd: &mut Command, tmp_tag: &Path) -> Result<(), IoError> {
        println!("Running {:?}", cmd);

        let status = try!(cmd.status());

        if ! status.success() {
            return Err(IoError::new(ErrorKind::Other, "Ctags exited with a non-zero error code"));
        }

        try!(fs::rename(tmp_tag, &self.tag_path));

        Ok(())
    }
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
