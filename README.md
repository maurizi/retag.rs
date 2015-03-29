# retag.rs [![Build Status](https://travis-ci.org/maurizi/retag.rs.svg?branch=master)](https://travis-ci.org/maurizi/retag.rs)
A tool that watches a directory and rebuilds your Exuberant Ctags when files change

Inspired by [vim-gutentags](https://github.com/ludovicchabant/vim-gutentags), this plugin watches for file changes in a directory and incrementally rebuilds your tag file for the files which have changed

To use this tool, Exuberant Ctags must be installed and on your PATH.

To have `retag` automatically watch your Git projects when you CD into them, you can install this repository as a ZSH plugin.
If you have `rustc` and `cargo` installed, `retag` will be compiled when first used.
For installation instructions on ZSH plugins, please see https://github.com/unixorn/awesome-zsh-plugins#installation.

PRs adding support for other shells are highly welcome!

Built using rust nightly 2015-03-26.
