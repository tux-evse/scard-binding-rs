/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::prelude::*;
use afbv4::prelude::*;

pub struct ScardHandle {
    pcsc: PcscClient,
    ctrl: Option<*mut dyn ScardControl>,
}

pub type ScardAction = PcscAction;
pub type ScardCmd = PcscCmd;

pub enum MonitorAction {
    START,
    STOP,
}

pub enum ScardState {
    PRESENT,
    EMPTY,
    UNKNOWN,
}

impl ScardHandle {
    // prepare handle for open operation
    pub fn new(jconf: JsoncObj, verbosity: i32, callback: Option<*mut dyn PcscControl>) -> Result<ScardHandle, AfbError> {
        let pcsc = PcscClient::new(jconf, verbosity, callback)?;

        let handle= ScardHandle { pcsc, ctrl: None };
        Ok(handle)
    }

    // register background callback to reader TBD
    pub fn connect(&self) -> Result<&Self, AfbError> {
        Ok(self)
    }

    pub fn reader_name(&self) -> &'static str {
        self.pcsc.get_reader_name()
    }

    pub fn get_uuid(&self) -> Result<u64, AfbError> {
        self.pcsc.reader_check()?;
        self.pcsc.get_uuid()
    }

    pub fn get_cmd_by_uid(&self, cuid: &str) -> Result<ScardCmd, AfbError> {
        self.pcsc.get_cmd_by_uid(cuid)
    }

    pub fn read_data(&self, cmd: &PcscCmd) -> Result<String, AfbError> {
        self.get_uuid()?;
        self.pcsc.get_data(cmd)
    }

    pub fn write_data(&self, cmd: &PcscCmd, data: &[u8]) -> Result<(), AfbError> {
        self.get_uuid()?;
        self.pcsc.set_data(cmd, data)
    }

    pub fn set_callback(&mut self, ctrlbox: Box<dyn ScardControl>) -> &mut Self {
        self.ctrl = Some(Box::leak(ctrlbox));
        self
    }

    pub fn monitor(&self, action:MonitorAction) -> Result<u64, AfbError> {

        let tid= match action {
            MonitorAction::START => {
                self.pcsc.monitor_start()?
             },
            MonitorAction::STOP => {
                self.pcsc.monitor_stop(0)?;
                0
            },
        };

        Ok(tid)
    }
}

pub trait ScardControl {
    fn scard_monitor(&mut self, status: ScardState);
}
