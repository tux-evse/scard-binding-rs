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
 */

use crate::prelude::*;
use afbv4::prelude::*;
use libnfc::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

struct WriteScardCtx {
    scard: Rc<ScardHandle>,
    cmd: ScardCmd,
}

AfbVerbRegister!(WriteScardVerb, write_scard_cb, WriteScardCtx);
fn write_scard_cb(
    rqt: &AfbRequest,
    args: &AfbData,
    ctx: &mut WriteScardCtx,
) -> Result<(), AfbError> {
    // because of devtools limitation we have to use a full json object
    let jsonc = args.get::<JsoncObj>(0)?;
    let data= jsonc.get::<String>("data")?;
    afb_log_msg!(Notice,rqt,"writing:{}", data);
    ctx.scard.write_data(&ctx.cmd, data.as_str().as_bytes())?;

    rqt.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct ReadScardCtx {
    scard: Rc<ScardHandle>,
    cmd: ScardCmd,
}

AfbVerbRegister!(ReadScardVerb, read_scard_cb, ReadScardCtx);
fn read_scard_cb(
    rqt: &AfbRequest,
    _args: &AfbData,
    ctx: &mut ReadScardCtx,
) -> Result<(), AfbError> {
    let data = ctx.scard.read_data(&ctx.cmd)?;
    rqt.reply(data, 0);
    Ok(())
}

struct UuidScardCtx {
    scard: Rc<ScardHandle>,
}

AfbVerbRegister!(UuidScardVerb, uuid_scard_cb, UuidScardCtx);
fn uuid_scard_cb(
    rqt: &AfbRequest,
    _args: &AfbData,
    ctx: &mut UuidScardCtx,
) -> Result<(), AfbError> {
    let uuid = ctx.scard.get_uuid()?;
    rqt.reply(uuid, 0);
    Ok(())
}

struct ScardMonitorCtx {
    event: &'static AfbEvent,
    tid: Cell<u64>,
}

// TBD: this callback is call directly from CAPI, when it should be called from sacrd-nfc
struct MonitorCtx {
    monitor: Rc<ScardMonitorCtx>,
}
impl PcscControl for MonitorCtx {
    fn scard_monitor(&mut self, _scard: &PcscClient, status: PcscState) {
        let count = match status {
            PcscState::PRESENT => self.monitor.event.push("PRESENT"),
            PcscState::EMPTY => self.monitor.event.push("ABSENT"),
            PcscState::UNKNOWN => self.monitor.event.push("UNKNOWN"),
        };

        // no more listener let stop monitoring thread
        if count == 0 {
            //let _=scard.monitor_stop(self.monitor.tid.get());
            //self.monitor.tid.set(0);
        }
    }
}

struct EventScardCtx {
    scard: Rc<ScardHandle>,
    monitor: Rc<ScardMonitorCtx>,
}
AfbVerbRegister!(EventScardVerb, event_scard_cb, EventScardCtx);
fn event_scard_cb(
    rqt: &AfbRequest,
    args: &AfbData,
    ctx: &mut EventScardCtx,
) -> Result<(), AfbError> {
    match args.get::<&ApiAction>(0)? {
        ApiAction::START => {
            if ctx.monitor.tid.get() == 0 {
                let tid = ctx.scard.monitor(MonitorAction::START)?;
                ctx.monitor.tid.set(tid);
            }
            ctx.monitor.event.subscribe(rqt)?;
        }
        ApiAction::STOP => {
            // monitoring thread stop is done within monitoring callback
            ctx.monitor.event.unsubscribe(rqt)?;
        }
    }

    rqt.reply(AFB_NO_DATA, 0);
    Ok(())
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {
    // parse NFC config and connect to reader
    let event = AfbEvent::new("reader");
    let monitor = Rc::new(ScardMonitorCtx {
        event: event,
        tid: Cell::new(0),
    });

    let callback = Box::new(MonitorCtx {
        monitor: monitor.clone(),
    });
    let scard = Rc::new(ScardHandle::new(
        config.nfc.clone(),
        config.verbosity,
        Some(Box::leak(callback)),
    )?);

    let verb_ctx = Box::new(EventScardVerb {
        scard: scard.clone(),
        monitor: monitor.clone(),
    });

    let _verb = AfbVerb::new("monitoring")
        .set_info("subscribe to scard transaction event")
        .set_action("['START','STOP']")?
        .set_callback(verb_ctx)
        .finalize()?;

    // TBD (Fulup) TBD Monitoring works but preempt pcscclient.
    // It should probably use an independent pcscClient handle
    // api.add_verb(verb);
    // api.add_event(event);

    // loop on command and create corresponding verbs
    let cmds = config.nfc.clone().get::<JsoncObj>("cmds")?;
    for idx in 0..cmds.count()? {
        let cmd = cmds.index::<JsoncObj>(idx)?;
        let cuid = cmd.get::<String>("uid")?;
        let cmd = scard.get_cmd_by_uid(cuid.as_str())?;
        let info = cmd.get_info();
        let uid = cmd.get_uid();
        let mut verb_usage= "none";

        let verb_ctx: Box<dyn AfbRqtControl> = match cmd.get_action() {
            ScardAction::READ => Box::new(ReadScardVerb {
                scard: scard.clone(),
                cmd: cmd,
            }),

            ScardAction::WRITE => {
                verb_usage= "value";
                Box::new(WriteScardVerb {
                scard: scard.clone(),
                cmd: cmd,
            })},

            _ => Box::new(UuidScardVerb {
                scard: scard.clone(),
            }),
        };

        let verb = AfbVerb::new(uid)
            .set_info(info)
            .set_usage(verb_usage)
            .set_callback(verb_ctx)
            .finalize()?;

        api.add_verb(verb);
    }

    Ok(())
}
