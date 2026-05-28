// Example external library providing axis_fn_1001
// Compile: rustc --crate-type=staticlib --extern axis_rust_bridge=../rust-bridge/target/debug/libaxis_rust_bridge.rlib examples/libaxis_example.rs -o examples/libaxis_example.a -L dependency=../rust-bridge/target/debug/deps

extern crate axis_rust_bridge;
use axis_rust_bridge::runtime::value::Value;

/// Example implementation of axis_fn_1001: integer addition
/// CCall(1001, [a, b]) → a + b
#[no_mangle]
pub extern "C" fn axis_fn_1001(args: Value) -> Value {
    // Expect a 2-tuple of integers
    match args {
        Value::Tuple(ref vec) if vec.len() == 2 => {
            if let (Value::Int(a), Value::Int(b)) = (&vec[0], &vec[1]) {
                return Value::Int(a + b);
            }
        }
        _ => {}
    }
    panic!("axis_fn_1001: expected 2-tuple of integers, got {:?}", args);
}

/// Example implementation of axis_fn_1002: integer multiplication
/// CCall(1002, [a, b]) → a * b
#[no_mangle]
pub extern "C" fn axis_fn_1002(args: Value) -> Value {
    match args {
        Value::Tuple(ref vec) if vec.len() == 2 => {
            if let (Value::Int(a), Value::Int(b)) = (&vec[0], &vec[1]) {
                return Value::Int(a * b);
            }
        }
        _ => {}
    }
    panic!("axis_fn_1002: expected 2-tuple of integers, got {:?}", args);
}

/// Example implementation of axis_fn_2001: print to stdout
/// CCall(2001, [value]) → prints value and returns unit
#[no_mangle]
pub extern "C" fn axis_fn_2001(args: Value) -> Value {
    println!("axis_fn_2001: {:?}", args);
    Value::Unit
}
