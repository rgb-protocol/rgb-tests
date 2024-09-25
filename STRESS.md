# Stress tests

## back and forth

To run the `back_and_forth` stress test, set the `LOOPS` variable to the
requested number of loops and then, from the project root, for example execute:
```sh
LOOPS=20 cargo test --test stress back_and_forth::case_1 -- --ignored
```

This will produce a CSV report file that can be opened in a spreadsheet program
manually or by running:
```sh
open test-data/stress/back_and_forth-<timestamp>.csv
```

Stress tests have been parametrized the same way some integration tests are.
To select which test case you want to run, find the case attribute you want to
use (e.g. `#[case(TT::Witness, DT::Wpkh, DT::Tr)]`) and if, as an example, it's
the 4th one, run `<test_name>::case_4`. Note that case numbers are zero-padded
so if for example there are 20 test cases, case 4 would be called `case_04`.

## random transfers

To run the random transfers stress test, from the project root, execute:
```sh
cargo test --profile reldebug --test stress random_transfers -- --ignored --nocapture
```

The `reldebug` profile is equivalent to release mode plus minimal debug info
and has no performance overhead compared to the default `release` profile. This
ensures the resulting timings reflect the real-world performance and minimizes
the need to recompile code when enabling the `memprof` feature (see below).

The test issues random assets (default: 5) to random wallets (default: 5) and
executes loops (default: 50) that pick a random sender and (1-3) random
recipients and executes a transfer with random parameters (amount, transfer
type).

The random number generator is initialized with a random seed by default. If a
seed is provided via the `SEED` environment variable, that will be used
instead. The test will print the used seed to the console, so it can be used to
reproduce the exact same test choices (along with `ASSETS`, `WALLETS` and
`LOOPS`).

Environment variables that can be set to control the test run:
- `ASSETS`: the number of assets to issue (minimum: 1)
- `LOOPS`: the number of loops (minimum: 1)
- `SEED`: the seed to initialize the random number generator
- `VERBOSE`: if set to 1, more detailed data about each loop will be output
- `WALLETS`: the number of wallets to use (minimum: 1)


Example invocation with variables:
```sh
ASSETS=1 WALLETS=2 LOOPS=100 cargo test --profile reldebug --test stress random_transfers -- --ignored --nocapture
```

Each test run produces multiple files containing data about several aspects of
each loop. Files includes the test parameters in the filename and are saved
inside the `test-data/stress/` directory. The common part of the filename has
the form:
```sh
random_transfers_seeded-<RNG_seed>-<loops>-<assets>-<wallets>-<timestamp>
```

Files for the following metrics are produced:
- time and size CSV
  - sender wallet index
  - transfer recipients (with contract index, amount and transfer type)
  - transfer setup time in ms
  - transfer (send) time in ms
  - block mining (incl. waiting for indexer to sync) time in ms
  - valitation + accept times for consignments (0 if unused) in ms
  - size of consignments (0 if unused) in bytes
  - total wallet sync time in ms
  - number of synced wallets
- CPU usage CSV
  - percent used (per-core)
  - wall time in ms
  - usr CPU time in ms
  - sys CPU time in ms
- network I/O (when restricted to a single interface, see below) CSV
  - received bytes
  - sent bytes
- TCP connections CSV
  - incoming connections
  - outgoing connections
  - failed connections
  - incoming packets
  - bad incoming packets
  - outgoing packets
  - retransmitted packets
  - resets received
  - resets sent
- UDP connections CSV
  - incoming packets
  - outgoing packets
  - dropped packets
  - incoming total errors
  - incoming checksum errors
  - receive buffer errors
  - send buffer errors
  - ignored multicast packets
- disk I/O (when restricted to a single device, see below) CSV
  - read bytes
  - written bytes

Network and disk I/O can be collected for all interfaces and devices, in which
case no file is generated and only a final report is printed. A single device
can be selected via environment variables, to restrict data collection to that
alone and, in this case, a CSV file is generated.
The following environment variables can be set to control I/O data collection:
- `DISK`: the disk device name to restrict I/O collection to
- `NETIF`: the network interface name to restrict I/O collection to

Memory profiling uses [dhat](https://github.com/nnethercote/dhat-rs). It's
disabled by default as it slows down the run significantly due to memory
allocations being slower when profiling. It can be enabled via the `memprof`
feature. It is recommended to always profile with release mode optimizations
enabled, to avoid further performance hits, with minimal debug info present.
The `reldebug` profile addresses this. To execute a profiling run, execute:
```sh
cargo test --profile reldebug --features memprof --test stress random_transfers -- --ignored --nocapture
```

This will produce an additional JSON file that can be viewed with `dh_view`
from the valgrind project. To view the memory profile:
- clone valgrind (`git clone --depth=1 git://sourceware.org/git/valgrind.git`)
- open `dhat/dh_view.html` from the valgrind clone in a browser
- load the desired JSON file
- see the [dh-manual](https://valgrind.org/docs/manual/dh-manual.html) section
  10.3 for details on the output

Enabling the `memprof` feature also also spawns a background thread that
continously samples memory usage and writes the values to a CSV file. The file
will have one row per loop, each containing the used memory in KB sampled each
100ms.

Most of these metrics are system-wide, so resource usage due to other processes
on the machine will be included. For a more precise data collection, it's best
to run the test in isolation. A Dockerfile and compose file are available to
execute test runs inside a Docker container.

To run in docker, support services must be running. To start them execute:
```sh
tests/start_services.sh
```
Then, to run the random transfers test with default parameters, execute:
```sh
docker compose run --rm --build runner
```

Note that, while in general there's no issue in running the test several times
without restarting the underlying services, in the context of seeded runs,
where the outcome is expected to be the same across runs, this might lead to
subtle differences in UTXO management since fee rates might vary between runs.
For this reason it's recommended to execute seeded runs after (re)starting the
services from scratch.

The environment variables described above are passed through to the container
and have the same effect (except for `SKIP_INIT` which is always set).

Some docker-specific environment variables are also available:
- `MEMPROF`: enables memory profiling
- `MY_GID`: sets the group id
- `MY_UID`: sets the user id

Memory profiling needs to be enabled via the `MEMPROF` environment variable as
the test command is set by the docker entrypoint.

Example invocation with variables:
```sh
MEMPROF=1 MY_UID=$(id -u) MY_GID=$(id -g) ASSETS=1 WALLETS=2 LOOPS=100 docker compose run --rm --build runner
```

At the end of the test, the wallets that have been used are serialized and
saved in `test-data/stress/saves/` as a JSON file, which name includes the
parameters used to run the test. This allows to resume a previous run by
setting the `LOAD_ID` environment variable to the json filename (extention
excluded). For the resume to work the services used for the previous run need
to still be in execution, so setting `SKIP_INIT` to 1 is mandatory and is done
automatically when `LOAD_ID` is set.
As an example, if a test is run with `docker compose run --rm --build runner`
and outputs:
```sh
saved test wallets to test-data/stress/saves/13774037641746752869-50-100-50-1761491993.json
```
the run can be resumed by calling:
```sh
LOAD_ID=13774037641746752869-50-100-50-1761491993 docker compose run --rm runner
```
This loads the saved wallets and the test parameters (seed, loop number, asset
number and wallet number). The loop number can be overridden by setting the
`LOOPS` variable like in normal runs but the `SEED`, `ASSETS` and `WALLETS`
variables are instead ignored and the ones from the `LOAD_ID` are always used.
Note that only the previous run can be resumed and it's not possible to resume
the same saved run twice as the wallet states have since changed.

When developing, if there are no changes or only to test code in `tests/`, the
`--build` parameter can be omitted to avoid re-buildind the image each time.
