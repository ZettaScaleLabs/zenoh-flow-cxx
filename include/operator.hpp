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

#pragma once
#include <wrapper.hpp>

namespace zenoh {
namespace flow {

class State {
private:
  std::uint8_t counter;
public:
  State ();
  void increaseCounter ();
  std::uint8_t getCounter ();
};

// Configuration is a JSON string, use any C++ JSON library to parse it.
std::unique_ptr<State> initialize(rust::Str json_configuration);

bool
input_rule(Context &context, std::unique_ptr<State> &state,
           rust::Vec<InputToken> &tokens);

rust::Vec<Output>
run(Context &context, std::unique_ptr<State> &state, rust::Vec<Input> inputs);

rust::Vec<Output>
output_rule(Context &context, std::unique_ptr<State> &state, rust::Vec<Output> run_outputs, LocalDeadlineMiss deadlinemiss);

} // namespace flow
} // namespace zenoh
