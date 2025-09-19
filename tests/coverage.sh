#!/bin/bash
#
# script to run project tests and report code coverage
# uses llvm-cov (https://github.com/taiki-e/cargo-llvm-cov)

set -eo pipefail

BINLIST_FILE="target/cov_binlist.txt"
COV=("cargo" "llvm-cov")
COV_OPTS=("--no-report" "--release")
CARGO_TEST_OPTS=("--")
COV_RUN_DIR="target/cov_runs"
COV_RUN_FILE="$COV_RUN_DIR/run"
RUN_UNIT_TESTS=1
RUST_TEST_THREADS=1

_die() {
    echo "ERR: $*"
    exit 1
}

_tit() {
    echo
    echo "========================================"
    echo "$@"
    echo "========================================"
}

help() {
    echo "$NAME [<option>] [...]"
    echo ""
    echo "options:"
    echo "    -h  --help             show this help message"
    echo "    -i  --ignore-run-fail  run all tests regardless of failure"
    echo "    -t  --test <test>      only run <test> integration test(s)"
    echo "    -nu --no-unit          don't run unit tests for submodules"
    echo "    -tt --threads <n>      run with <n> test threads (default: 1)"
    echo "        --ci               run in CI mode"
}

# cmdline arguments
while [ -n "$1" ]; do
    case $1 in
        -h | --help)
            help
            exit 0
            ;;
        -i | --ignore-run-fail)
            COV_OPTS+=("--ignore-run-fail")
            ;;
        -t | --test)
            [ -z "$2" ] && _die "test pattern required"
            CARGO_TEST_OPTS+=("$2")
            shift
            ;;
        -nu | --no-unit)
            RUN_UNIT_TESTS=0
            ;;
        -tt | --threads)
            [ -z "$2" ] && _die "test thread number required"
            if ! [ "$2" -gt 0 ] 2>/dev/null; then
                _die "invalid test thread number ($2)"
            fi
            RUST_TEST_THREADS="$2"
            shift
            ;;
        --ci)
            CI=1
            ;;
        *)
            help
            _die "unsupported argument \"$1\""
            ;;
    esac
    shift
done

# requirements
if [ -z "$CI" ]; then
    _tit "installing requirements"
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov
fi

# coverage
export CARGO_LLVM_COV_TARGET_DIR="target/llvm-cov-target"
export RUST_TEST_THREADS

## initial cleanup
_tit "cleaning previous coverage data"
"${COV[@]}" clean

## initial setup
rm -f $BINLIST_FILE && mkdir -p target && touch $BINLIST_FILE
mkdir -p $COV_RUN_DIR && rm -f $COV_RUN_FILE.*
RUN=0

_tit "generating coverage data"
## integration tests (electrum)
export INDEXER=electrum
"${COV[@]}" "${COV_OPTS[@]}" "${CARGO_TEST_OPTS[@]}" 2>&1 | tee $COV_RUN_FILE.$RUN
((RUN += 1))
## integration tests (esplora + altered)
export INDEXER=esplora
"${COV[@]}" "${COV_OPTS[@]}" "${CARGO_TEST_OPTS[@]}" 2>&1 | tee $COV_RUN_FILE.$RUN
((RUN += 1))
SKIP_INIT=1 "${COV[@]}" "${COV_OPTS[@]}" --features altered "${CARGO_TEST_OPTS[@]}" 2>&1 | tee $COV_RUN_FILE.$RUN
((RUN += 1))
# integration test cleanup
unset INDEXER
docker compose -f tests/compose.yaml --profile='*' down -v --remove-orphans

## unit tests
if [ "$RUN_UNIT_TESTS" = 1 ]; then
    SUBMODULE_PATHS=$(git submodule | awk '{print $2}' | grep -v altered_submodules)
    for SP in $SUBMODULE_PATHS; do
        FEATURES="--all-features"
        [ "$SP" = "rgb-ascii-armor" ] && FEATURES=""
        "${COV[@]}" "${COV_OPTS[@]}" --manifest-path "$SP/Cargo.toml" --workspace $FEATURES --all-targets 2>&1 | tee $COV_RUN_FILE.$RUN
        ((RUN += 1))
    done
fi

# report
_tit "generating coverage report"
## generate unique + sorted binary list from run logs
find $COV_RUN_DIR -type f | while read -r RF; do
    awk -F'(' '/Running/ {print $2}' "$RF" | awk -F')' '{print $1}' >>$BINLIST_FILE
done
sort -u "$BINLIST_FILE" -o "$BINLIST_FILE"
## generate llvm-cov object list from binary list
LLVM_COV_OBJECTS=()
BINLIST=$(cat $BINLIST_FILE)
for B in $BINLIST; do
    LLVM_COV_OBJECTS+=("-object" "$B")
done
## get llvm-cov path (version used by cargo-llvm-cov)
LLVM_COV="$(rustc --print sysroot)/lib/rustlib/$(rustc -Vv | grep host | awk '{print $2}')/bin/llvm-cov"
## generate coverage report
PROFDATA_PATH="target/llvm-cov-target/rgb-tests.profdata"
IGNORE_PATTERN="/rgb\-tests(/.*)?/(tests|examples|benches)/|/rgb\-tests/target/llvm\-cov\-target($|/)|^$HOME/\.cargo/(registry|git)/|^$HOME/\.rustup/toolchains($|/)"
if [ -z "$CI" ]; then
    # generate default report (profdata)
    "${COV[@]}" report --release --html
    if [ "$RUN_UNIT_TESTS" = 1 ]; then
        # merge coverage from all tested binaries (incl. unit tests)
        $LLVM_COV show \
            --format=html \
            --output-dir=target/llvm-cov/html \
            --instr-profile=$PROFDATA_PATH \
            --ignore-filename-regex="$IGNORE_PATTERN" \
            -show-instantiations=false \
            -show-line-counts-or-regions \
            -show-expansions \
            -show-branches=count \
            -show-mcdc \
            -Xdemangler="$HOME/.cargo/bin/cargo-llvm-cov" \
            -Xdemangler=llvm-cov \
            -Xdemangler=demangle \
            "${LLVM_COV_OBJECTS[@]}"
    fi
    # show html report location
    echo "generated html report: target/llvm-cov/html/index.html"
else
    LCOV_FILE="coverage.lcov"
    # generate default report (profdata)
    "${COV[@]}" report --release --lcov --output-path $LCOV_FILE --verbose
    if [ "$RUN_UNIT_TESTS" = 1 ]; then
        # merge coverage from all tested binaries (incl. unit tests)
        $LLVM_COV export \
            --format=lcov \
            --instr-profile=$PROFDATA_PATH \
            --ignore-filename-regex="$IGNORE_PATTERN" \
            "${LLVM_COV_OBJECTS[@]}" \
            >$LCOV_FILE
    fi
fi
