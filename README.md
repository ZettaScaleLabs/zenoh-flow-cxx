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

A single CMake file is provided in order to accomplish both tasks.

### Requirements

- Rust: see the [installation page](https://www.rust-lang.org/tools/install)
- cxxbridge

  ```sh
  cargo install cxxbridge-cmd
  ```


### Building your C++ component

On Unix-based machines.

```sh
mkdir build && cd build
cmake ..
make
```

This will:
1. generate the rust library that wraps the Zenoh-Flow APIs,
2. call `cxxbridge` to generate the bindings needed by Zenoh Flow, in particular the header file `include/wrapper.hpp` and the source file `src/wrapper.cpp`,
3. "patch" the header file `include/wrapper.hpp`,
4. compile the Rust code located under the `vendor/wrapper` folder and generate a static library `libwrapper.a`,
5. compile the C++ wrapper code,
6. compile the operator,
7. link everything together producing `build/libcxx_operator.dylib` (`.so` on Linux).

The `libcxx_operator` library can then be loaded by Zenoh Flow!

-----------
# Acknowledgments

We relied on the  [CXX — safe FFI between Rust and C++](https://github.com/dtolnay/cxx) library to generates the bindings.
