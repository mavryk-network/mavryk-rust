// Copyright (c) SimpleStaking, Viable Systems and Tezedge Contributors
// SPDX-License-Identifier: MIT

#![allow(clippy::ptr_arg)]

use ocaml_interop::*;

pub type ContextKey = OCamlList<String>;

#[derive(PartialEq, Debug)]
pub struct ContextHash(pub Vec<u8>);

#[derive(PartialEq, Debug)]
pub struct ProtocolHash(pub Vec<u8>);

enum TaggedHash<'a> {
    Hash(&'a [u8]),
}

/// Opaque representation of the Index
pub struct MavrykFfiContextIndex {}

/// Opaque representation of the Context
pub struct MavrykFfiContext {}

/// Opaque representation of subtrees
pub struct MavrykFfiTree {}

// Conversion between OCaml and Rust representations of hashes

unsafe impl FromOCaml<ContextHash> for ContextHash {
    fn from_ocaml(v: OCaml<ContextHash>) -> Self {
        ContextHash(unsafe { v.field::<OCamlBytes>(0).to_rust() })
    }
}

unsafe impl ToOCaml<ContextHash> for ContextHash {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, ContextHash> {
        let hash = TaggedHash::Hash(&self.0);
        ocaml_alloc_variant! {
            cr, hash => {
                TaggedHash::Hash(hash: OCamlBytes)
            }
        }
    }
}

unsafe impl FromOCaml<ProtocolHash> for ProtocolHash {
    fn from_ocaml(v: OCaml<ProtocolHash>) -> Self {
        ProtocolHash(unsafe { v.field::<OCamlBytes>(0).to_rust() })
    }
}

unsafe impl ToOCaml<ProtocolHash> for ProtocolHash {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, ProtocolHash> {
        let hash = TaggedHash::Hash(&self.0);
        ocaml_alloc_variant! {
            cr, hash => {
                TaggedHash::Hash(hash: OCamlBytes)
            }
        }
    }
}

// FFI function declarations
mod ffi {
    use super::{
        ContextHash, ContextKey, ProtocolHash, MavrykFfiContext, MavrykFfiContextIndex, MavrykFfiTree,
    };
    use ocaml_interop::*;

    ocaml! {
        pub fn mavryk_context_init_irmin(data_dir: String, genesis: (String, String, String), sandbox_json_patch_context: Option<(String, String)>) -> Result<(MavrykFfiContextIndex, ContextHash), String>;
        pub fn mavryk_context_init_tezedge(data_dir: String, genesis: (String, String, String), sandbox_json_patch_context: Option<(String, String)>) -> Result<(MavrykFfiContextIndex, ContextHash), String>;
        pub fn mavryk_context_close(index: MavrykFfiContextIndex);
        // Context query
        pub fn mavryk_context_mem(ctxt: MavrykFfiContext, key: ContextKey) -> bool;
        pub fn mavryk_context_mem_tree(ctxt: MavrykFfiContext, key: ContextKey) -> bool;
        pub fn mavryk_context_find(ctxt: MavrykFfiContext, key: ContextKey) -> Option<OCamlBytes>;
        pub fn mavryk_context_find_tree(ctxt: MavrykFfiContext, key: ContextKey) -> Option<MavrykFfiTree>;
        pub fn mavryk_context_add(ctxt: MavrykFfiContext, key: ContextKey, value: OCamlBytes) -> MavrykFfiContext;
        pub fn mavryk_context_add_tree(ctxt: MavrykFfiContext, key: ContextKey, tree: MavrykFfiTree) -> MavrykFfiContext;
        pub fn mavryk_context_remove(ctxt: MavrykFfiContext, key: ContextKey) -> MavrykFfiContext;
        pub fn mavryk_context_hash(time: OCamlInt64, message: Option<String>, ctxt: MavrykFfiContext) -> ContextHash;
        // TODO:
        // mavryk_context_list, mavryk_context_fold
        // Repository
        pub fn mavryk_context_checkout(idx: MavrykFfiContextIndex, ctxt_hash: ContextHash) -> Option<MavrykFfiContext>;
        pub fn mavryk_context_commit(time: OCamlInt64, message: String, ctxt: MavrykFfiContext) -> ContextHash;
        // Tree
        pub fn mavryk_context_tree_mem(tree: MavrykFfiTree, key: ContextKey) -> bool;
        pub fn mavryk_context_tree_mem_tree(tree: MavrykFfiTree, key: ContextKey) -> bool;
        pub fn mavryk_context_tree_find(tree: MavrykFfiTree, key: ContextKey) -> Option<OCamlBytes>;
        pub fn mavryk_context_tree_find_tree(tree: MavrykFfiTree, key: ContextKey) -> Option<MavrykFfiTree>;
        pub fn mavryk_context_tree_add(tree: MavrykFfiTree, key: ContextKey, value: OCamlBytes) -> MavrykFfiTree;
        pub fn mavryk_context_tree_add_tree(tree: MavrykFfiTree, key: ContextKey, subtree: MavrykFfiTree) -> MavrykFfiTree;
        pub fn mavryk_context_tree_remove(tree: MavrykFfiTree, key: ContextKey) -> MavrykFfiTree;
        pub fn mavryk_context_tree_empty(ctxt: MavrykFfiContext) -> MavrykFfiTree;
        pub fn mavryk_context_tree_is_empty(tree: MavrykFfiTree) -> bool;
        pub fn mavryk_context_tree_hash(tree: MavrykFfiTree) -> ContextHash;
        // TODO:
        // mavryk_context_tree_list, mavryk_context_tree_fold
        // Special
        pub fn mavryk_context_get_protocol(ctxt: MavrykFfiContext) -> ProtocolHash;
        pub fn mavryk_context_add_protocol(ctxt: MavrykFfiContext, proto_hash: ProtocolHash) -> MavrykFfiContext;
        // TODO:
        // mavryk_context_get_test_chain, mavryk_context_add_test_chain, mavryk_context_remove_test_chain
    }
}

// Context operations

pub mod context {
    use super::*;

    pub fn init_irmin(
        cr: &mut OCamlRuntime,
        data_dir: &str,
        genesis: (String, String, String),
        sandbox_json_patch_context: Option<(String, String)>,
    ) -> Result<(BoxRoot<MavrykFfiContextIndex>, ContextHash), String> {
        let data_dir = data_dir.to_boxroot(cr);
        let genesis = genesis.to_boxroot(cr);
        let sandbox_json_patch_context = sandbox_json_patch_context.to_boxroot(cr);
        let result =
            ffi::mavryk_context_init_irmin(cr, &data_dir, &genesis, &sandbox_json_patch_context);
        match cr.get(&result).to_result() {
            Ok(result) => Ok((BoxRoot::new(result.fst()), result.snd().to_rust())),
            Err(err) => Err(err.to_rust()),
        }
    }

    pub fn init_tezedge(
        cr: &mut OCamlRuntime,
        genesis: (String, String, String),
        sandbox_json_patch_context: Option<(String, String)>,
    ) -> Result<(BoxRoot<MavrykFfiContextIndex>, ContextHash), String> {
        let genesis = genesis.to_boxroot(cr);
        let sandbox_json_patch_context = sandbox_json_patch_context.to_boxroot(cr);
        let data_dir = ":inmem:".to_boxroot(cr);
        let result =
            ffi::mavryk_context_init_tezedge(cr, &data_dir, &genesis, &sandbox_json_patch_context);
        match cr.get(&result).to_result() {
            Ok(result) => Ok((BoxRoot::new(result.fst()), result.snd().to_rust())),
            Err(err) => Err(err.to_rust()),
        }
    }

    pub fn close(cr: &mut OCamlRuntime, index: OCamlRef<MavrykFfiContextIndex>) {
        ffi::mavryk_context_close(cr, index);
    }

    // Context query

    pub fn mem(cr: &mut OCamlRuntime, ctxt: OCamlRef<MavrykFfiContext>, key: &Vec<String>) -> bool {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_mem(cr, ctxt, &key).to_rust(cr)
    }

    pub fn mem_tree(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
    ) -> bool {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_mem_tree(cr, ctxt, &key).to_rust(cr)
    }

    pub fn find(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
    ) -> Option<Vec<u8>> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_find(cr, ctxt, &key).to_rust(cr)
    }

    pub fn find_tree(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
    ) -> Option<BoxRoot<MavrykFfiTree>> {
        let key = key.to_boxroot(cr);
        let result = ffi::mavryk_context_find_tree(cr, ctxt, &key);
        cr.get(&result).to_option().map(BoxRoot::new)
    }

    pub fn add(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
        value: &[u8],
    ) -> BoxRoot<MavrykFfiContext> {
        let key = key.to_boxroot(cr);
        let value = value.to_boxroot(cr);
        ffi::mavryk_context_add(cr, ctxt, &key, &value)
    }

    pub fn add_tree(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
        tree: OCamlRef<MavrykFfiTree>,
    ) -> BoxRoot<MavrykFfiContext> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_add_tree(cr, ctxt, &key, tree)
    }

    pub fn remove(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        key: &Vec<String>,
    ) -> BoxRoot<MavrykFfiContext> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_remove(cr, ctxt, &key)
    }

    pub fn hash(
        cr: &mut OCamlRuntime,
        time: i64,
        message: Option<String>,
        ctxt: OCamlRef<MavrykFfiContext>,
    ) -> ContextHash {
        let time = time.to_boxroot(cr);
        let message = message.to_boxroot(cr);
        ffi::mavryk_context_hash(cr, &time, &message, ctxt).to_rust(cr)
    }

    // Repository

    pub fn checkout(
        cr: &mut OCamlRuntime,
        ctxt_idx: OCamlRef<MavrykFfiContextIndex>,
        hash: &ContextHash,
    ) -> Option<BoxRoot<MavrykFfiContext>> {
        let hash = hash.to_boxroot(cr);
        let result = ffi::mavryk_context_checkout(cr, ctxt_idx, &hash);
        cr.get(&result).to_option().map(BoxRoot::new)
    }

    pub fn commit(
        cr: &mut OCamlRuntime,
        time: i64,
        message: &str,
        ctxt: OCamlRef<MavrykFfiContext>,
    ) -> ContextHash {
        let time = time.to_boxroot(cr);
        let message = message.to_boxroot(cr);
        ffi::mavryk_context_commit(cr, &time, &message, ctxt).to_rust(cr)
    }

    // Special

    pub fn get_protocol(cr: &mut OCamlRuntime, ctxt: OCamlRef<MavrykFfiContext>) -> Vec<u8> {
        let proto_hash: ProtocolHash = ffi::mavryk_context_get_protocol(cr, ctxt).to_rust(cr);
        proto_hash.0
    }

    pub fn add_protocol(
        cr: &mut OCamlRuntime,
        ctxt: OCamlRef<MavrykFfiContext>,
        value: &[u8],
    ) -> BoxRoot<MavrykFfiContext> {
        let value = ProtocolHash(value.into());
        let value = value.to_boxroot(cr);
        ffi::mavryk_context_add_protocol(cr, ctxt, &value)
    }
}

// Subtree operations

pub mod tree {
    use super::*;

    pub fn empty(cr: &mut OCamlRuntime, ctxt: OCamlRef<MavrykFfiContext>) -> BoxRoot<MavrykFfiTree> {
        ffi::mavryk_context_tree_empty(cr, ctxt)
    }

    pub fn is_empty(cr: &mut OCamlRuntime, tree: OCamlRef<MavrykFfiTree>) -> bool {
        ffi::mavryk_context_tree_is_empty(cr, tree).to_rust(cr)
    }

    pub fn mem(cr: &mut OCamlRuntime, tree: OCamlRef<MavrykFfiTree>, key: &Vec<String>) -> bool {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_tree_mem(cr, tree, &key).to_rust(cr)
    }

    pub fn mem_tree(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
    ) -> bool {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_tree_mem_tree(cr, tree, &key).to_rust(cr)
    }

    pub fn find(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
    ) -> Option<Vec<u8>> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_tree_find(cr, tree, &key).to_rust(cr)
    }

    pub fn find_tree(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
    ) -> Option<BoxRoot<MavrykFfiTree>> {
        let key = key.to_boxroot(cr);
        let result = ffi::mavryk_context_tree_find_tree(cr, tree, &key);
        cr.get(&result).to_option().map(BoxRoot::new)
    }

    pub fn add(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
        value: &[u8],
    ) -> BoxRoot<MavrykFfiTree> {
        let key = key.to_boxroot(cr);
        let value = value.to_boxroot(cr);
        ffi::mavryk_context_tree_add(cr, tree, &key, &value)
    }

    pub fn add_tree(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
        subtree: OCamlRef<MavrykFfiTree>,
    ) -> BoxRoot<MavrykFfiTree> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_tree_add_tree(cr, tree, &key, subtree)
    }

    pub fn remove(
        cr: &mut OCamlRuntime,
        tree: OCamlRef<MavrykFfiTree>,
        key: &Vec<String>,
    ) -> BoxRoot<MavrykFfiTree> {
        let key = key.to_boxroot(cr);
        ffi::mavryk_context_tree_remove(cr, tree, &key)
    }

    pub fn hash(cr: &mut OCamlRuntime, tree: OCamlRef<MavrykFfiTree>) -> ContextHash {
        ffi::mavryk_context_tree_hash(cr, tree).to_rust(cr)
    }
}

#[cfg(test)]
macro_rules! key {
    ($key:expr) => {{
        $key.split('/').map(String::from).collect::<Vec<String>>()
    }};
    ($($arg:tt)*) => {{
        key!(format!($($arg)*))
    }};
}

#[cfg(test)]
fn test_context_inodes(
    cr: &mut OCamlRuntime,
    tezedge_index: OCamlRef<MavrykFfiContextIndex>,
    tezedge_genesis_hash: &ContextHash,
    irmin_index: OCamlRef<MavrykFfiContextIndex>,
    irmin_genesis_hash: &ContextHash,
) {
    use std::time::SystemTime;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut tezedge_ctxt = context::checkout(cr, tezedge_index, tezedge_genesis_hash).unwrap();
    let mut irmin_ctxt = context::checkout(cr, irmin_index, irmin_genesis_hash).unwrap();

    let commit_string = Some("commit".to_string());

    let tezedge_commit_hash_init =
        context::hash(cr, time as i64, commit_string.clone(), &tezedge_ctxt);
    let irmin_commit_hash_init = context::hash(cr, time as i64, commit_string.clone(), &irmin_ctxt);

    for index in 0..2_000 {
        let key = key!(format!("root/{}", index));

        tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key, "".as_bytes());
        irmin_ctxt = context::add(cr, &irmin_ctxt, &key, "".as_bytes());

        let tezedge_commit_hash =
            context::hash(cr, time as i64, commit_string.clone(), &tezedge_ctxt);
        let irmin_commit_hash = context::hash(cr, time as i64, commit_string.clone(), &irmin_ctxt);

        assert_eq!(irmin_commit_hash, tezedge_commit_hash);
    }

    for index in 0..2_000 {
        let key = key!(format!("root/{}", index));

        tezedge_ctxt = context::remove(cr, &tezedge_ctxt, &key);
        irmin_ctxt = context::remove(cr, &irmin_ctxt, &key);

        let tezedge_commit_hash =
            context::hash(cr, time as i64, commit_string.clone(), &tezedge_ctxt);
        let irmin_commit_hash = context::hash(cr, time as i64, commit_string.clone(), &irmin_ctxt);

        assert_eq!(irmin_commit_hash, tezedge_commit_hash);
    }

    let tezedge_commit_hash_end =
        context::hash(cr, time as i64, commit_string.clone(), &tezedge_ctxt);
    let irmin_commit_hash_end = context::hash(cr, time as i64, commit_string, &irmin_ctxt);
    assert_eq!(irmin_commit_hash_end, tezedge_commit_hash_end);

    assert_eq!(irmin_commit_hash_init, irmin_commit_hash_end);
    assert_eq!(tezedge_commit_hash_init, tezedge_commit_hash_end);
}

#[cfg(test)]
fn test_context_calls(cr: &mut OCamlRuntime) {
    use std::time::SystemTime;
    use tempfile::tempdir;

    // Initialize the persistent OCaml runtime and initialize callbacks
    mavryk_context::ffi::initialize_callbacks();

    let irmin_dir = tempdir().unwrap();
    let genesis = (
        "2019-08-06T15:18:56Z".to_string(),
        "BLockGenesisGenesisGenesisGenesisGenesiscde8db4cX94".to_string(),
        "PtBMwNZT94N7gXKw4i273CKcSaBrrBnqnt3RATExNKr9KNX2USV".to_string(),
    );

    let (tezedge_index, tezedge_genesis_hash) =
        context::init_tezedge(cr, genesis.clone(), None).unwrap();
    let (irmin_index, irmin_genesis_hash) =
        context::init_irmin(cr, irmin_dir.path().to_str().unwrap(), genesis, None).unwrap();

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    test_context_inodes(
        cr,
        &tezedge_index,
        &tezedge_genesis_hash,
        &irmin_index,
        &irmin_genesis_hash,
    );

    let tezedge_ctxt = context::checkout(cr, &tezedge_index, &tezedge_genesis_hash).unwrap();
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &vec![], "123".as_bytes());
    let tezedge_tree = context::find_tree(cr, &tezedge_ctxt, &vec![]).unwrap();
    let tezedge_tree_hash = tree::hash(cr, &tezedge_tree);
    let tezedge_commit_hash = context::commit(cr, time as i64, "commit", &tezedge_ctxt);

    let irmin_ctxt = context::checkout(cr, &irmin_index, &irmin_genesis_hash).unwrap();
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &vec![], "123".as_bytes());
    let irmin_tree = context::find_tree(cr, &irmin_ctxt, &vec![]).unwrap();
    let irmin_tree_hash = tree::hash(cr, &irmin_tree);
    let irmin_commit_hash = context::commit(cr, time as i64, "commit", &irmin_ctxt);

    // Make sure that the hash is correct when the root of the tree is a value (blob)
    assert_eq!(irmin_commit_hash, tezedge_commit_hash);
    assert_eq!(tezedge_tree_hash, irmin_tree_hash);

    let tezedge_ctxt = context::checkout(cr, &tezedge_index, &tezedge_genesis_hash).unwrap();
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key!("empty/value"), "".as_bytes());
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key!("some/path"), "value".as_bytes());
    let tezedge_ctxt = context::remove(cr, &tezedge_ctxt, &key!("some/path/nested"));
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key!("some/path2"), "value".as_bytes());
    let tezedge_ctxt = context::remove(cr, &tezedge_ctxt, &key!("some/path2"));
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key!("some/path3"), "value".as_bytes());
    let tezedge_empty_tree = tree::empty(cr, &tezedge_ctxt);
    let tezedge_ctxt =
        context::add_tree(cr, &tezedge_ctxt, &key!("some/path3"), &tezedge_empty_tree);
    let tezedge_ctxt = context::add(
        cr,
        &tezedge_ctxt,
        &key!("some/path4/nest"),
        "value".as_bytes(),
    );
    let tezedge_ctxt = context::add(cr, &tezedge_ctxt, &key!("some/path4"), "value".as_bytes());
    let tezedge_ctxt_hash = context::hash(cr, time as i64, None, &tezedge_ctxt);
    let tezedge_commit_hash = context::commit(cr, time as i64, "commit", &tezedge_ctxt);

    let irmin_ctxt = context::checkout(cr, &irmin_index, &irmin_genesis_hash).unwrap();
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &key!("empty/value"), "".as_bytes());
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &key!("some/path"), "value".as_bytes());
    let irmin_ctxt = context::remove(cr, &irmin_ctxt, &key!("some/path/nested"));
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &key!("some/path2"), "value".as_bytes());
    let irmin_ctxt = context::remove(cr, &irmin_ctxt, &key!("some/path2"));
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &key!("some/path3"), "value".as_bytes());
    let irmin_empty_tree = tree::empty(cr, &irmin_ctxt);
    let irmin_ctxt = context::add_tree(cr, &irmin_ctxt, &key!("some/path3"), &irmin_empty_tree);
    let irmin_ctxt = context::add(
        cr,
        &irmin_ctxt,
        &key!("some/path4/nest"),
        "value".as_bytes(),
    );
    let irmin_ctxt = context::add(cr, &irmin_ctxt, &key!("some/path4"), "value".as_bytes());
    let irmin_ctxt_hash = context::hash(cr, time as i64, None, &irmin_ctxt);
    let irmin_commit_hash = context::commit(cr, time as i64, "commit", &irmin_ctxt);

    assert_eq!(irmin_commit_hash, tezedge_commit_hash);
    assert_eq!(tezedge_ctxt_hash, irmin_ctxt_hash);

    let tezedge_ctxt = context::checkout(cr, &tezedge_index, &tezedge_commit_hash);

    assert!(tezedge_ctxt.is_some());

    let irmin_ctxt = context::checkout(cr, &irmin_index, &irmin_commit_hash);

    assert!(irmin_ctxt.is_some());

    let tezedge_ctxt = tezedge_ctxt.unwrap();
    let irmin_ctxt = irmin_ctxt.unwrap();

    assert!(context::mem(cr, &tezedge_ctxt, &key!("some/path")));
    assert!(context::mem_tree(cr, &tezedge_ctxt, &key!("some/path")));
    assert!(!context::mem(cr, &tezedge_ctxt, &key!("some/path2")));
    assert!(!context::mem(cr, &tezedge_ctxt, &key!("some")));
    assert!(context::mem_tree(cr, &tezedge_ctxt, &key!("some")));
    assert!(context::mem(cr, &irmin_ctxt, &key!("some/path")));
    assert!(context::mem_tree(cr, &irmin_ctxt, &key!("some/path")));
    assert!(!context::mem(cr, &irmin_ctxt, &key!("some/path2")));
    assert!(!context::mem(cr, &irmin_ctxt, &key!("some")));
    assert!(context::mem_tree(cr, &irmin_ctxt, &key!("some")));

    let tezedge_tree = tree::empty(cr, &tezedge_ctxt);

    assert!(tree::is_empty(cr, &tezedge_tree));

    let tezedge_tree = tree::add(cr, &tezedge_tree, &key!("some/path"), "value".as_bytes());
    let tezedge_tree = tree::add(cr, &tezedge_tree, &key!("some/path2"), "value".as_bytes());
    let tezedge_tree = tree::remove(cr, &tezedge_tree, &key!("some/path2"));

    assert!(tree::mem(cr, &tezedge_tree, &key!("some/path")));
    assert!(!tree::mem(cr, &tezedge_tree, &key!("some/path2")));
    assert!(!tree::is_empty(cr, &tezedge_tree));

    let irmin_tree = tree::empty(cr, &irmin_ctxt);

    assert!(tree::is_empty(cr, &irmin_tree));

    let irmin_tree = tree::add(cr, &irmin_tree, &key!("some/path"), "value".as_bytes());
    let irmin_tree = tree::add(cr, &irmin_tree, &key!("some/path2"), "value".as_bytes());
    let irmin_tree = tree::remove(cr, &irmin_tree, &key!("some/path2"));

    assert!(tree::mem(cr, &irmin_tree, &key!("some/path")));
    assert!(!tree::mem(cr, &irmin_tree, &key!("some/path2")));
    assert!(!tree::is_empty(cr, &irmin_tree));

    let tezedge_ctxt = context::add_tree(cr, &tezedge_ctxt, &key!("tree"), &tezedge_tree);
    let irmin_ctxt = context::add_tree(cr, &irmin_ctxt, &key!("tree"), &irmin_tree);

    let tezedge_commit_hash = context::commit(cr, time as i64, "commit", &tezedge_ctxt);
    let irmin_commit_hash = context::commit(cr, time as i64, "commit", &irmin_ctxt);

    assert_eq!(tezedge_commit_hash, irmin_commit_hash);

    let tezedge_ctxt = context::checkout(cr, &tezedge_index, &tezedge_commit_hash);

    assert!(tezedge_ctxt.is_some());

    let irmin_ctxt = context::checkout(cr, &irmin_index, &irmin_commit_hash);

    assert!(irmin_ctxt.is_some());

    let tezedge_ctxt = tezedge_ctxt.unwrap();

    assert!(context::find_tree(cr, &tezedge_ctxt, &key!("tree/some")).is_some());
    assert!(context::find_tree(cr, &tezedge_ctxt, &key!("tree/some/nonexistent")).is_none());
    assert!(context::find_tree(cr, &tezedge_ctxt, &key!("tree/some/path")).is_some());
    assert!(context::find_tree(cr, &tezedge_ctxt, &key!("tree")).is_some());
    assert!(context::find_tree(cr, &tezedge_ctxt, &key!("nonexistent")).is_none());
    assert!(context::find_tree(cr, &tezedge_ctxt, &vec![]).is_some());
    assert!(context::find(cr, &tezedge_ctxt, &key!("tree")).is_none());
    assert!(!context::mem(cr, &tezedge_ctxt, &key!("tree/some/path2")));
    let tv = context::find_tree(cr, &tezedge_ctxt, &key!("tree/some/path")).unwrap();
    assert!(!tree::is_empty(cr, &tv));
    assert!(!context::mem(cr, &tezedge_ctxt, &key!("tree/some")));
    assert!(context::mem(cr, &tezedge_ctxt, &key!("tree/some/path")));

    let irmin_ctxt = irmin_ctxt.unwrap();

    assert!(context::find_tree(cr, &irmin_ctxt, &key!("tree/some")).is_some());
    assert!(context::find_tree(cr, &irmin_ctxt, &key!("tree/some/nonexistent")).is_none());
    assert!(context::find_tree(cr, &irmin_ctxt, &key!("tree/some/path")).is_some());
    assert!(context::find_tree(cr, &irmin_ctxt, &key!("tree")).is_some());
    assert!(context::find_tree(cr, &irmin_ctxt, &key!("nonexistent")).is_none());
    assert!(context::find_tree(cr, &irmin_ctxt, &vec![]).is_some());
    assert!(context::find(cr, &irmin_ctxt, &key!("tree")).is_none());
    assert!(!context::mem(cr, &irmin_ctxt, &key!("tree/some/path2")));
    let tv = context::find_tree(cr, &irmin_ctxt, &key!("tree/some/path")).unwrap();
    assert!(!tree::is_empty(cr, &tv));
    assert!(!context::mem(cr, &irmin_ctxt, &key!("tree/some")));
    assert!(context::mem(cr, &irmin_ctxt, &key!("tree/some/path")));

    let tezedge_tree = context::find_tree(cr, &tezedge_ctxt, &key!("tree")).unwrap();
    let irmin_tree = context::find_tree(cr, &irmin_ctxt, &key!("tree")).unwrap();
    let tezedge_ctxt = context::add_tree(cr, &tezedge_ctxt, &key!("copy/path/tree"), &tezedge_tree);
    let irmin_ctxt = context::add_tree(cr, &irmin_ctxt, &key!("copy/path/tree"), &irmin_tree);

    assert_eq!(
        context::hash(cr, 1, None, &tezedge_ctxt),
        context::hash(cr, 1, None, &irmin_ctxt)
    );
}

#[test]
fn test_context_calls_with_runtime() {
    mavryk_interop::runtime::execute(move |rt: &mut OCamlRuntime| {
        test_context_calls(rt);
    })
    .unwrap();
}
