#!/bin/bash

DIR=`dirname $0`
cd $DIR/..

# use libafb development version if any
export LD_LIBRARY_PATH="/usr/local/lib64:$LD_LIBRARY_PATH"
export PATH="/usr/local/lib64:$PATH"
clear

# give access to devtools via TCP port
PERMISION_ADM=`which cynagora-admin 2>/dev/null`
if test -n "$PERMISION_ADM"; then
    echo "NOTICE: Grant full permission to 'Hello'"
    cynagora-admin set '' 'HELLO' '' '*' yes 2> /dev/null
fi

if ! test -f ./lib/libafb_nfc.so; then
    echo "FATAL: missing libafb_nfc.so use: cargo build"
    exit 1
fi

# start binder with test config
afb-binder -vvv --config=./etc/binding-nfc.json
