// Generated code modules

pub mod axis_core_ir_0_3_capnp {
    include!(concat!(
        env!("OUT_DIR"),
        "/core_ir_spec/axis_core_ir_0_3_capnp.rs"
    ));
}

// Wave C: Embedded specs (only available in sealed mode)
#[cfg(feature = "sealed")]
include!("embedded_mod.rs");
