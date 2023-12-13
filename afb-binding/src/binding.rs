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

pub(crate) fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

AfbDataConverter!(api_actions, ApiAction);
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase", tag = "action")]
pub(crate) enum ApiAction {
    #[default]
    START,
    STOP,
}

pub struct BindingCfg {
    pub uid: &'static str,
    pub nfc: JsoncObj,
    pub verbosity: i32,
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    // add binding custom converter
    api_actions::register()?;

    let uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "nfc"
    };

    let api = if let Ok(value) = jconf.get::<String>("api") {
        to_static_str(value)
    } else {
        uid
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    afb_log_msg!(
        Info,
        rootv4,
        "Binding starting uid:{} api:{} info:{}",
        uid,
        api,
        info
    );

    let permission = if let Ok(value) = jconf.get::<String>("permission") {
        AfbPermission::new(to_static_str(value))
    } else {
        AfbPermission::new("acl:nfc:client")
    };

    let nfc = if let Ok(value) = jconf.get::<JsoncObj>("nfc") {
        value
    } else {
        return Err(AfbError::new(
            "nfc-config-fail",
            "mandatory nfc' config missing",
        ));
    };

    let verbosity = if let Ok(value) = jconf.get::<i32>("verbosity") {
        value
    } else {
        0
    };

    let config = BindingCfg {
        uid,
        nfc,
        verbosity,
    };

    // create backend API
    let api = AfbApi::new(api).set_info(info).set_permission(permission);
    register_verbs(api, config)?;

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
