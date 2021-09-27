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

project(ZenohFlowCxxOperator VERSION 0.1.0)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

set(CMAKE_CXX_FLAGS "-Wall -Wextra")
set(CMAKE_CXX_FLAGS_DEBUG "-g")
set(CMAKE_CXX_FLAGS_RELEASE "-O3")

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(target_lib "cxx_operator")
set(operator_src "src/operator.cpp")
set(cxxbridge_header_out ${CMAKE_SOURCE_DIR}/include/operator_wrapper.hpp)
set(cxxbridge_source_out ${CMAKE_SOURCE_DIR}/src/operator_wrapper.cpp)
set(cxxwrapper_lib ${CMAKE_SOURCE_DIR}/vendor/operator-wrapper/target/release/liboperator_wrapper.a)

file(READ ${cxxbridge_header_out} header_error)
string(REPLACE "using State = ::zenoh::flow::State;"
  ""  header_ok
  "${header_error}")
file(WRITE ${cxxbridge_header_out} "${header_ok}")

add_custom_target(wrapper_target ALL DEPENDS ${cxxwrapper_lib})
add_custom_command(
  OUTPUT ${cxxwrapper_lib}
  COMMAND cargo build --release
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/vendor/operator-wrapper
  )

add_library(CxxWrapper STATIC IMPORTED GLOBAL)
add_dependencies(CxxWrapper wrapper_target)
set_target_properties(CxxWrapper
  PROPERTIES
  IMPORTED_LOCATION ${cxxwrapper_lib}
  )

add_library(${target_lib} SHARED
  ${operator_src}
  ${cxxbridge_source_out})
target_include_directories(${target_lib} PUBLIC ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(${target_lib} CxxWrapper)
