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

use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    Configuration, Data, DeadlineMiss, Node, NodeOutput, Operator, PortId, State, Token,
    TokenAction, ZFError, ZFResult, ZFState,
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
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    pub struct Output {
        pub port_id: String,
        pub data: Vec<u8>,
    }

    pub enum TokenStatus {
        Pending,
        Ready,
    }

    pub enum TokenAction {
        Consume,
        Drop,
        Keep,
        Wait,
    }

    pub struct DeadlineMiss {
        pub elapsed_ms: u64,
        pub deadline_duration_ms: u64,
        pub is_set: bool,
    }

    pub struct Token {
        pub status: TokenStatus,
        pub action: TokenAction,
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
    }

    unsafe extern "C++" {
        include!("operator.hpp");

        type State;

        fn initialize(configuration: &Vec<Configuration>) -> UniquePtr<State>;

        fn input_rule(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            tokens: &mut Vec<Token>,
        ) -> Result<bool>;

        fn run(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            inputs: Vec<Input>,
        ) -> Result<Vec<Output>>;

        fn output_rule(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            outputs: Vec<Output>,
            deadline_miss: DeadlineMiss,
        ) -> Result<Vec<Output>>;
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

impl ffi::Token {
    pub fn try_new(token: Token, port_id: &str) -> ZFResult<Self> {
        match token {
            Token::Pending => Ok(Self {
                status: ffi::TokenStatus::Pending,
                action: ffi::TokenAction::Wait,
                port_id: port_id.to_string(),
                data: Vec::new(),
                timestamp: 0,
            }),

            Token::Ready(mut token) => {
                let data = token.get_data_mut().try_as_bytes()?.as_ref().clone();

                Ok(Self {
                    status: ffi::TokenStatus::Ready,
                    action: ffi::TokenAction::Consume,
                    port_id: port_id.to_string(),
                    data,
                    timestamp: token.get_timestamp().get_time().as_u64(),
                })
            }
        }
    }
}

impl From<TokenAction> for ffi::TokenAction {
    fn from(action: TokenAction) -> Self {
        match action {
            TokenAction::Consume => ffi::TokenAction::Consume,
            TokenAction::Drop => ffi::TokenAction::Drop,
            TokenAction::Keep => ffi::TokenAction::Keep,
        }
    }
}

impl ffi::Input {
    fn try_new(port_id: &str, data_message: &zenoh_flow::DataMessage) -> ZFResult<Self> {
        let data = data_message.data.try_as_bytes()?.as_ref().clone();

        Ok(Self {
            port_id: port_id.to_string(),
            data,
            timestamp: data_message.timestamp.get_time().as_u64(),
        })
    }
}

impl ffi::Output {
    fn try_new(port_id: &str, data: &zenoh_flow::Data) -> ZFResult<Self> {
        Ok(Self {
            port_id: port_id.to_string(),
            data: data.try_as_bytes()?.as_ref().clone(),
        })
    }
}

impl From<Option<DeadlineMiss>> for ffi::DeadlineMiss {
    fn from(deadline_miss: Option<DeadlineMiss>) -> Self {
        match deadline_miss {
            Some(deadline_miss) => Self {
                elapsed_ms: (deadline_miss.elapsed.as_secs_f64() * 1_000_000 as f64).floor() as u64,
                deadline_duration_ms: (deadline_miss.deadline.as_secs_f64() * 1_000_000 as f64)
                    .floor() as u64,
                is_set: true,
            },
            None => Self {
                elapsed_ms: 0,
                deadline_duration_ms: 0,
                is_set: false,
            },
        }
    }
}

/*
 *
 * CxxOperator.
 *
 */

pub struct CxxOperator;

impl Node for CxxOperator {
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

impl Operator for CxxOperator {
    fn input_rule(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut State,
        tokens: &mut HashMap<zenoh_flow::PortId, zenoh_flow::Token>,
    ) -> zenoh_flow::ZFResult<bool> {
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        // let res_cxx_tokens: ZFResult<Vec<ffi::Token>> = tokens
        //     .iter_mut()
        //     .map(|(port_id, token)| ffi::Token::try_new(token, port_id))
        //     .collect();
        // let mut cxx_tokens = res_cxx_tokens?;

        let mut cxx_tokens: Vec<ffi::Token> = Vec::with_capacity(tokens.len());
        for (port_id, token) in tokens.iter_mut() {
            let old = std::mem::replace(token, Token::Pending);
            cxx_tokens.push(ffi::Token::try_new(old, port_id)?);
        }

        let mut cxx_context = ffi::Context::from(context);

        {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::input_rule(&mut cxx_context, &mut wrapper.state, &mut cxx_tokens)
                    .map_err(|_| ZFError::GenericError)
            }
        }
    }

    fn run(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut zenoh_flow::State,
        inputs: &mut HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, Data>> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        let result_cxx_inputs: ZFResult<Vec<ffi::Input>> = inputs
            .iter_mut()
            .map(|(port_id, data_message)| ffi::Input::try_new(port_id, data_message))
            .collect();
        let cxx_inputs = result_cxx_inputs?;

        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::run(&mut cxx_context, &mut wrapper.state, cxx_inputs)
                    .map_err(|_| ZFError::GenericError)?
            }
        };

        let mut result: HashMap<zenoh_flow::PortId, Data> =
            HashMap::with_capacity(cxx_outputs.len());
        for cxx_output in cxx_outputs.into_iter() {
            result.insert(cxx_output.port_id.into(), Data::from_bytes(cxx_output.data));
        }

        Ok(result)
    }

    fn output_rule(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut State,
        mut outputs: HashMap<zenoh_flow::PortId, Data>,
        deadline_miss: Option<DeadlineMiss>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, zenoh_flow::NodeOutput>> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        let res_run_outputs: ZFResult<Vec<ffi::Output>> = outputs
            .iter_mut()
            .map(|(port_id, data)| ffi::Output::try_new(port_id, data))
            .collect();
        let run_outputs = res_run_outputs?;
        let deadline_miss = ffi::DeadlineMiss::from(deadline_miss);
        let cxx_outputs = {
            #[allow(unused_unsafe)]
            unsafe {
                ffi::output_rule(
                    &mut cxx_context,
                    &mut wrapper.state,
                    run_outputs,
                    deadline_miss,
                )
                .map_err(|_| ZFError::GenericError)?
            }
        };

        let mut results: HashMap<PortId, NodeOutput> = HashMap::with_capacity(outputs.len());
        // NOTE: default output rule for now.
        for output in cxx_outputs.into_iter() {
            results.insert(
                output.port_id.into(),
                NodeOutput::Data(Data::from_bytes(output.data)),
            );
        }

        Ok(results)
    }
}

zenoh_flow::export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(CxxOperator) as Arc<dyn Operator>)
}
