#!/usr/bin/env bash
#
# change all submodules in the current project to the requested branch or tag
#
# optionally use the specified remote where the branch or tag are to be found
# optionally fetch the specified remote, or all if none was specified
#
# if the specified parameters are invalid (e.g. the branch doesn't exist),
# nothing is done and execution continues to the next submodule

# colors
NC='\033[0m'        # no color
BLUE='\033[0;34m'   # blue
GREEN='\033[0;32m'  # green
ORANGE='\033[0;33m' # orange
RED='\033[0;31m'    # red

# vars
BRANCH=""                           # branch to change to
DIR="$(realpath "$(dirname "$0")")" # script path
FETCH=0                             # don't fetch by default
REMOTE=""                           # remote where to look for revs
TAG=""                              # tag to change to

# helper functions
_die() {
    printf "\n${RED}ERROR: %s${NC}\n" "$@"
    exit 1
}

_head() {
    printf "${GREEN}%s${NC}\n" "$@"
}

_log() {
    printf "${BLUE}%s${NC}\n" "$@"
}

_war() {
    printf "${ORANGE}WARNING: %s${NC}\n" "$@"
}

_init() {
    # change to script directory (project root)
    pushd "$DIR" >/dev/null || exit 1

    # get submodule list
    if ! [ -r .gitmodules ]; then
        _die "project has no git submodules"
    fi
    SUBS=$(git submodule | awk '{print $2}' | grep -v altered_submodules)
}

_cleanup() {
    # go back to calling directory
    while [ "$(dirs -p | wc -l)" -gt 1 ]; do
        popd >/dev/null || exit 1
    done
    exit 0
}

# help functions
change_help() {
    echo "$0 change <options>"
    echo ""
    echo "options:"
    echo "    -b --branch <branch>  change to the specified branch"
    echo "    -f --fetch            fetch provided remote (or all if none specified)"
    echo "    -h --help             show this message help"
    echo "    -r --remote <remote>  remote to be used"
    echo "    -t --tag    <tag?     change to the specified tag"
}

fetch_help() {
    echo "$0 fetch [options]"
    echo ""
    echo "options:"
    echo "    -h --help             show this message help"
    echo "    -r --remote <remote>  remote to be used"
}

# command functions
change() {
    # option handling
    while [ -n "$1" ]; do
        case $1 in
            -h | --help)
                change_help
                exit 0
                ;;
            -b | --branch)
                BRANCH="$2"
                shift
                ;;
            -f | --fetch)
                FETCH=1
                ;;
            -r | --remote)
                REMOTE="$2"
                shift
                ;;
            -t | --tag)
                TAG="$2"
                shift
                ;;
            *)
                change_help
                _die "unsupported option \"$1\""
                ;;
        esac
        shift
    done

    # check branch/tag combination makes sense
    if [ -z "$BRANCH" ] && [ -z "$TAG" ]; then
        change_help
        _die "please specify a branch or a tag to switch to"
    fi
    if [ -n "$BRANCH" ] && [ -n "$TAG" ]; then
        change_help
        _die "please either specify a branch or a tag, not both"
    fi

    # optionally add the remote
    if [ -n "$REMOTE" ]; then
        if [ -n "$BRANCH" ]; then
            BRANCH="$REMOTE/$BRANCH"
        else
            change_help
            _die "specifying a remote only makes sense if a branch is specified as well"
        fi
    fi

    # update submodule revs
    for sub in $SUBS; do
        # init
        _head "---- submodule: $sub"
        while [ "$(dirs -p | wc -l)" -gt 2 ]; do
            popd >/dev/null || exit 1
        done
        pushd "$DIR/$sub" >/dev/null || exit 1
        # make sure the remote exists, if specified
        if [ -n "$REMOTE" ]; then
            if ! git remote | grep -xq "$REMOTE"; then
                _war "remote \"$REMOTE\" not found"
                continue
            fi
        fi
        # update repo
        if [ "$FETCH" = 1 ]; then
            if [ -n "$REMOTE" ]; then
                git fetch "$REMOTE"
            else
                git fetch --all
            fi
        fi
        # branch case
        if [ -n "$BRANCH" ]; then
            # check if specified branch exists
            if ! git branch -r | grep -wq "$BRANCH"; then
                _war "branch \"$BRANCH\" not found"
                continue
            fi
            # checkout the specified branch
            git checkout "$BRANCH"
            # pull latest changes
            git pull
        fi
        # tag case
        if [ -n "$TAG" ]; then
            if ! git tag | grep -xq "$TAG"; then
                _war "tag \"$TAG\" not found"
                continue
            fi
            # checkout the specified tag
            git checkout "$TAG"
        fi
    done
}

fetch() {
    # option handling
    while [ -n "$1" ]; do
        case $1 in
            -h | --help)
                fetch_help
                exit 0
                ;;
            -r | --remote)
                REMOTE="$2"
                shift
                ;;
            *)
                fetch_help
                _die "unsupported option \"$1\""
                ;;
        esac
        shift
    done

    # remote handling
    [ -n "$REMOTE" ] || REMOTE="--all"

    # fetch submodules
    # shellcheck disable=SC2016
    git submodule foreach \
        "git fetch $REMOTE || :" # ignore failures (i.e. missing remote)
}

help() {
    echo "$0 <command>"
    echo ""
    echo "commands:"
    echo "    change <options>  change submodule revs"
    echo "    fetch  <options>  fetch all submodules"
    echo "    help              show this help message"
    echo "    status            show all submodule revs"
}

status() {
    # option handling
    [ -n "$1" ] && _die "the status command takes no arguments"

    # shellcheck disable=SC2016
    git submodule foreach --quiet \
        'echo "" $(git rev-list -n1 HEAD) $sm_path $(git describe --all | sed "s,^\(tags\|remotes\|heads\)/,(,;s/$/)/")'
}

# initialize and setup cleanup on exit
if [ -z "$1" ]; then
    help
    _die "please provide a command"
fi
_init
trap _cleanup EXIT HUP INT QUIT TERM

# command handling
VALID_CMDS=("change" "fetch" "help" "status")
CMD="$1"
if [ -n "${VALID_CMDS[$CMD]}" ]; then
    shift
    $CMD "$@"
else
    help
    _die "unsupported command \"$1\""
fi
