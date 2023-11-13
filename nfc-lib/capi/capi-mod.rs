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

use afbv4::prelude::*;
use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;

pub mod cglue {
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    // import AfbJson do not reimplement a new version
    use afbv4::prelude::JsoncJso;
    type json_object = JsoncJso;
    #[repr(C)]
    // map pscsc abstract pointer should remains allocate within C only
    pub struct pcscCmdT {
        not_to_be_used: u32,
    }
    include!("_capi-map.rs");
}

pub struct PcscClient {
    config: *mut cglue::pcscConfigT,
    handle: Cell<*mut cglue::pcscHandleT>,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PcscsOption {
    UNKNOWN = cglue::pcscOptsE_PCSC_OPT_UNKNOWN,
    TIMEOUT = cglue::pcscOptsE_PCSC_OPT_TIMEOUT,
    VERBOSE = cglue::pcscOptsE_PCSC_OPT_VERBOSE,
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PcscsState {
    EMPTY,
    PRESENT,
    UNKNOWN,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PcscsMonitor {
    UNKNOWN = cglue::pcscMonitorActionE_PCSC_MONITOR_UNKNOWN,
    WAIT = cglue::pcscMonitorActionE_PCSC_MONITOR_WAIT,
    CANCEL = cglue::pcscMonitorActionE_PCSC_MONITOR_CANCEL,
    KILL = cglue::pcscMonitorActionE_PCSC_MONITOR_KILL,
}

// C callback with by reader monitoring signature
#[no_mangle]
pub extern "C" fn reader_monitor_cb(
    _handle: *mut cglue::pcscHandleT,
    state: u64,
    ctx: *mut std::ffi::c_void,
) -> i32 {
    let ctx = unsafe { &mut *(ctx as *mut PcscClient) };

    let state = match state {
        cglue::PCSC_SCARD_STATE_EMPTY => PcscsState::EMPTY,
        cglue::PCSC_SCARD_STATE_PRESENT => PcscsState::PRESENT,
        _ => PcscsState::UNKNOWN,
    };
    ctx.callback(state)
}

impl PcscClient {
    // prepare handle for open operation
    pub fn new(jconf: JsoncObj, verbosity: i32) -> Result<PcscClient, AfbError> {
        let config = unsafe { cglue::pcscParseConfig(jconf.into_raw(), verbosity) };
        if config == 0 as *mut cglue::pcscConfigT {
            return Err(AfbError::new("pcsc-config-fail", jconf.to_string()));
        }

        let client = PcscClient {
            config,
            handle: Cell::new(0 as *mut cglue::pcscHandleT),
        };

        // connect and return object of successful
        client.connect()?;
        Ok(client)
    }

    // move back pointer to object
    pub fn get_config(&self) -> &cglue::pcscConfigT {
        unsafe { &*(self.config) }
    }

    pub fn get_reader_name(&self) -> &'static str {
        let handle = self.handle.get();
        match unsafe { CStr::from_ptr(cglue::pcscReaderName(handle)) }.to_str() {
            Err(_) => "not UTF8 reader devname",
            Ok(value) => value,
        }
    }

    pub fn get_reader_error(&self) -> &'static str {
        let handle = self.handle.get();
        match unsafe { CStr::from_ptr(cglue::pcscErrorMsg(handle)) }.to_str() {
            Err(_) => "invalid error",
            Ok(value) => value,
        }
    }

    pub fn connect(&self) -> Result<&Self, AfbError> {
        let config = self.get_config();

        let handle = unsafe { cglue::pcscConnect(config.uid, config.reader) };
        if handle == 0 as *mut cglue::pcscHandleT {
            return Err(AfbError::new(
                "pcsc-connect-fail",
                format!("Fail to connect to reader={}", self.get_reader_name()),
            ));
        }

        // set reader options
        unsafe {
            cglue::pcscSetOpt(
                handle,
                cglue::pcscOptsE_PCSC_OPT_VERBOSE,
                config.verbose as u64,
            );
            cglue::pcscSetOpt(handle, cglue::pcscOptsE_PCSC_OPT_TIMEOUT, config.timeout);
        }

        let tid = unsafe {
            cglue::pcscMonitorReader(
                handle,
                Some(reader_monitor_cb),
                self as *const _ as *mut std::ffi::c_void,
            )
        };
        if tid <= 0 {
            return Err(AfbError::new(
                "pcsc-monitoring-fail",
                format!(
                    "Fail start reader monitoring thread reader={}",
                    self.get_reader_name()
                ),
            ));
        }

        // update handle with object cell
        self.handle.set(handle);

        Ok(self)
    }

    // call from reader monitoring
    pub(crate) fn callback(&self, state: PcscsState) -> i32 {
        let _handle = self.handle.get();

        match state {
            PcscsState::PRESENT => {
                let _err = self.get_uuid();
                0
            }

            PcscsState::EMPTY => -1,
            _ => -1,
        }
    }

    pub fn get_uuid(&self) -> Result<u64, AfbError> {
        let handle = self.handle.get();

        // try to get card UUID (work with almost any model)
        let uuid = unsafe { cglue::pcscGetCardUuid(handle) };
        if uuid == 0 {
            return Err(AfbError::new(
                "pcsc-get-uuid",
                format!(
                    "Fail reading smart card UUID error={}",
                    self.get_reader_error()
                ),
            ));
        }
        afb_log_msg!(
            Debug,
            None,
            "Reader={} smart uuid={}",
            self.get_reader_name(),
            uuid
        );
        Ok(uuid)
    }

    pub fn reader_check(&self) -> Result<(), AfbError> {
        let handle = self.handle.get();

        // get reader status and wait 10 timeout for card
        let err = unsafe { cglue::pcscReaderCheck(handle, 10) };
        if err != 0 {
            return Err(AfbError::new(
                "pcsc-sync-check",
                format!(
                    "Fail connecting to reader={} error={}",
                    self.get_reader_name(),
                    self.get_reader_error()
                ),
            ));
        }
        Ok(())
    }

    fn get_cmd_by_uid(&self, cuid: &str) -> Result<*mut cglue::pcscCmdT, AfbError> {
        let suid = match CString::new(cuid) {
            Ok(value) => value,
            Err(_) => {
                return Err(AfbError::new(
                    "scard-cuid-invalid",
                    format!("Command cuid={} invalid", cuid),
                ))
            }
        };

        let cmd = unsafe { cglue::pcscCmdByUid(self.config, suid.as_ptr()) };
        if cmd == 0 as *mut cglue::pcscCmdT {
            return Err(AfbError::new(
                "scard-cuid-missing",
                format!("Command cuid={} not found", cuid),
            ));
        }

        Ok(cmd)
    }

    pub fn get_data(&self, cuid: &str) -> Result<String, AfbError> {
        let handle = self.handle.get();
        let cmd = self.get_cmd_by_uid(cuid)?;

        if unsafe { cglue::pcscCmdAction(cmd) } != cglue::pcscActionE_PCSC_ACTION_READ {
            return Err(AfbError::new(
                "scard-cuid-action",
                format!("Command cuid={} is not a read action", cuid),
            ));
        }

        // Fulup TBD initializing this buffer is stupid
        let mut buffer = vec![0; unsafe { cglue::pcscCmdDataLen(cmd) }].into_boxed_slice();
        let cdata = buffer.as_mut_ptr();

        // try to get card UUID (work with almost any model)
        let err = unsafe { cglue::pcscExecOneCmd(handle, cmd, cdata) };
        if err == 0 {
            return Err(AfbError::new(
                "scard-cuid-read",
                format!(
                    "Fail reading smart card cuid={} error={}",
                    cuid,
                    self.get_reader_error()
                ),
            ));
        }

        // return data is a C string
        let cstring = unsafe { CStr::from_ptr(cdata as *const i8) };
        let slice = {
            match cstring.to_str() {
                Err(_) => {
                    return Err(AfbError::new(
                        "scard-cuid-data",
                        format!("Fail concerting smard cuid={} data to utf8", cuid,),
                    ))
                }
                Ok(value) => value,
            }
        };

        afb_log_msg!(
            Debug,
            None,
            "Reader={} cuid={} data={}",
            self.get_reader_name(),
            cuid,
            slice
        );
        Ok(slice.to_string())
    }

    pub fn set_data(&self, cuid: &str, data: &[u8]) -> Result<(), AfbError> {
        let handle = self.handle.get();
        let cmd = self.get_cmd_by_uid(cuid)?;

        if unsafe { cglue::pcscCmdAction(cmd) } != cglue::pcscActionE_PCSC_ACTION_WRITE {
            return Err(AfbError::new(
                "scard-cuid-action",
                format!("Command cuid={} is not a write action", cuid),
            ));
        }

        // try to get card UUID (work with almost any model)
        let err = unsafe { cglue::pcscExecOneCmd(handle, cmd, data.as_ptr() as *const _ as *mut u8) };
        if err == 0 {
            return Err(AfbError::new(
                "scard-cuid-write",
                format!(
                    "Fail writing smart card cuid={} error={}",
                    cuid,
                    self.get_reader_error()
                ),
            ));
        }

        Ok(())
    }
}
