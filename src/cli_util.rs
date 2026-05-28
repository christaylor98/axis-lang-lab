/// CLI Input Normalization Utilities
///
/// Provides deterministic, set-based input handling for the Axis toolchain.
/// All path inputs are:
/// - Expanded (files and directories)
/// - Sorted deterministically
/// - Deduplicated
/// - Canonicalized
///
/// This module ensures that CLI semantics are idempotent and order-independent,
/// except for default name derivation which uses the FIRST resolved input.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Normalize a list of path inputs into a deterministic, deduplicated set.
///
/// # Process:
/// 1. Expand each path (file → that file, directory → recursive walk)
/// 2. Sort all expanded paths
/// 3. Canonicalize or normalize paths
/// 4. Insert into HashSet for deduplication
///
/// # Returns:
/// A vector of unique paths in deterministic order.
///
/// # Note:
/// Repeated inputs are silently deduplicated with no warnings or diagnostics.
pub fn normalize_path_inputs(
    inputs: &[PathBuf],
    extension_filter: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }

    let mut expanded = Vec::new();

    // Step 1: Expand all inputs
    for path in inputs {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if path.is_file() {
            expanded.push(path.clone());
        } else if path.is_dir() {
            // Recursive walk
            walk_dir_recursive(path, &mut expanded, extension_filter)?;
        }
    }

    // Step 2: Sort deterministically (lexicographic)
    expanded.sort();

    // Step 3: Canonicalize paths
    let mut canonicalized = Vec::new();
    for path in &expanded {
        match path.canonicalize() {
            Ok(canonical) => canonicalized.push(canonical),
            Err(_) => {
                // Fallback to normalized relative path if canonicalization fails
                canonicalized.push(normalize_path(path));
            }
        }
    }

    // Step 4: Deduplicate using HashSet
    let unique_set: HashSet<PathBuf> = canonicalized.into_iter().collect();

    // Step 5: Return as sorted vector for determinism
    let mut result: Vec<PathBuf> = unique_set.into_iter().collect();
    result.sort();

    Ok(result)
}

/// Recursively walk a directory and collect all files matching the optional extension filter.
fn walk_dir_recursive(
    dir: &Path,
    output: &mut Vec<PathBuf>,
    extension_filter: Option<&str>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        format!("Failed to read directory {}: {}", dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!("Failed to read directory entry in {}: {}", dir.display(), e)
        })?;

        let path = entry.path();

        if path.is_file() {
            // Apply extension filter if specified
            if let Some(ext) = extension_filter {
                if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                    output.push(path);
                }
            } else {
                output.push(path);
            }
        } else if path.is_dir() {
            walk_dir_recursive(&path, output, extension_filter)?;
        }
    }

    Ok(())
}

/// Normalize a path (make it clean and consistent).
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

/// Derive a default name from the first resolved input path.
///
/// # Returns:
/// The basename (filename without extension) of the first path.
pub fn derive_name_from_first_input(inputs: &[PathBuf]) -> Option<String> {
    inputs.first().and_then(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_deduplicates() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("test.ax");
        fs::write(&file1, "content").unwrap();

        // Same file specified twice
        let inputs = vec![file1.clone(), file1.clone()];
        let result = normalize_path_inputs(&inputs, None).unwrap();

        // Should deduplicate to one file
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_directory_expansion() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path();
        
        fs::write(dir.join("a.ax"), "a").unwrap();
        fs::write(dir.join("b.ax"), "b").unwrap();
        
        let subdir = dir.join("sub");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("c.ax"), "c").unwrap();

        let inputs = vec![dir.to_path_buf()];
        let result = normalize_path_inputs(&inputs, Some("ax")).unwrap();

        // Should find all 3 .ax files
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_derive_name() {
        let path = PathBuf::from("path/to/myfile.ax");
        let name = derive_name_from_first_input(&[path]);
        assert_eq!(name, Some("myfile".to_string()));
    }
}
