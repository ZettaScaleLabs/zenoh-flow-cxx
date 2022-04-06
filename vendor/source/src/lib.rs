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

use async_trait::async_trait;
use cxx::UniquePtr;
use std::{fmt::Debug, sync::Arc};
use zenoh_flow::{Configuration, Context, Data, Node, Source, State, ZFError, ZFResult, ZFState};

extern crate zenoh_flow;

#[cxx::bridge(namespace = "zenoh::flow")]
pub mod ffi {

    /// Context is a structure provided by Zenoh Flow to access
    /// the execution context directly from the nodes.
    ///
    /// It contains the `mode` as size_t.
    pub struct Context {
        pub mode: usize,
    }

    unsafe extern "C++" {
        include!("source.hpp");

        /// This type abstracts the user's state type inside Zenoh Flow.
        ///
        type State;

        /// This method is used to initialize the state of the node.
        /// It is called by the Zenoh Flow runtime when initializing the data flow
        /// graph.
        /// An example of node state is files that should be opened, connection
        /// to devices or internal configuration.
        fn initialize(json_configuration: &str) -> UniquePtr<State>;

        /// This method is the actual one producing the data.
        /// It is triggered on a loop, and if the `period` is specified
        /// in the descriptor it is triggered with the given period.
        /// This method is `async` therefore I/O is possible, e.g. reading data
        /// from a file/external device.
        ///
        /// The Source can access its state and context while executing,
        fn run(context: &mut Context, state: &mut UniquePtr<State>) -> Result<Vec<u8>>;
    }
}

/*
 *
 * Zenoh-flow glue.
 *
 */

unsafe impl Send for ffi::State {}
unsafe impl Sync for ffi::State {}

pub struct StateWrapper {
    pub state: UniquePtr<ffi::State>,
}

impl ZFState for StateWrapper {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Debug for StateWrapper {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<&mut zenoh_flow::Context> for ffi::Context {
    fn from(context: &mut zenoh_flow::Context) -> Self {
        Self { mode: context.mode }
    }
}

/*
 *
 * CxxSource implementation.
 *
 */

pub struct CxxSource;

impl Node for CxxSource {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        let cxx_configuration = match configuration {
            Some(config) => match config.as_object() {
                Some(config) => serde_json::to_string(config)?,
                None => String::from("{}"),
            },

            None => String::from("{}"),
        };

        let state = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::initialize(&cxx_configuration)
            }
        };
        Ok(State::from(StateWrapper { state }))
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

#[async_trait]
impl Source for CxxSource {
    async fn run(&self, context: &mut Context, dyn_state: &mut State) -> ZFResult<Data> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;

        let cxx_output_res: ZFResult<Vec<u8>> = async {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state).map_err(|_| ZFError::GenericError)
            }
        }
        .await;
        let cxx_output = cxx_output_res?;
        Ok(Data::from_bytes(cxx_output))
    }
}

zenoh_flow::export_source!(register);

fn register() -> ZFResult<Arc<dyn Source>> {
    Ok(Arc::new(CxxSource) as Arc<dyn Source>)
}
