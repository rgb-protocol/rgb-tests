# RGB tests

This repository contains tests for RGB.

## Requirements
- Linux OS
- Docker with its `compose` plugin
- Rust

## Run
Clone the project, including submodules:

```sh
git clone https://github.com/rgb-protocol/rgb-tests --recurse-submodules
```

Note: after checking out to another commit, remember to run:
```sh
git submodule update
```

### Integration tests

To run the integration tests, from the project root, execute:
```sh
cargo test --test issuance --test transfers
```

:warning: **Warning:** if your machine has a lot of CPU cores, it could
happen that calls to indexers fail because of too many parallel requests. To
limit the test threads and avoid this issue set the `--test-threads` option
(e.g. `cargo test --test issuance --test transfers -- --test-threads=8`).

### Validation tests

To run consignment validation tests, from the project root, execute:

```sh
cargo test --test validation
```

### Stress tests

To run a single stress test, set the `LOOPS` variable to the requested number
of loops and then, from the project root, for example execute:
```sh
LOOPS=20 cargo test --test stress back_and_forth::case_1 -- --ignored
```

This will produce a CSV report file that can be opened in a spreadsheet program
manually or by running:
```sh
open test-data/stress/<timestamp>.csv
```

Stress tests have been parametrized the same way some integration tests are.
To select which test case you want to run, find the case attribute you want to
use (e.g. `#[case(TT::Witness, DT::Wpkh, DT::Tr)]`) and if, as an example, it's
the 4th one, run `<test_name>::case_4`. Note that case numbers are zero-padded
so if for example there are 20 test cases, case 4 would be called `case_04`.

### Test services

Test services will be automatically (re)started by the test command and will
run in docker containers.
If you don't have the docker images they will be automatically pulled. Note
that in this case the first test execution will be slower.
Also note that there's no automatic shutdown of test services, you'll need to
manually remove the docker containers with:
```sh
docker compose -f tests/compose.yaml --profile='*' down -v --remove-orphans
```

The indexer used by the tests is configurable, currently esplora and electrum
are supported. You can change the indexer type by setting the `INDEXER`
environment variable, for example:
```sh
INDEXER=esplora cargo test  # default
INDEXER=electrum cargo test
```

If you are developing new tests and want a faster execution, you can set
`SKIP_INIT=1` to avoid restarting the test services. Please note that you
cannot switch to another indexer when using this option, you'll have to use the
same indexer type from the previous test execution.

A feature `altered` is there to run some specific tests with altered crates
(e.g. tests that simulate a modified sender that tries to perform an attack).
Run `cargo test --features altered` to run those tests.

### Coverage

To run the tests and generate a code coverage report run:
```sh
./tests/coverage.sh
```
This will generate a report in `target/llvm-cov/html/index.html` that you can
visualize on a browser (e.g. `firefox target/llvm-cov/html/index.html`).

Coverage will be measured for all patched crates.

The GitHub organizations of submodule repositories are:
- https://github.com/rgb-protocol
- https://github.com/LNP-BP
- https://github.com/BP-WG
- https://github.com/aluvm
- https://github.com/strict-types
- https://github.com/rust-amplify
- https://github.com/UBIDECO

## Contribute

### Submodule revision change helper
A helper to change the revision for all submodules is available in the form of
the `sub-rev-change.sh` script.

As an example, it can be used to point all submodules to the `master` branch
with:
```sh
./submodules-rev.sh change --branch master
```

See the help for more details on its usage:
```sh
./submodules-rev.sh help
```

To check which revision is checked-out for each submodule run:
```sh
./submodules-rev.sh status
```

### PRs showing bugs

If a PR introduces tests showing bugs it will be merged if the failing tests
have the `#[ignore]` attribute. When the project is updated to a new RGB
release, maintainers will check if the ignored tests are passing and if so the
attribute will be dropped.
