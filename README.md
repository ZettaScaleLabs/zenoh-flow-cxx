# C++ Zenoh Flow Components

[![Join the chat at https://gitter.im/atolab/zenoh-flow](https://badges.gitter.im/atolab/zenoh-flow.svg)](https://gitter.im/atolab/zenoh-flow?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

[Zenoh Flow](https://github.com/eclipse-zenoh/zenoh-flow) provides a Zenoh-based dataflow programming framework for computations that span from the cloud to the device.

:warning: **This software is still in alpha status and should _not_ be used in production. Breaking changes are likely to happen and the API is not stable.**

-----------

### Requirements

- Rust: see the [installation page](https://www.rust-lang.org/tools/install)
- cxxbridge
  ```sh
  cargo install cxxbridge-cmd
  ```


### Building your C++ node

The following commands have been tested Linux and MacOS machines. They are working with the latest version of [Zenoh Flow](https://github.com/eclipse-zenoh/zenoh-flow).

- Building the Source:
  ```sh
  mkdir build && cd build
  cmake .. -DSOURCE=ON
  make
  ```

- Building the Operator:
  ```sh
  mkdir build && cd build
  cmake .. -DOPERATOR=ON
  make
  ```

- Building the Sink:
  ```sh
  mkdir build && cd build
  cmake .. -DSINK=ON
  make
  ```

This will:
1. generate the rust library that wraps the Zenoh-Flow APIs;
2. call `cxxbridge` to generate the bindings needed by Zenoh Flow, in particular the header `include/wrapper.hpp` and the source `src/wrapper.cpp` files;
3. "patch" the header file `include/wrapper.hpp`;
4. compile the Rust code located under the `vendor/wrapper` folder and generate a static library `libwrapper.a`;
5. compile the C++ wrapper code,
6. compile the node,
7. link everything together producing `build/libcxx_XXX.dylib` (`.so` on Linux) — where `XXX` is the kind of node.

The resulting library can then be loaded by Zenoh Flow!


### Building with docs.

When calling cmake it is possible to pass the `-DBUILD_DOC=ON` parameter, this will instruct CMake to build also the documentation.

The documentation is built leveraging on `doxygen` so verify that it is installed on your machine.

The documentation will then accessible in `build/doc_doxygen/html/index.html`.

-----------
# Acknowledgments

We relied on the  [CXX — safe FFI between Rust and C++](https://github.com/dtolnay/cxx) library to generates the bindings.
