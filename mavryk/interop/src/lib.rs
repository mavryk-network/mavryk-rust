// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

// Cannot enable because of the call to `initialize_tezedge_ipc_callbacks`.
// Once dynamic linking has been got rid of this can be re-enabled.
//#![forbid(unsafe_code)]

use std::sync::Once;

use crypto::hash::ProtocolHash;
use ocaml_interop::{OCamlRuntime, ToOCaml};

use runtime::OCamlBlockPanic;
use mavryk_api::ffi::{ContextDataError, RustBytes, MavrykErrorTrace};
use mavryk_conv::OCamlMavrykErrorTrace;

pub mod ipc_message_encoding;

type TzResult<T> = Result<T, OCamlMavrykErrorTrace>;

mod mavryk_ffi {
    use crate::TzResult;
    use ocaml_interop::{ocaml, OCamlBytes, OCamlInt, OCamlList};

    ocaml! {
        pub fn ffi_server_loop(sock_cmd_path: String) -> TzResult<OCamlInt>;
        pub fn ffi_apply_encoded_message(msg: OCamlBytes) -> Result<OCamlBytes, String>;
        pub fn decode_context_data(
            protocol_hash: OCamlBytes,
            key: OCamlList<OCamlBytes>,
            data: OCamlBytes
        ) -> TzResult<Option<OCamlBytes>>;
    }
}

#[cfg(feature = "fuzzing")]
pub mod fuzzing_coverage {
    use ocaml_interop::{OCaml, OCamlRuntime};

    mod bisect_ffi {
        use ocaml_interop::ocaml;
        ocaml! {
            pub fn dump(a: ());
            pub fn reset_counters(a: ());
        }
    }

    pub fn dump(rt: &mut OCamlRuntime) {
        bisect_ffi::dump(rt, &OCaml::unit());
    }

    pub fn reset_counters(rt: &mut OCamlRuntime) {
        bisect_ffi::reset_counters(rt, &OCaml::unit());
    }
}

pub fn decode_context_data(
    protocol_hash: ProtocolHash,
    key: Vec<String>,
    data: RustBytes,
) -> Result<Option<String>, ContextDataError> {
    let protocol_hash: RustBytes = protocol_hash.into();
    runtime::execute(move |rt: &mut OCamlRuntime| {
        let protocol_hash = protocol_hash.to_boxroot(rt);
        let key_list = key.to_boxroot(rt);
        let data = data.to_boxroot(rt);

        let result = mavryk_ffi::decode_context_data(rt, &protocol_hash, &key_list, &data);
        let result = rt.get(&result).to_result();

        match result {
            Ok(decoded_data) => Ok(decoded_data.to_rust()),
            Err(e) => Err(ContextDataError::from(e.to_rust::<MavrykErrorTrace>())),
        }
    })
    .unwrap_or_else(|p| {
        Err(ContextDataError::DecodeError {
            message: p.to_string(),
        })
    })
}

pub fn start_ipc_loop(
    sock_cmd_path: String,
) -> Result<Result<i64, MavrykErrorTrace>, OCamlBlockPanic> {
    runtime::execute(move |rt: &mut OCamlRuntime| {
        let sock_cmd_path = sock_cmd_path.to_boxroot(rt);
        mavryk_ffi::ffi_server_loop(rt, &sock_cmd_path).to_rust(rt)
    })
}

pub fn apply_encoded_message(
    msg: mavryk_protocol_ipc_messages::ProtocolMessage,
) -> Result<mavryk_protocol_ipc_messages::NodeMessage, String> {
    let encoded_msg = bincode::serialize(&msg).unwrap();

    let result: Result<Vec<u8>, String> = runtime::execute(move |rt: &mut OCamlRuntime| {
        let encoded_msg = encoded_msg.to_boxroot(rt);
        let result = mavryk_ffi::ffi_apply_encoded_message(rt, &encoded_msg);
        result.to_rust(rt)
    })
    .unwrap();

    result.map(|bytes| bincode::deserialize(&bytes).unwrap())
}

/// Initializes the ocaml runtime and the mavryk-ffi callback mechanism.
fn setup() -> OCamlRuntime {
    static INIT: Once = Once::new();
    let ocaml_runtime = OCamlRuntime::init();

    INIT.call_once(|| {
        force_libmavryk_linking();
        mavryk_context::ffi::initialize_callbacks();
        ipc_message_encoding::initialize_callbacks();
    });

    ocaml_runtime
}

pub fn shutdown() {
    runtime::shutdown()
}

/// This modules will allow you to call OCaml code:
///
/// ```ocaml
/// let echo = fun value: string -> value
/// let _ = Callback.register "echo" echo
/// ```
///
/// It can be then easily awaited in rust:
///
/// ```rust, no_run
/// use mavryk_interop::runtime::OCamlCallResult;
/// use mavryk_interop::runtime;
/// use ocaml_interop::{ocaml, ToOCaml, FromOCaml, OCamlRuntime};
///
/// ocaml! {
///     pub fn echo(value: String) -> String;
/// }
///
/// fn ocaml_fn_echo(arg: String) -> OCamlCallResult<String> {
///     runtime::spawn(move |rt: &mut OCamlRuntime| {
///         let value = arg.to_boxroot(rt);
///         let ocaml_result = echo(rt, &value);
///         ocaml_result.to_rust(rt)
///     })
/// }
///
/// let result = futures::executor::block_on(
///     ocaml_fn_echo("Hello world!".into())
/// ).unwrap();
/// assert_eq!("Hello world!", &result);
/// ```
pub mod runtime;

pub fn force_libmavryk_linking() {
    mavryk_sys::force_libmavryk_linking();
}
