// Test helper: Create valid .coreir file for integration testing
// This mimics the Language Lab's encode_capnp() logic

use capnp::message;
use capnp::serialize;
use std::fs::File;

// Re-export from lib
use axis_rust_bridge::axis_core_ir_0_2_capnp;

#[test]
fn create_simple_int_coreir() {
    // Create Cap'n Proto message
    let mut message_builder = message::Builder::new_default();
    let mut bundle_builder = message_builder.init_root::<axis_core_ir_0_2_capnp::core_bundle::Builder>();
    
    // Set version
    bundle_builder.set_version("0.2");
    
    // Create CoreTerm: IntLit(42)
    let mut term_builder = bundle_builder.init_core_term();
    let mut int_lit = term_builder.init_c_int_lit();
    int_lit.set_value(42);
    
    // Serialize to bytes
    let mut output = File::create("/tmp/int42.coreir").expect("failed to create file");
    serialize::write_message(&mut output, &message_builder).expect("failed to serialize");
    
    println!("Created /tmp/int42.coreir");
}
