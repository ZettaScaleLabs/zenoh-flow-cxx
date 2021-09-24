# C++ Zenoh Flow Components

[![Join the chat at https://gitter.im/atolab/zenoh-flow](https://badges.gitter.im/atolab/zenoh-flow.svg)](https://gitter.im/atolab/zenoh-flow?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[Zenoh Flow](https://github.com/eclipse-zenoh/zenoh-flow) provides a Zenoh-based dataflow programming framework for computations that span from the cloud to the device.

:warning: **This software is still in alpha status and should _not_ be used in production. Breaking changes are likely to happen and the API is not stable.**

-----------
## Description

TODO

- [ ] What is a component?
- [ ] Why a separate C++ package?

## Compiling

The generation of the library is a two-steps process:
1. generating the header and source files bridging Rust and C++,
2. generating the shared library.

Two separate CMake files are provided in order to accomplish both tasks.

### Requirements

- Rust: see the [installation page](https://www.rust-lang.org/tools/install)
- cxxbridge

  ```sh
  cargo install cxxbridge-cmd
  ```

### Generating the hearder and source files

On Unix-based machines.

```sh
cd vendor/operator-wrapper
mkdir build && cd build
cmake ..
make
```

This will generate two files:
- `include/operator_wrapper.hpp`
- `src/operator_wrapper.cpp`

Under the hood the `cxxbridge` command is used and parses the file `vendor/operator-wrapper/src/lib.rs` to generate the bindings needed by Zenoh Flow.

:warning: As of 2021-09-24 it seems the generated header file is "incorrect" and a manual edit (realized in the second CMake file) is required.

### Generating the shared library

On Unix-based machines.

```sh
mkdir build && cd build
cmake ..
make
```

This will:
1. "patch" the header file `include/operator_wrapper.hpp`,
2. compile the Rust code located under the `vendor/operator_wrapper` folder and generate a static library `liboperator_wrapper.a`,
3. compile the C++ wrapper code,
4. compile the operator,
5. link everything together producing `build/libcxx_operator.dylib` (`.so` on Linux).

The `libcxx_operator` library can then be loaded by Zenoh Flow!

-----------
# Acknowledgments

We relied on the  [CXX — safe FFI between Rust and C++](https://github.com/dtolnay/cxx) library to generates the bindings.
