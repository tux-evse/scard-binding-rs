# scard-binding-rs
Scard NFC reader afb binding

## dependencies
https://github.com/redpesk-common/sec-pcscd-client

## provisioning scard

scard-binding-rs testing config allow (contract/name/status) provisioning. Nevertheless depending
on the number of nfc-scard you should provision for your test, it might be simpler to provision them
directly from your Desktop with your own provision config file as in bellow sample.

*Check sec-pcscd-client README for detail config.json definition.*

**Check NFC reader presence**
```
# open reader defined in your config.json
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --list

# check your test card by presenting it on the reader
```

**Step-by-step NFC card provisioning**
```
# cmd-group=0(uuid) => check your card model is supported
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --group=0

# cmd-group=9 => set card ACLs (set key-A for read and key-B for write)
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --group=9

# cmd-group=2(Mr Kermichue) => write data blocks(3+4+5) using ket-B
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --group=2

# cmd-group=3(Mme Kermichue) => write data blocks(3+4+5) using ket-B
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --group=2

# check data blocks(3.4.5) are readable with key-A
/usr/local/redpesk/pcscs-client/bin/pcscd-client --config=afb-binding/etc/pcscd-client-test.json --group=1
```