# retag.rs [![Build Status](https://travis-ci.org/maurizi/retag.rs.svg?branch=master)](https://travis-ci.org/maurizi/retag.rs)
A tool that watches a directory and rebuilds your Exuberant Ctags when files change

Inspired by [vim-gutentags](https://github.com/ludovicchabant/vim-gutentags), this plugin watches for file changes in a directory and incrementally rebuilds your tag file for the files which have changed.

Built using rust nightly 2015-03-26.

To use this tool, Exuberant Ctags must be installed and on your PATH.
