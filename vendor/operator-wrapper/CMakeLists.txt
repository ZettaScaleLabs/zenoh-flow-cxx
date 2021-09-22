#
# Copyright (c) 2017, 2021 ADLINK Technology Inc.
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ADLINK zenoh team, <zenoh@adlink-labs.tech>
#

cmake_minimum_required(VERSION 3.10)

project(CxxWrapper VERSION 0.1.0)

find_program(CXXBRIDGE cxxbridge REQUIRED PATHS $ENV{HOME}/.cargo/bin)
message(STATUS "Using cxxbridge: ${CXXBRIDGE}")

set(cxxbridge_in ${CMAKE_SOURCE_DIR}/src/lib.rs)
set(cxxbridge_source_out ${CMAKE_SOURCE_DIR}/../../src/operator_wrapper.cpp)
set(cxxbridge_header_out ${CMAKE_SOURCE_DIR}/../../include/operator_wrapper.hpp)
set(operator_wrapper_lib ${CMAKE_SOURCE_DIR}/target/release/liboperator_wrapper.a)

add_custom_target(CxxBridge ALL DEPENDS ${cxxbridge_source_out} ${cxxbridge_header_out})
add_custom_command(
  OUTPUT ${cxxbridge_header_out} ${cxxbridge_source_out}
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --output ${cxxbridge_source_out}
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --header --output ${cxxbridge_header_out}
  COMMENT "Generating CXX bridge: ${cxxbridge_source_out} ${cxxbridge_header_out}"
  DEPENDS ${cxxbridge_in}
  )
