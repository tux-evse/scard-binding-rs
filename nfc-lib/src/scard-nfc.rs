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
}


impl ScardHandle {
    // prepare handle for open operation
    pub fn new(jconf: JsoncObj, verbosity: i32) -> Result<ScardHandle, AfbError> {
        let pcsc= PcscClient::new (jconf, verbosity) ?;
        Ok(ScardHandle{pcsc})
    }

    // register background callback to reader TBD
    pub fn connect(&self) -> Result<&Self, AfbError> {
        Ok(self)
    }

    pub fn reader_name (&self) -> &'static str {
        self.pcsc.get_reader_name()
    }

    pub fn get_uuid(&self) -> Result <u64, AfbError> {
        self.pcsc.reader_check()?;
        self.pcsc.get_uuid()
    }

    pub fn read_data(&self, cuid: &str) -> Result <String, AfbError> {
        self.pcsc.reader_check()?;
        self.pcsc.get_data(cuid)
    }

    pub fn send_data(&self, cuid: &str, data: &[u8]) -> Result <(), AfbError> {
        self.pcsc.reader_check()?;
        self.pcsc.set_data(cuid, data)
    }

}

