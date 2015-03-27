# Build the binary if it is not already present and not on the PATH
retag_dir=$(dirname $0)

if ! which retag >/dev/null; then
    PATH="$PATH:$retag_dir/target/release"
    if [[ ! -f "$retag_dir/target/release/retag" ]]; then
        if which rustc >/dev/null && which cargo >/dev/null; then
            echo "Building retag"
            current_dir=$(pwd)
            cd $retag_dir && cargo build --release && cd $current_dir
        else
            print "retag binary not present, and rustc and cargo are also not present\n"\
                  "Please install Rust or add the retag binary to your PATH" >&2
            return
        fi
    fi
fi

function retag_cwd() {
    # Check if this is a Git repo
    PROJECT_ROOT=`git rev-parse --show-toplevel 2> /dev/null`
    if (( $? != 0 )); then
        PROJECT_ROOT=""
    fi

    # If we just moved out of our current project, stop retag
    if [[ -n $CURRENT_RETAG_DIR && $PROJECT_ROOT != $CURRENT_RETAG_DIR ]]; then
        start-stop-daemon --stop -p "$CURRENT_RETAG_DIR/.git/retag.pid" --exec $(which retag) --oknodo >/dev/null 2>/dev/null
    fi

    # If we moved into a new project, stqrt retag up
    if [[ "$PROJECT_ROOT" != "" ]]; then
        # TODO: Add daemonization to retag itself and drop start-stop-daemon
        start-stop-daemon --start -m --pidfile "$PROJECT_ROOT/.git/retag.pid" --exec $(which retag) -b --chdir $PROJECT_ROOT >/dev/null 2>/dev/null
        if (( $? == 0 )); then
            export CURRENT_RETAG_DIR=$(pwd)
        fi
    elif [[ -n $CURRENT_RETAG_DIR ]]; then
        unset CURRENT_RETAG_DIR
    fi
}


# Append retag_cwd to the chpwd_functions array, so it will be called on cd
# http://zsh.sourceforge.net/Doc/Release/Functions.html
if ! (( $chpwd_functions[(I)retag_cwd] )); then
    chpwd_functions+=(retag_cwd)
fi
