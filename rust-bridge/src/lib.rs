// axis-rust-bridge library

pub mod cli_util;

// Generated Cap'n Proto schema
pub mod axis_core_ir_0_3_capnp {
    include!(concat!(env!("OUT_DIR"), "/core_ir_spec/axis_core_ir_0_3_capnp.rs"));
}

pub mod core_ir;
pub mod core_loader;
pub mod executor;
pub mod runtime;
pub use runtime::emit_rust;
