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
    // map pcsc abstract pointer should remains allocate within C only
    pub struct pcscCmdT {
        not_to_be_used: u32,
    }
    include!("_capi-map.rs");
}

pub struct PcscCmd {
    handle: *mut cglue::pcscCmdT,
}

pub struct PcscClient {
    config: *mut cglue::pcscConfigT,
    handle: Cell<*mut cglue::pcscHandleT>,
    ctrl: Option<*mut dyn PcscControl>,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PcscOption {
    UNKNOWN = cglue::pcscOptsE_PCSC_OPT_UNKNOWN,
    TIMEOUT = cglue::pcscOptsE_PCSC_OPT_TIMEOUT,
    VERBOSE = cglue::pcscOptsE_PCSC_OPT_VERBOSE,
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum PcscState {
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

    let mut scard_state = PcscState::UNKNOWN;

    if (state & cglue::PCSC_SCARD_STATE_EMPTY) != 0 {
        scard_state = PcscState::EMPTY
    }
    if (state & cglue::PCSC_SCARD_STATE_PRESENT) != 0 {
        scard_state = PcscState::PRESENT
    }

    match  ctx.ctrl {
        None => {},
        Some(ctrl) => unsafe {(*ctrl).scard_monitor(ctx, scard_state);}
    }

    0
}

pub enum PcscAction {
    READ,
    WRITE,
    UUID,
}

pub trait PcscControl {
    fn scard_monitor(&mut self, pcsc: &PcscClient, status: PcscState);
}

impl PcscCmd {
    pub fn get_action(&self) -> PcscAction {
        match unsafe { cglue::pcscCmdAction(self.handle) } {
            cglue::pcscActionE_PCSC_ACTION_READ => PcscAction::READ,
            cglue::pcscActionE_PCSC_ACTION_WRITE => PcscAction::WRITE,
            cglue::pcscActionE_PCSC_ACTION_UUID => PcscAction::UUID,
            _ => PcscAction::UUID,
        }
    }

    pub fn get_uid(&self) -> &'static str {
        match unsafe { CStr::from_ptr(cglue::pcscCmdUid(self.handle)) }.to_str() {
            Err(_) => "Not-UTF8-uid",
            Ok(value) => value,
        }
    }

    pub fn get_info(&self) -> &'static str {
        match unsafe { CStr::from_ptr(cglue::pcscCmdInfo(self.handle)) }.to_str() {
            Err(_) => "Not-UTF8-uid",
            Ok(value) => value,
        }
    }
}

impl PcscClient {
    // prepare handle for open operation
    pub fn new(jconf: JsoncObj, verbosity: i32, callback:Option <*mut dyn PcscControl>) -> Result<PcscClient, AfbError> {
        let config = unsafe { cglue::pcscParseConfig(jconf.into_raw(), verbosity) };
        if config == 0 as *mut cglue::pcscConfigT {
            return Err(AfbError::new("pcsc-config-fail", jconf.to_string()));
        }

        let client = PcscClient {
            config,
            handle: Cell::new(0 as *mut cglue::pcscHandleT),
            ctrl: callback,
        };

        // connect and return object of successful
        client.connect()?;
        Ok(client)
    }

    pub fn get_reader_name(&self) -> &'static str {
        let handle = self.handle.get();
        match unsafe { CStr::from_ptr(cglue::pcscReaderName(handle)) }.to_str() {
            Err(_) => "Not-UTF8-reader-name",
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
        let handle = unsafe { cglue::pcscConnect((*self.config).uid, (*self.config).reader) };
        if handle == 0 as *mut cglue::pcscHandleT {
            return Err(AfbError::new(
                "pcsc-connect-fail",
                format!("Fail to connect to reader check config"),
            ));
        }

        // set reader options
        unsafe {
            cglue::pcscSetOpt(
                handle,
                cglue::pcscOptsE_PCSC_OPT_VERBOSE,
                (*self.config).verbose as u64,
            );
            cglue::pcscSetOpt(
                handle,
                cglue::pcscOptsE_PCSC_OPT_TIMEOUT,
                (*self.config).timeout,
            );
        }

        // update handle with object cell
        self.handle.set(handle);

        Ok(self)
    }

    // monitor one transaction (until scard is removed)
    pub fn monitor_start(&self) -> Result<u64, AfbError> {
        let handle = self.handle.get();

        let tid = unsafe {
            cglue::pcscMonitorReader(
                handle,
                Some(reader_monitor_cb),
                self as *const _ as *mut std::ffi::c_void,
            )
        };

        if tid == 0 {
            return Err(AfbError::new(
                "pcsc-monitoring-start",
                format!(
                    "Fail start reader monitoring thread reader={}",
                    self.get_reader_name()
                ),
            ));
        }
        Ok(tid)
    }

    // monitor one transaction (until scard is removed)
    pub fn monitor_stop(&self, tid: u64) -> Result<(), AfbError> {
        let handle = self.handle.get();

        let rc =
            unsafe { cglue::pcscMonitorWait(handle, cglue::pcscMonitorActionE_PCSC_MONITOR_WAIT, tid) };

        if rc <= 0 {
            return Err(AfbError::new(
                "pcsc-monitoring-stop",
                format!(
                    "Fail start reader monitoring thread reader={}",
                    self.get_reader_name()
                ),
            ));
        }
        Ok(())
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

    pub fn get_cmd_by_uid(&self, cuid: &str) -> Result<PcscCmd, AfbError> {
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
        Ok(PcscCmd { handle: cmd })
    }

    pub fn get_data(&self, cmd: &PcscCmd) -> Result<String, AfbError> {
        let handle = self.handle.get();

        // Fulup TBD initializing this buffer is stupid
        let mut buffer = vec![0; unsafe { cglue::pcscCmdDataLen(cmd.handle) }].into_boxed_slice();
        let cdata = buffer.as_mut_ptr();

        // try to get card UUID (work with almost any model)
        let err = unsafe { cglue::pcscExecOneCmd(handle, cmd.handle, cdata) };
        if err < 0 {
            return Err(AfbError::new(
                "scard-cuid-read",
                format!("cuid={} error={}", cmd.get_uid(), self.get_reader_error()),
            ));
        }

        // return data is a C string
        let cstring = unsafe { CStr::from_ptr(cdata as *const i8) };
        let slice = {
            match cstring.to_str() {
                Err(_) => {
                    return Err(AfbError::new(
                        "scard-cuid-data",
                        format!("cuid={} no utf8 data", cmd.get_uid(),),
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
            cmd.get_uid(),
            slice
        );
        Ok(slice.to_string())
    }

    pub fn set_data(&self, cmd: &PcscCmd, data: &[u8]) -> Result<(), AfbError> {
        let handle = self.handle.get();

        // try to get card UUID (work with almost any model)
        let err = unsafe {
            cglue::pcscExecOneCmd(handle, cmd.handle, data.as_ptr() as *const _ as *mut u8)
        };
        if err < 0 {
            return Err(AfbError::new(
                "scard-cuid-write",
                format!("cuid={} {}", cmd.get_uid(), self.get_reader_error()),
            ));
        }

        Ok(())
    }
}
