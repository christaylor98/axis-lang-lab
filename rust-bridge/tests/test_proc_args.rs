// Integration test for axis_proc_args primitive
// This test validates that axis_proc_args is callable and returns correct data

use axis_rust_bridge::runtime::shim::axis_proc_args;
use axis_rust_bridge::runtime::value::{Value, init_runtime, get_str};

#[test]
fn test_proc_args_returns_valid_list() {
    init_runtime();
    
    // Call axis_proc_args (zero-arity, pass Unit)
    let result = axis_proc_args(Value::Unit);
    
    // Should return a Value::List
    assert!(matches!(result, Value::List(_)), "axis_proc_args must return a Value::List");
    
    if let Value::List(items) = result {
        // Should contain at least one element (the program name)
        assert!(!items.is_empty(), "Process args should contain at least the program name");
        
        // All items should be strings
        for item in &items {
            assert!(matches!(item, Value::Str(_)), 
                "All process args must be Value::Str, got {:?}", item);
        }
        
        // First element should be a valid string (program path)
        if let Value::Str(handle) = items[0] {
            let program_path = get_str(handle);
            assert!(!program_path.is_empty(), "Program path should not be empty");
        }
    }
}

#[test]
fn test_proc_args_deterministic() {
    init_runtime();
    
    // Multiple calls should return the same list
    let result1 = axis_proc_args(Value::Unit);
    let result2 = axis_proc_args(Value::Unit);
    
    assert_eq!(result1, result2, "axis_proc_args must be deterministic within a single process");
}

#[test]
fn test_proc_args_preserves_order() {
    init_runtime();
    
    // Get process args
    let result = axis_proc_args(Value::Unit);
    
    if let Value::List(items) = result {
        // Convert to strings for easier comparison
        let arg_strings: Vec<String> = items.iter()
            .map(|v| {
                if let Value::Str(h) = v {
                    get_str(*h)
                } else {
                    panic!("Expected string value");
                }
            })
            .collect();
        
        // Get the real OS args for comparison
        let os_args: Vec<String> = std::env::args().collect();
        
        // Should match exactly
        assert_eq!(arg_strings, os_args, 
            "axis_proc_args must return args in exact OS order");
    } else {
        panic!("axis_proc_args must return a list");
    }
}
