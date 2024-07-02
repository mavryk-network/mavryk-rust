Mavryk crypto
===========

Component contains cryptographic algorithms for hashing, signing & signature verification, with a slant towards
those used by [mavryk](https://mavryk.org/) - and in particular the `Kernel SDK` for [smart rollups](https://protocol.mavryk.org/alpha/smart_rollups.html).

## Hash module

`mavryk_crypto::hash` contains definitions for common hashes in tezos - such as contract & address hashes. These
support `b58check` encoding/decoding with the same prefixes used in the rest of tezos - such as `mv1` for `ed25519` addresses.

These support encoding/decoding to binary with the `mavryk_encoding` crate.
