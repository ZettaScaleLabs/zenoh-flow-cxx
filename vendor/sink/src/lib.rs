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
use zenoh_flow::{
    runtime::deadline::E2EDeadlineMiss, runtime::message::DataMessage, Configuration, Context,
    Node, Sink, State, ZFError, ZFResult, ZFState,
};

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

    /// A Zenoh Flow Input data.
    ///
    /// It contains:
    /// - `port_id` the port id from where the data was received.
    /// - `data` as std::vector<uint8_t>.
    /// - `timestamp` an uHLC timestamp associated with the data.
    /// - `e2d_deadline_miss` list of `E2EDeadlineMiss`.
    #[derive(Debug)]
    pub struct Input {
        pub data: Vec<u8>,
        pub timestamp: u64,
        pub e2d_deadline_miss: Vec<E2EDeadlineMiss>,
    }

    /// A End to End Deadline.
    /// A deadline can apply for a whole graph or for a subpart of it.
    #[derive(Debug)]
    pub struct E2EDeadlineMiss {
        pub from: OutputDescriptor,
        pub to: InputDescriptor,
        pub start: u64,
        pub end: u64,
    }

    /// Describes one output
    ///
    /// Example:
    ///
    /// ```yaml
    /// node : Counter
    /// output : Counter
    /// ```
    ///
    #[derive(Debug)]
    pub struct OutputDescriptor {
        pub node: String,
        pub output: String,
    }

    /// Describes one input
    ///
    /// Example:
    ///
    /// ```yaml
    /// node : SumOperator
    /// input : Number
    /// ```
    #[derive(Debug)]
    pub struct InputDescriptor {
        pub node: String,
        pub input: String,
    }

    unsafe extern "C++" {
        include!("sink.hpp");

        /// This type abstracts the user's state type inside Zenoh Flow.
        ///
        type State;

        /// This method is used to initialize the state of the node.
        /// It is called by the Zenoh Flow runtime when initializing the data flow
        /// graph.
        /// An example of node state is files that should be opened, connection
        /// to devices or internal configuration.
        fn initialize(json_configuration: &str) -> UniquePtr<State>;

        /// This method is the actual one consuming the data.
        /// It is triggered whenever data arrives on the Sink input.
        /// This method is `async` therefore I/O is possible, e.g. writing to
        /// a file or interacting with an external device.
        ///
        /// The Sink can access its state and context while executing,
        fn run(context: &mut Context, state: &mut UniquePtr<State>, input: Input) -> Result<()>;
    }
}

/*
 *
 * Zenoh Flow glue.
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

impl ffi::Input {
    fn from_data_message(
        data_message: &mut zenoh_flow::runtime::message::DataMessage,
    ) -> ZFResult<Self> {
        let data = data_message
            .get_inner_data()
            .try_as_bytes()?
            .as_ref()
            .clone();
        let e2d_deadline_miss: Vec<ffi::E2EDeadlineMiss> = data_message
            .get_missed_end_to_end_deadlines()
            .iter()
            .map(|e2e_deadline| e2e_deadline.into())
            .collect();

        Ok(Self {
            data,
            timestamp: data_message.get_timestamp().get_time().as_u64(),
            e2d_deadline_miss,
        })
    }
}

impl From<&E2EDeadlineMiss> for ffi::E2EDeadlineMiss {
    fn from(e2d_deadline_miss: &E2EDeadlineMiss) -> Self {
        let to = ffi::InputDescriptor {
            node: (*e2d_deadline_miss.to.node.as_ref()).into(),
            input: (*e2d_deadline_miss.to.input.as_ref()).into(),
        };
        let from = ffi::OutputDescriptor {
            node: (*e2d_deadline_miss.from.node.as_ref()).into(),
            output: (*e2d_deadline_miss.from.output.as_ref()).into(),
        };

        Self {
            from,
            to,
            start: e2d_deadline_miss.start.get_time().as_u64(),
            end: e2d_deadline_miss.end.get_time().as_u64(),
        }
    }
}

/*
 *
 * CxxSink implementation.
 *
 */

pub struct CxxSink;

impl Node for CxxSink {
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
impl Sink for CxxSink {
    async fn run(
        &self,
        context: &mut Context,
        dyn_state: &mut State,
        mut input: DataMessage,
    ) -> ZFResult<()> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        let cxx_input = ffi::Input::from_data_message(&mut input)?;

        {
            let cxx_output_res: ZFResult<()> = async {
                #[allow(unused_unsafe)]
                unsafe {
                    ffi::run(&mut cxx_context, &mut wrapper.state, cxx_input)
                        .map_err(|_| ZFError::GenericError)
                }
            }
            .await;
            let cxx_output = cxx_output_res?;
            Ok(cxx_output)
        }
    }
}

zenoh_flow::export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(CxxSink) as Arc<dyn Sink>)
}
