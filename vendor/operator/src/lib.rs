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

use cxx::UniquePtr;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use zenoh_flow::{
    runtime::deadline::E2EDeadlineMiss, Configuration, Data, InputToken, LocalDeadlineMiss, Node,
    NodeOutput, Operator, PortId, State, TokenAction, ZFError, ZFResult, ZFState,
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
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
        pub e2d_deadline_miss: Vec<E2EDeadlineMiss>,
    }

    /// A Zenoh Flow Output data.
    ///
    /// It contains:
    /// - `port_id` the port where the data will be sent.
    /// - `data` as std::vector<uint8_t>.
    #[derive(Debug)]
    pub struct Output {
        pub port_id: String,
        pub data: Vec<u8>,
    }

    /// The status of a token representing the input.
    /// It can be either containing the data or the information the data is
    /// still pending.
    #[derive(Debug)]
    pub enum TokenStatus {
        Pending,
        Ready,
    }

    /// The action that can be executed on a token.
    /// Once the Token is created with some data inside,
    /// different actions could be executed by the input rules.
    ///
    /// - Consume (default) the data will be consumed when run is triggered
    /// - Drop the data will be dropped
    /// - Keep the data will be kept for the current and the next
    /// time the run is triggered, if can be set back to `Consume` by the user.
    #[derive(Debug)]
    pub enum TokenAction {
        Consume,
        Drop,
        Keep,
        Wait,
    }
    /// A structure containing all the information regarding a missed, local, deadline.
    ///
    /// - `is_set` is the deadline is set.
    /// - `deadline_duration_ms`: the duration of the deadline.
    /// - `elapsed_ms`: the duration of the execution.
    #[derive(Debug)]
    pub struct LocalDeadlineMiss {
        pub elapsed_ms: u64,
        pub deadline_duration_ms: u64,
        pub is_set: bool,
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

    /// The token representing the input.
    #[derive(Debug)]
    pub struct InputToken {
        pub status: TokenStatus,
        pub action: TokenAction,
        pub port_id: String,
        pub data: Vec<u8>,
        pub timestamp: u64,
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
        include!("operator.hpp");
        /// This type abstracts the user's state type inside Zenoh Flow.
        ///
        type State;

        /// This method is used to initialize the state of the node.
        /// It is called by the Zenoh Flow runtime when initializing the data flow
        /// graph.
        /// An example of node state is files that should be opened, connection
        /// to devices or internal configuration.
        fn initialize(json_configuration: &str) -> UniquePtr<State>;

        /// This method is called when data is received on one or more inputs.
        /// The result of this method is use as discriminant to trigger the
        /// operator's run function.
        /// The operator can access to its context and its state during execution.
        ///
        /// The received data is provided as [`InputToken`](`InputToken`) that
        /// represent the state of the associated port.
        /// Based on the tokens and on the data users can decide if trigger
        /// the run or not.
        fn input_rule(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            tokens: &mut Vec<InputToken>,
        ) -> Result<bool>;

        /// This method is the actual one processing the data.
        /// It is triggered based on the result of the `input_rule`.
        /// As operators are computing over data,
        /// *I/O should not be done in the run*.
        ///
        /// The operator can access to its context and its state during execution.
        /// The result of a computation can also not provide any output.
        /// When it does provide output the `PortId` used should match the one
        /// defined in the descriptor for the operator. Any not matching `PortId`
        /// will be dropped.
        fn run(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            inputs: Vec<Input>,
        ) -> Result<Vec<Output>>;

        /// This method is called after the run, and can be used for
        /// further analysis and adjustment over the computed data.
        /// E.g. flooring a value to a specified MAX, or check if it is within
        /// a given range.
        ///
        fn output_rule(
            context: &mut Context,
            state: &mut UniquePtr<State>,
            outputs: Vec<Output>,
            deadline_miss: LocalDeadlineMiss,
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

impl ffi::InputToken {
    pub fn try_new(token: InputToken, port_id: &str) -> ZFResult<Self> {
        match token {
            InputToken::Pending => Ok(Self {
                status: ffi::TokenStatus::Pending,
                action: ffi::TokenAction::Wait,
                port_id: port_id.to_string(),
                data: Vec::new(),
                timestamp: 0,
            }),

            InputToken::Ready(mut token) => {
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
    fn try_new(port_id: &str, data_message: &mut zenoh_flow::DataMessage) -> ZFResult<Self> {
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
            port_id: port_id.to_string(),
            data,
            timestamp: data_message.get_timestamp().get_time().as_u64(),
            e2d_deadline_miss,
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

impl From<Option<LocalDeadlineMiss>> for ffi::LocalDeadlineMiss {
    fn from(deadline_miss: Option<LocalDeadlineMiss>) -> Self {
        match deadline_miss {
            Some(deadline_miss) => Self {
                elapsed_ms: (deadline_miss.elapsed.as_secs_f64() * 1_000_000.0).floor() as u64,
                deadline_duration_ms: (deadline_miss.deadline.as_secs_f64() * 1_000_000.0).floor()
                    as u64,
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
 * CxxOperator.
 *
 */

pub struct CxxOperator;

impl Node for CxxOperator {
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

impl Operator for CxxOperator {
    fn input_rule(
        &self,
        context: &mut zenoh_flow::Context,
        dyn_state: &mut State,
        tokens: &mut HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>,
    ) -> zenoh_flow::ZFResult<bool> {
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        // let res_cxx_tokens: ZFResult<Vec<ffi::InputToken>> = tokens
        //     .iter_mut()
        //     .map(|(port_id, token)| ffi::InputToken::try_new(token, port_id))
        //     .collect();
        // let mut cxx_tokens = res_cxx_tokens?;

        let mut cxx_tokens: Vec<ffi::InputToken> = Vec::with_capacity(tokens.len());
        for (port_id, token) in tokens.iter_mut() {
            let old = std::mem::replace(token, InputToken::Pending);
            cxx_tokens.push(ffi::InputToken::try_new(old, port_id)?);
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
        deadline_miss: Option<LocalDeadlineMiss>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, zenoh_flow::NodeOutput>> {
        let mut cxx_context = ffi::Context::from(context);
        let wrapper = dyn_state.try_get::<StateWrapper>()?;
        let res_run_outputs: ZFResult<Vec<ffi::Output>> = outputs
            .iter_mut()
            .map(|(port_id, data)| ffi::Output::try_new(port_id, data))
            .collect();
        let run_outputs = res_run_outputs?;
        let deadline_miss = ffi::LocalDeadlineMiss::from(deadline_miss);
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
