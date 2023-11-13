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
use libnfc::prelude::*;
use afbv4::prelude::*;
use std::rc::Rc;

struct SendScardCtx {
    scard: Rc<ScardHandle>,
}

AfbVerbRegister!(SendScardVerb, send_scard_cb, SendScardCtx);
fn send_scard_cb(rqt: &AfbRequest, args: &AfbData, ctx: &mut SendScardCtx) -> Result<(), AfbError> {

    // extract command UUID
    let cuid= args.get::<String>(0)?;
    let data= args.get::<String>(1)?;

    ctx.scard.send_data (cuid.as_str(),data.as_str().as_bytes())?;

    rqt.reply(AFB_NO_DATA, 0);
    Ok(())
}


struct ReadScardCtx {
    scard: Rc<ScardHandle>,
}

AfbVerbRegister!(ReadScardVerb, read_scard_cb, ReadScardCtx);
fn read_scard_cb(rqt: &AfbRequest, args: &AfbData, ctx: &mut ReadScardCtx) -> Result<(), AfbError> {

    // extract command UUID
    let cuid= args.get::<String>(0)?;

    let data= ctx.scard.read_data (cuid.as_str())?;

    rqt.reply(data, 0);
    Ok(())
}

struct UuidScardCtx {
    scard: Rc<ScardHandle>,
}

AfbVerbRegister!(UuidScardVerb, uuid_scard_cb, UuidScardCtx);
fn uuid_scard_cb(rqt: &AfbRequest, _args: &AfbData, ctx: &mut UuidScardCtx) -> Result<(), AfbError> {

    let uuid= ctx.scard.get_uuid()?;
    rqt.reply(uuid, 0);
    Ok(())
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {

    // parse NFC config and connect to reader
    let handle=Rc::new(ScardHandle::new(config.nfc,config.verbosity)?);

    let uuid_vb = AfbVerb::new("read_uuid")
    .set_info("read scard uuid number")
    .set_usage("no-input")
    .set_callback(Box::new(UuidScardCtx {
        scard: handle.clone(),
    }))
    .finalize()?;

    let read_vb = AfbVerb::new("read_data")
    .set_info("read data block from from scard using cuid=label")
    .set_usage("cuid:string")
    .set_sample("'user-id'")?
    .set_sample("'get-status'")?
    .set_callback(Box::new(ReadScardCtx {
        scard: handle.clone(),
    }))
    .finalize()?;

    let send_vb = AfbVerb::new("send_data")
    .set_info("send data block to scard using cuid==label")
    .set_usage("cuid:string")
    .set_sample("'set-status'")?
    .set_callback(Box::new(ReadScardCtx {
        scard: handle.clone(),
    }))
    .finalize()?;

    // add verbs to api
    api.add_verb(uuid_vb);
    api.add_verb(read_vb);
    api.add_verb(send_vb);

    Ok(())
}
