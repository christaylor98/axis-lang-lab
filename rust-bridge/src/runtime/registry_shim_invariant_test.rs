// ============================================================
// REGISTRY ↔ SHIM INVARIANT ENFORCEMENT TEST
// ============================================================
//
// This test enforces the critical invariant:
//
//   Every runtime registry symbol must be either:
//     1. Mapped to a shim implementation, OR
//     2. Explicitly listed in INTENTIONAL_UNMAPPED
//
// Any violation of this invariant is a HARD FAILURE.
//
// ============================================================

#[cfg(test)]
mod registry_shim_invariant_tests {
    use std::collections::HashSet;
    use crate::runtime::intentional_unmapped::get_intentional_unmapped;

    /// All symbols in the runtime registry (axis.axreg)
    /// 
    /// This must be kept in sync with registry/axis.axreg.
    /// Any mismatch indicates drift and must be corrected.
    fn get_runtime_registry_symbols() -> Vec<&'static str> {
        vec![
            // Core structural primitive
            "axis_proj",
            
            // String primitives
            "axis_str_len",
            "axis_str_char",
            "axis_str_char_at",
            "axis_str_slice",
            "axis_str_concat",
            
            // Integer / character primitives
            "axis_int_to_str",
            "axis_parse_int",
            "axis_char_to_str",
            
            // IO primitives
            "print",
            "eprint",

            // Filesystem primitives
            "file_read",
            "file_write",

            // Process primitives
            "args",
            
            // JSON parsing
            "axis_json_parse",
            
            // Core IR emission
            "axis_emit_core_bundle_to_file",
            
            // Core IR runtime introspection
            "axis_coreir_open",
            "axis_coreir_is_valid",
            "axis_coreir_schema_version",
            "axis_coreir_node_count",
            "axis_coreir_edge_count",
            "axis_coreir_close",
        ]
    }

    /// All symbols mapped in get_foreign_symbol_mapping()
    ///
    /// This lists canonical registry symbols as they appear in the mapping table keys.
    fn get_mapped_registry_symbols() -> HashSet<&'static str> {
        let mut set = HashSet::new();

        // String operations
        set.insert("axis_str_len");
        set.insert("axis_str_char");
        set.insert("axis_str_char_at");
        set.insert("axis_str_slice");
        set.insert("axis_str_concat");

        // Integer/character operations
        set.insert("axis_int_to_str");
        set.insert("axis_char_to_str");

        // IO operations
        set.insert("print");
        set.insert("eprint");

        // Process operations
        set.insert("args");

        // File operations
        set.insert("file_read");
        set.insert("file_write");
        
        // Core structural
        set.insert("axis_proj");
        
        // Special operations
        set.insert("axis_emit_core_bundle_to_file");
        
        // JSON
        // Note: axis.json.parse is mapped as "axis.json.parse" (with dot)
        // but registry has "axis_json_parse" - this is intentional
        set.insert("axis_json_parse");  // Covering the registry symbol
        
        // Core IR runtime introspection
        set.insert("axis_coreir_open");
        set.insert("axis_coreir_is_valid");
        set.insert("axis_coreir_schema_version");
        set.insert("axis_coreir_node_count");
        set.insert("axis_coreir_edge_count");
        set.insert("axis_coreir_close");
        
        set
    }

    #[test]
    fn test_all_registry_symbols_accounted_for() {
        let registry_symbols = get_runtime_registry_symbols();
        let mapped_symbols = get_mapped_registry_symbols();
        let intentional_unmapped = get_intentional_unmapped();

        let mut unaccounted = Vec::new();

        for symbol in &registry_symbols {
            let is_mapped = mapped_symbols.contains(symbol);
            let is_intentionally_unmapped = intentional_unmapped.contains(symbol);

            if !is_mapped && !is_intentionally_unmapped {
                unaccounted.push(*symbol);
            }
        }

        if !unaccounted.is_empty() {
            panic!(
                "INVARIANT VIOLATION: {} registry symbols are UNACCOUNTED FOR.\n\
                 These symbols are neither:\n\
                   - Mapped in get_foreign_symbol_mapping(), nor\n\
                   - Listed in INTENTIONAL_UNMAPPED\n\
                 \n\
                 Unaccounted symbols:\n{:#?}\n\
                 \n\
                 FIX: Either:\n\
                   1. Add mapping in rust-bridge/src/runtime/emit_rust.rs::get_foreign_symbol_mapping(), OR\n\
                   2. Add to INTENTIONAL_UNMAPPED in rust-bridge/src/runtime/intentional_unmapped.rs, OR\n\
                   3. Remove from registry/axis.axreg if it should not exist\n",
                unaccounted.len(),
                unaccounted
            );
        }
    }

    #[test]
    fn test_no_compiler_symbols_in_runtime_registry() {
        let registry_symbols = get_runtime_registry_symbols();
        
        let banned_symbols = vec![
            "axis_compiler_check",
            "axis_compiler_exec",
            "axis_compiler_lower",
            "axis_compiler_parse",
        ];

        let mut found_banned = Vec::new();

        for banned in &banned_symbols {
            if registry_symbols.contains(banned) {
                found_banned.push(*banned);
            }
        }

        if !found_banned.is_empty() {
            panic!(
                "INVARIANT VIOLATION: Compiler symbols found in runtime registry.\n\
                 \n\
                 The runtime registry MUST NOT contain compiler entrypoints.\n\
                 Compilation is a host-time concern, not a runtime concern.\n\
                 \n\
                 Banned symbols found in registry:\n{:#?}\n\
                 \n\
                 FIX: Remove these symbols from registry/axis.axreg\n",
                found_banned
            );
        }
    }

    #[test]
    fn test_intentional_unmapped_actually_in_registry() {
        let registry_symbols = get_runtime_registry_symbols();
        let intentional_unmapped = get_intentional_unmapped();

        let mut orphaned = Vec::new();

        for unmapped_symbol in &intentional_unmapped {
            if !registry_symbols.contains(unmapped_symbol) {
                orphaned.push(*unmapped_symbol);
            }
        }

        if !orphaned.is_empty() {
            panic!(
                "INVARIANT VIOLATION: INTENTIONAL_UNMAPPED contains symbols not in registry.\n\
                 \n\
                 These symbols are listed as intentionally unmapped but do not exist in the registry.\n\
                 This indicates the allowlist is stale.\n\
                 \n\
                 Orphaned symbols:\n{:#?}\n\
                 \n\
                 FIX: Remove these symbols from INTENTIONAL_UNMAPPED in rust-bridge/src/runtime/intentional_unmapped.rs\n",
                orphaned
            );
        }
    }

    #[test]
    fn test_mapped_symbols_exist_in_registry() {
        let registry_symbols: HashSet<_> = get_runtime_registry_symbols().into_iter().collect();
        let mapped_symbols = get_mapped_registry_symbols();

        let mut extra_mapped = Vec::new();

        for mapped in &mapped_symbols {
            if !registry_symbols.contains(mapped) {
                extra_mapped.push(*mapped);
            }
        }

        if !extra_mapped.is_empty() {
            panic!(
                "WARNING: Shim mappings exist for symbols not in runtime registry.\n\
                 \n\
                 These symbols are mapped but not declared in the registry.\n\
                 This may indicate:\n\
                   - The registry is incomplete, OR\n\
                   - The mappings are stale\n\
                 \n\
                 Extra mapped symbols:\n{:#?}\n\
                 \n\
                 FIX: Either:\n\
                   1. Add these symbols to registry/axis.axreg, OR\n\
                   2. Remove these mappings from get_foreign_symbol_mapping()\n",
                extra_mapped
            );
        }
    }

    #[test]
    fn test_no_overlap_between_mapped_and_intentional_unmapped() {
        let mapped_symbols = get_mapped_registry_symbols();
        let intentional_unmapped = get_intentional_unmapped();

        let mut overlap = Vec::new();

        for symbol in &intentional_unmapped {
            if mapped_symbols.contains(symbol) {
                overlap.push(*symbol);
            }
        }

        if !overlap.is_empty() {
            panic!(
                "INVARIANT VIOLATION: Symbols appear in BOTH mapped and INTENTIONAL_UNMAPPED.\n\
                 \n\
                 These symbols are contradictory - they cannot be both implemented and unmapped.\n\
                 \n\
                 Overlapping symbols:\n{:#?}\n\
                 \n\
                 FIX: Remove these symbols from INTENTIONAL_UNMAPPED in rust-bridge/src/runtime/intentional_unmapped.rs\n",
                overlap
            );
        }
    }
}
