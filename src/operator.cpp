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

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <memory>
#include <operator.hpp>
#include <string>
#include <vector>

#include <iostream>

namespace zenoh {
namespace flow {

State::State() { counter = 0; }

void State::increaseCounter(void) { counter += 1; }

std::uint8_t State::getCounter(void) { return counter; }

std::unique_ptr<State> initialize(rust::Str json_configuration) {
  std::cout << "Configuration: " << std::endl;
  std::cout << json_configuration.data() << '\0' << std::endl;
  return std::make_unique<State>();
}

bool input_rule(Context &context, std::unique_ptr<State> &state,
                rust::Vec<InputToken> &tokens) {
  for (auto token : tokens) {
    if (token.status != TokenStatus::Ready) {
      return false;
    }
  }

  return true;
}

rust::Vec<Output> run(Context &context, std::unique_ptr<State> &state,
                      rust::Vec<Input> inputs) {
  state->increaseCounter();
  rust::Vec<std::uint8_t> counter = {state->getCounter()};
  Output count{"count", counter};
  rust::Vec<Output> results{count};
  return results;
}

rust::Vec<Output> output_rule(Context &context, std::unique_ptr<State> &state,
                              rust::Vec<Output> run_outputs,
                              LocalDeadlineMiss deadlinemiss) {
  // DeadlineMiss should be handled here.
  return run_outputs;
}

} // namespace flow
} // namespace zenoh
