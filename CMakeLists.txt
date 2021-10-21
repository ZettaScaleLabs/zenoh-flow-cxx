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

project(ZenohFlowCxxComponent VERSION 0.1.0)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
  set(CARGO_BUILD_TYPE --release)
  set(CARGO_BUILD_TYPE_DIR release)
endif()

message("-- CMAKE_BUILD_TYPE = ${CMAKE_BUILD_TYPE}")

string( TOLOWER "${CMAKE_BUILD_TYPE}" cm_build_type )
if(cm_build_type STREQUAL "debug" )
  set(CARGO_BUILD_TYPE_DIR debug)
endif()

message("-- CARGO_BUILD_TYPE = ${CARGO_BUILD_TYPE_DIR}")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_FLAGS "-Wall -Wextra")
set(CMAKE_CXX_FLAGS_DEBUG "-g")
set(CMAKE_CXX_FLAGS_RELEASE "-O3")

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# TODO: Windows compatibility.
find_program(CXXBRIDGE cxxbridge REQUIRED PATHS $ENV{HOME}/.cargo/bin)
message(STATUS "Using cxxbridge: ${CXXBRIDGE}")

set(CMAKE_SCRIPTS_DIR ${CMAKE_SOURCE_DIR}/cmake)

#
# Setup: We need to know what we are generating.
#
option(SOURCE    "Set the Component to SOURCE")
option(OPERATOR  "Set the Component to OPERATOR")
option(SINK      "Set the Component to SINK")


if (NOT ${SOURCE} AND NOT ${OPERATOR} AND NOT ${SINK})
  message(FATAL_ERROR "The type of Component to generate must be specified (i.e., -DOPERATOR=ON, -DSINK=ON, -DSOURCE=ON).")
elseif ((${SOURCE} AND (${OPERATOR} OR ${SINK})) OR (${OPERATOR} AND (${SOURCE} OR ${SINK})))
  message(FATAL_ERROR "Only one type of Component can be generated.")
endif()

if (${SOURCE})
  set(component source)
elseif (${SINK})
  set(component sink)
elseif (${OPERATOR})
  set(component operator)
endif()

set(LIB_NAME "cxx_${component}" CACHE STRING "The name of the generated library.")

#
# First step: generating the bridge files.
#
# The generated lib.rs file will be parsed by `cxxbridge` to generate the
# bindings between Zenoh Flow and the C++ components.
#
set(wrapper_dir  ${CMAKE_SOURCE_DIR}/vendor/wrapper)
set(include_dir  ${wrapper_dir}/include)

set(cxxbridge_in         ${wrapper_dir}/src/lib.rs)
set(cxxbridge_source_out ${CMAKE_SOURCE_DIR}/src/wrapper.cpp)
set(cxxbridge_header_out ${CMAKE_SOURCE_DIR}/include/wrapper.hpp)

add_custom_target(CxxBridge ALL
  DEPENDS ${cxxbridge_in} ${cxxbridge_header_out} ${cxxbridge_source_out}
  )

add_custom_command(
  OUTPUT ${cxxbridge_in} ${cxxbridge_header_out} ${cxxbridge_source_out}
  COMMAND ${CMAKE_COMMAND} -Dcomponent=${component} -Dinclude_dir=${include_dir} -Dcxxbridge_in=${cxxbridge_in} -P ${CMAKE_SCRIPTS_DIR}/GenerateLibRs.cmake
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --output ${cxxbridge_source_out}
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --header --output ${cxxbridge_header_out}
  COMMAND ${CMAKE_COMMAND} -DHEADER=${cxxbridge_header_out} -P ${CMAKE_SCRIPTS_DIR}/Patcher.cmake
  COMMENT "Generating CXX bridge:"
  )

#
# Second step: generating the static library containing the Rust "glue".
#
# This part is actually called from the top-level CMakeLists file. The static
# library will be linked with the resulting component shared library.
set(cxxwrapper_lib ${wrapper_dir}/target/${CARGO_BUILD_TYPE_DIR}/libwrapper.a)

add_custom_target(WrapperLib ALL DEPENDS ${cxxwrapper_lib})
add_custom_command(
  OUTPUT ${cxxwrapper_lib}
  # COMMAND cargo update
  COMMAND cargo build ${CARGO_BUILD_TYPE}
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/vendor/wrapper
  DEPENDS ${cxxbridge_in}
  COMMENT "Generating Rust library:"
  )

add_library(CxxWrapper STATIC IMPORTED GLOBAL)
add_dependencies(CxxWrapper WrapperLib)
set_target_properties(CxxWrapper
  PROPERTIES
  IMPORTED_LOCATION ${cxxwrapper_lib}
  )

#
# Third step: generating the final component shared library.
#
set(component_src src/${component}.cpp)

add_library(${LIB_NAME} SHARED
  ${component_src}
  ${cxxbridge_source_out})
add_dependencies(${LIB_NAME} CxxBridge)
target_include_directories(${LIB_NAME} PUBLIC ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(${LIB_NAME} PUBLIC CxxWrapper)
