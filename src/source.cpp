//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

#include <memory>
#include <ostream>
#include <thread>
#include <chrono>

#include <source.hpp>

#include <chrono>
#include <thread>

namespace zenoh {
namespace flow {

using byte_t = unsigned char;

State::State() {}

std::unique_ptr<State> initialize(rust::Str json_configuration) {
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<State>();
}

rust::Vec<byte_t>
run(Context &context, std::unique_ptr<State> &state)
{

  std::this_thread::sleep_for(std::chrono::milliseconds(1000));

  rust::Vec<byte_t> tick = { 1 };
  return tick;
}
} // namespace flow
} // namespace zenoh
