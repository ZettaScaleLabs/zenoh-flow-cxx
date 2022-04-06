#
# Copyright (c) 2022 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#

cmake_minimum_required(VERSION 3.10)

project(ZenohFlowCxxNode VERSION 0.2.0)

# TODO: Detect that we are running on a Apple M1 and set this variable to this value.
# https://gitlab.kitware.com/cmake/cmake/-/issues/20989
# set(CMAKE_OSX_ARCHITECTURES arm64)
# message("-- CMAKE_OSX_ARCHITECTURES = ${CMAKE_OSX_ARCHITECTURES}")

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
option(SOURCE    "Set the Node to SOURCE")
option(OPERATOR  "Set the Node to OPERATOR")
option(SINK      "Set the Node to SINK")
option(BUILD_DOC "Build documentation")


if (NOT ${SOURCE} AND NOT ${OPERATOR} AND NOT ${SINK})
  message(FATAL_ERROR "The type of Node to generate must be specified (i.e., -DOPERATOR=ON, -DSINK=ON, -DSOURCE=ON).")
elseif ((${SOURCE} AND (${OPERATOR} OR ${SINK})) OR (${OPERATOR} AND (${SOURCE} OR ${SINK})))
  message(FATAL_ERROR "Only one type of Node can be generated.")
endif()

if (${SOURCE})
  set(node source)
elseif (${SINK})
  set(node sink)
elseif (${OPERATOR})
  set(node operator)
endif()



set(LIB_NAME "cxx_${node}" CACHE STRING "The name of the generated library.")

#
# First step: generating the bridge files.
#
# The generated lib.rs file will be parsed by `cxxbridge` to generate the
# bindings between Zenoh Flow and the C++ nodes.
#
set(node_dir  ${CMAKE_SOURCE_DIR}/vendor/${node})

set(cxxbridge_in         ${node_dir}/src/lib.rs)
set(cxxbridge_source_out ${CMAKE_SOURCE_DIR}/src/wrapper.cpp)
set(cxxbridge_header_out ${CMAKE_SOURCE_DIR}/include/wrapper.hpp)

add_custom_target(CxxBridge ALL
  DEPENDS ${cxxbridge_header_out} ${cxxbridge_source_out}
  )

add_custom_command(
  OUTPUT ${cxxbridge_header_out} ${cxxbridge_source_out}
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --output ${cxxbridge_source_out}
  COMMAND ${CXXBRIDGE} ${cxxbridge_in} --header --output ${cxxbridge_header_out}
  COMMAND ${CMAKE_COMMAND} -DHEADER=${cxxbridge_header_out} -P ${CMAKE_SCRIPTS_DIR}/Patcher.cmake
  COMMENT "Generating CXX bridge:"
  )


#
# Generating the doc if the option BUILD_DOC is set
#
#
if (${BUILD_DOC})
  find_package(Doxygen)
  if (DOXYGEN_FOUND)
      # set input and output files
      set(DOXYGEN_IN ${CMAKE_SOURCE_DIR}/docs/Doxyfile.in)
      set(DOXYGEN_OUT ${CMAKE_CURRENT_BINARY_DIR}/Doxyfile)

      # request to configure the file
      configure_file(${DOXYGEN_IN} ${DOXYGEN_OUT} @ONLY)
      message("Doxygen build started")

      # note the option ALL which allows to build the docs together with the application
      add_custom_target( docs ALL
          COMMAND ${DOXYGEN_EXECUTABLE} ${DOXYGEN_OUT}
          WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
          COMMENT "Generating API documentation with Doxygen"
          VERBATIM )
  else (DOXYGEN_FOUND)
    message("Doxygen need to be installed to generate the doxygen documentation")
  endif (DOXYGEN_FOUND)
endif()

#
# Second step: generating the static library containing the Rust "glue".
#
# This part is actually called from the top-level CMakeLists file. The static
# library will be linked with the resulting node shared library.
set(rust_lib ${node_dir}/target/${CARGO_BUILD_TYPE_DIR}/lib${node}.a)

add_custom_target(RustLib ALL DEPENDS ${rust_lib})
add_custom_command(
  OUTPUT ${rust_lib}
  COMMAND cargo update
  COMMAND cargo build ${CARGO_BUILD_TYPE}
  WORKING_DIRECTORY ${node_dir}
  DEPENDS ${cxxbridge_in}
  COMMENT "Generating Rust library:"
  )

add_library(CxxWrapper STATIC IMPORTED GLOBAL)
add_dependencies(CxxWrapper RustLib)
set_target_properties(CxxWrapper
  PROPERTIES
  IMPORTED_LOCATION ${rust_lib}
  )

#
# Third step: generating the final node shared library.
#
set(node_src src/${node}.cpp)

add_library(${LIB_NAME} SHARED
  ${node_src}
  ${cxxbridge_source_out})
add_dependencies(${LIB_NAME} CxxBridge)
target_include_directories(${LIB_NAME} PUBLIC ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(${LIB_NAME} PUBLIC CxxWrapper)
