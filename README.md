# qwb-2020-fuzzer

Fuzzers for finding vulnerabilities in the binaries of Qiangwang Cup 2020.

## Build

You must have Rust toolchain installed to build the fuzzer. It's recommended to install Rust toolchain via
[rustup](https://rustup.rs/).

Clone the repository:

```shell script
git clone https://github.com/Lancern/qwb-2020-fuzzer
cd qwb-2020-fuzzer
```

Then build using `cargo`:

```shell script
cargo build
```

The built artifacts are placed in the `target/debug` directory.

## Fuzzing

The fuzzer is based on [AFL++](https://github.com/AFLplusplus/AFLplusplus). Please build and install AFL++ before 
fuzzing. We have tested our fuzzer on AFL++ 2.68c and we recommend you use this version as well.

For instructions to build AFL++, please refer to its documentation.

### babynotes

After successfully build, two binaries relevant to fuzzing `babynotes` will be produced under `target/debug`:
* `babynotes-seed-cli`: CLI utility for manipulating seed files for fuzzing `babynotes`;
* `libbabynotes-mutator.so`: AFL++ custom mutator plugin for fuzzing `babynotes`.

To fuzz `babynotes`, create a fuzzing directory and create links to the above binaries:

```shell script
mkdir babynotes-fuzzing
cd babynotes-fuzzing
ln -s /path/to/babynotes-seed-cli babynotes-seed-cli
ln -s /path/to/libbabynotes-mutator.so libbabynotes-mutator.so
ln -s /path/to/babynotes babynotes
```

Then generate an empty test case as the seed test case for fuzzing `babynotes`:

```shell script
mkdir seeds
./babynotes-seed-cli gen -o seeds/seed
```

Then we can launch the fuzzer:

```shell script
export AFL_CUSTOM_MUTATOR_LIBRARY=./libbabynotes-mutator.so
export AFL_CUSTOM_MUTATOR_ONLY=1
afl-fuzz -i ./seeds -o findings -Q -m none -- babynotes
```

To run a test case that makes `babynotes` crash, you need to synthesis the test case into actual input format before
executing `babynotes`:

```shell script
./babynotes-seed-cli syn -o seed.syn /path/to/crash-test-case
babynotes < seed.syn
```

### babyheap

> Under construction
