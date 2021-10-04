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

message(STATUS "Generating ${cxxbridge_in}")

# NOTE: the order is important!
set(1_header_in          ${include_dir}/${component}-header.in)
set(2_shared_structs_in  ${include_dir}/shared-structs.in)
set(3_cpp_interface_in   ${include_dir}/${component}-cpp-interface.in)
set(4_zenoh_flow_glue_in ${include_dir}/zenoh-flow-glue.in)
set(5_implementation_in  ${include_dir}/${component}-impl.in)

# Create the list of files to concatenate in order to produce lib/src.rs.
set(lib_src_parts
  ${1_header_in}
  ${2_shared_structs_in}
  ${3_cpp_interface_in}
  ${4_zenoh_flow_glue_in}
  ${5_implementation_in})

# Empties the content of ${cxxbridge_in}.
file(WRITE ${cxxbridge_in} "")

# Appends the contents of each file and generate `src/lib.rs`.
foreach(part ${lib_src_parts})
  file(READ ${part} contents)
  file(APPEND ${cxxbridge_in} "${contents}")
endforeach()

message(STATUS "Generating ${cxxbridge_in} - done")
