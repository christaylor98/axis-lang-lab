// ============================================================
// RUNTIME REGISTRY INTENTIONAL UNMAPPED SYMBOLS
// ============================================================
//
// This file declares the INTENTIONAL_UNMAPPED allowlist.
//
// INVARIANT:
// Every symbol in the runtime registry MUST be either:
//   1. Mapped to a shim implementation, OR
//   2. Listed here as intentionally unmapped
//
// Any symbol not meeting these criteria is a BUG.
//
// ============================================================

use std::collections::HashSet;

/// Returns the set of registry symbols that are intentionally unmapped.
///
/// These symbols represent planned runtime functionality that does not yet
/// have a corresponding shim implementation. They are retained in the registry
/// to document the intended API surface but will panic if called at runtime.
///
/// Each entry must have a comment explaining WHY it is unmapped.
pub fn get_intentional_unmapped() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    
    // axis_parse_int: Planned standard runtime function for parsing strings to integers.
    // This is a legitimate runtime operation (user input/data parsing).
    // Currently unmapped, awaiting implementation.
    // TODO: Implement axis_parse_int in shim and remove from this list.
    set.insert("axis_parse_int");
    
    set
}

/// Returns all symbols that should be mapped in get_foreign_symbol_mapping().
///
/// This is the union of:
///   - All runtime registry symbols
///   - Minus the intentional unmapped symbols
pub fn get_required_mappings<'a>(registry_symbols: &'a [&'a str]) -> Vec<&'a str> {
    let unmapped = get_intentional_unmapped();
    registry_symbols
        .iter()
        .filter(|s| !unmapped.contains(**s))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intentional_unmapped_is_small() {
        let unmapped = get_intentional_unmapped();
        
        // The allowlist should be SHORT and EXPLICIT.
        // If this grows beyond 5 items, something is wrong.
        assert!(
            unmapped.len() <= 5,
            "INTENTIONAL_UNMAPPED allowlist has {} items, expected ≤ 5. This indicates drift from runtime/compiler boundary.",
            unmapped.len()
        );
    }

    #[test]
    fn test_axis_parse_int_is_intentionally_unmapped() {
        let unmapped = get_intentional_unmapped();
        assert!(
            unmapped.contains("axis_parse_int"),
            "axis_parse_int should be in INTENTIONAL_UNMAPPED allowlist"
        );
    }

    #[test]
    fn test_compiler_symbols_not_in_allowlist() {
        let unmapped = get_intentional_unmapped();
        
        // These should NOT be in the allowlist - they should be DELETED from registry
        assert!(!unmapped.contains("axis_compiler_check"));
        assert!(!unmapped.contains("axis_compiler_exec"));
        assert!(!unmapped.contains("axis_compiler_lower"));
        assert!(!unmapped.contains("axis_compiler_parse"));
    }
}
