//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

#include <iostream>
#include <memory>
#include <ostream>
#include <source.hpp>

namespace zenoh {
namespace flow {

using byte_t = unsigned char;

State::State() {}

std::unique_ptr<State>
initialize(const rust::Vec<Configuration> &configuration)
{
  //
  // /!\ NOTE: `make_unique` requires "c++14"
  //
  return std::make_unique<State>();
}

rust::Vec<byte_t>
run(Context &context, std::unique_ptr<State> &state)
{
  std::string input;

  std::cout << "Press ENTER.";
  std::getline(std::cin, input);

  rust::Vec<byte_t> tick = { 1 };
  return tick;
}
} // namespace flow
} // namespace zenoh
