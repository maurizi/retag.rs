# retag.rs [![Build Status](https://travis-ci.org/maurizi/retag.rs.svg?branch=master)](https://travis-ci.org/maurizi/retag.rs) [![Build status](https://ci.appveyor.com/api/projects/status/qxl5t5bjh5qi3c05/branch/master?svg=true)](https://ci.appveyor.com/project/maurizi/retag-rs/branch/master)
`retag` is commandline tool and shell plugin that watches for file changes in a directory and incrementally rebuilds your ctags file for the files which have changed.

## Installation
To use this tool, Exuberant Ctags must be installed and working.

You can install `retag` via `cargo install retag`, or as a ZSH plugin.

## Usage
To have `retag` automatically watch your Git projects when you CD into them, you can install this repository as a ZSH plugin.

Otherwise, you can run it from the command line.  Use `retag --help` for more information.

(The ZSH functionality currently requires `start-stop-daemon` for creating PIDfiles and daemonization.  I'm planning to add support for that directly in the Rust code eventually, to remove the need for `start-stop-daemon` and improve Windows support)

## Contributing
PRs adding support for other shells are highly welcome!
