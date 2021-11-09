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

use async_trait::async_trait;
use cxx::UniquePtr;
use std::{fmt::Debug, sync::Arc};
use zenoh_flow::{
    runtime::message::DataMessage, Configuration, Context, Node, Sink, State, ZFError, ZFResult,
    ZFState,
};

extern crate zenoh_flow;

#[cxx::bridge(namespace = "zenoh::flow")]
pub mod ffi {
    pub struct Context {
        pub mode: usize,
    }

    pub struct Configuration {
        pub key: String,
        pub value: String,
    }

    pub struct Input {
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    unsafe extern "C++" {
        include!("sink.hpp");

        type State;

        fn initialize(configuration: &Vec<Configuration>) -> UniquePtr<State>;

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
        data_message: &zenoh_flow::runtime::message::DataMessage,
    ) -> ZFResult<Self> {
        let data = data_message.data.try_as_bytes()?.as_ref().clone();

        Ok(Self {
            data,
            timestamp: data_message.timestamp.get_time().as_u64(),
        })
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
                Some(config) => {
                    let mut conf = vec![];
                    for (key, value) in config {
                        let entry = ffi::Configuration {
                            key: key.clone(),
                            value: value
                                .as_str()
                                .ok_or_else(|| ZFError::GenericError)?
                                .to_string(),
                        };
                        conf.push(entry);
                    }
                    conf
                }
                None => vec![],
            },

            None => vec![],
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
        input: DataMessage,
    ) -> ZFResult<()> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        let cxx_input = ffi::Input::from_data_message(&input)?;

        {
            #[allow(unused_unsafe)]
            unsafe {
                Ok(ffi::run(&mut cxx_context, &mut wrapper.state, cxx_input)
                    .map_err(|_| ZFError::GenericError)?)
            }
        }
    }
}

zenoh_flow::export_sink!(register);

fn register() -> ZFResult<Arc<dyn Sink>> {
    Ok(Arc::new(CxxSink) as Arc<dyn Sink>)
}
