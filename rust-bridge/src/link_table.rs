/// Link table management for external library symbols
/// 
/// The link table maps Core IR numeric function IDs to external symbol names
/// and library locations. This is bridge-owned metadata, disposable and non-authoritative.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const LINK_TABLE_DIR: &str = ".axis";
const LINK_TABLE_FILE: &str = "bridge-link.toml";

/// Link table structure (persisted as TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTable {
    pub link_table: LinkTableData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTableData {
    pub version: String,
    #[serde(default)]
    pub function: Vec<FunctionMapping>,
    #[serde(default)]
    pub search_path: Vec<SearchPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMapping {
    pub id: u64,
    pub symbol: String,
    pub library: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPath {
    pub path: String,
}

impl LinkTable {
    /// Create a new empty link table
    pub fn new() -> Self {
        LinkTable {
            link_table: LinkTableData {
                version: "0.1".to_string(),
                function: Vec::new(),
                search_path: Vec::new(),
            },
        }
    }

    /// Load link table from disk (returns empty table if not found)
    pub fn load() -> Result<Self, String> {
        let path = Self::get_path();
        
        if !path.exists() {
            // No link table exists yet - return empty
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read link table from {}: {}", path.display(), e))?;

        toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse link table: {}", e))
    }

    /// Save link table to disk
    pub fn save(&self) -> Result<(), String> {
        let dir_path = PathBuf::from(LINK_TABLE_DIR);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create .axis directory: {}", e))?;
        }

        let path = Self::get_path();
        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize link table: {}", e))?;

        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write link table to {}: {}", path.display(), e))
    }

    /// Get the path to the link table file
    fn get_path() -> PathBuf {
        PathBuf::from(LINK_TABLE_DIR).join(LINK_TABLE_FILE)
    }

    /// Build a lookup map for fast symbol resolution
    pub fn build_lookup(&self) -> HashMap<u64, (String, String)> {
        let mut map = HashMap::new();
        for func in &self.link_table.function {
            map.insert(func.id, (func.symbol.clone(), func.library.clone()));
        }
        map
    }

    /// Generate deterministic symbol name for a numeric ID (fallback)
    pub fn deterministic_symbol(id: u64) -> String {
        format!("axis_fn_{}", id)
    }

    /// Resolve symbol name for a function ID
    /// Returns (symbol_name, library_name_option)
    pub fn resolve_symbol(&self, id: u64) -> (String, Option<String>) {
        let lookup = self.build_lookup();
        match lookup.get(&id) {
            Some((symbol, lib)) => (symbol.clone(), Some(lib.clone())),
            None => (Self::deterministic_symbol(id), None),
        }
    }

    /// Add or update a function mapping (non-destructive merge)
    pub fn add_function(&mut self, id: u64, symbol: String, library: String) {
        // Check if mapping already exists
        for func in &mut self.link_table.function {
            if func.id == id {
                // Update existing
                func.symbol = symbol;
                func.library = library;
                return;
            }
        }
        
        // Add new
        self.link_table.function.push(FunctionMapping {
            id,
            symbol,
            library,
        });
    }

    /// Add search path if not already present
    pub fn add_search_path(&mut self, path: String) {
        if !self.link_table.search_path.iter().any(|p| p.path == path) {
            self.link_table.search_path.push(SearchPath { path });
        }
    }

    /// Get all unique library names referenced in the table
    pub fn get_libraries(&self) -> Vec<String> {
        let mut libs: Vec<String> = self.link_table.function
            .iter()
            .map(|f| f.library.clone())
            .collect();
        libs.sort();
        libs.dedup();
        libs
    }

    /// Get all search paths
    pub fn get_search_paths(&self) -> Vec<String> {
        self.link_table.search_path.iter().map(|p| p.path.clone()).collect()
    }

    /// Scan a directory for native libraries and extract symbol metadata
    /// This is opportunistic and non-destructive
    pub fn scan_directory(&mut self, dir_path: &Path) -> Result<usize, String> {
        if !dir_path.is_dir() {
            return Err(format!("{} is not a directory", dir_path.display()));
        }

        let mut added_count = 0;

        // Recursively walk directory looking for .a, .so, .dylib files
        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;

        for entry_result in entries {
            let entry = entry_result
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;
            
            let path = entry.path();
            
            if path.is_dir() {
                // Recursive scan
                added_count += self.scan_directory(&path)?;
            } else if path.is_file() {
                // Check if it's a native library
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str == "a" || ext_str == "so" || ext_str == "dylib" {
                        // Found a library - add to search paths
                        if let Some(parent) = path.parent() {
                            self.add_search_path(parent.to_string_lossy().to_string());
                            added_count += 1;
                        }
                    }
                }
            }
        }

        Ok(added_count)
    }
}

impl Default for LinkTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_symbol() {
        assert_eq!(LinkTable::deterministic_symbol(1001), "axis_fn_1001");
        assert_eq!(LinkTable::deterministic_symbol(0), "axis_fn_0");
    }

    #[test]
    fn test_resolve_symbol() {
        let mut table = LinkTable::new();
        table.add_function(1001, "custom_symbol".to_string(), "libfoo.a".to_string());

        let (sym, lib) = table.resolve_symbol(1001);
        assert_eq!(sym, "custom_symbol");
        assert_eq!(lib, Some("libfoo.a".to_string()));

        let (sym, lib) = table.resolve_symbol(9999);
        assert_eq!(sym, "axis_fn_9999");
        assert_eq!(lib, None);
    }

    #[test]
    fn test_add_function_merge() {
        let mut table = LinkTable::new();
        table.add_function(1001, "sym1".to_string(), "lib1".to_string());
        table.add_function(1001, "sym2".to_string(), "lib2".to_string());

        assert_eq!(table.link_table.function.len(), 1);
        assert_eq!(table.link_table.function[0].symbol, "sym2");
    }
}
