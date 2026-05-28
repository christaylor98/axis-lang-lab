// Wave C: Artifact manifest generation and embedding
// Provides deterministic build metadata for sealed artifacts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Artifact manifest - complete provenance and version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    /// Manifest format version
    pub manifest_version: String,

    /// Build timestamp (ISO 8601)
    pub build_timestamp: String,

    /// Git commit hash (if available)
    pub git_commit: Option<String>,

    /// Git branch (if available)
    pub git_branch: Option<String>,

    /// Compiler version (rustc)
    pub compiler_version: String,

    /// Spec versions and hashes
    pub specs: HashMap<String, SpecInfo>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Spec information - version and content hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInfo {
    /// Spec file path (relative to project root)
    pub path: String,

    /// Content hash (SHA-256)
    pub content_hash: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Optional version tag
    pub version: Option<String>,
}

impl ArtifactManifest {
    /// Create a new manifest with current build information
    pub fn new() -> Self {
        use chrono::Utc;

        Self {
            manifest_version: "1.0.0".to_string(),
            build_timestamp: Utc::now().to_rfc3339(),
            git_commit: Self::get_git_commit(),
            git_branch: Self::get_git_branch(),
            compiler_version: Self::get_compiler_version(),
            specs: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a spec to the manifest
    pub fn add_spec(
        &mut self,
        name: String,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read(path)?;
        let hash = Self::compute_hash(&content);

        self.specs.insert(
            name,
            SpecInfo {
                path: path.to_string_lossy().to_string(),
                content_hash: hash,
                size_bytes: content.len() as u64,
                version: None,
            },
        );

        Ok(())
    }

    /// Compute SHA-256 hash of content
    fn compute_hash(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Get git commit hash
    fn get_git_commit() -> Option<String> {
        std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Get git branch name
    fn get_git_branch() -> Option<String> {
        std::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Get rustc version
    fn get_compiler_version() -> String {
        std::process::Command::new("rustc")
            .args(&["--version"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Generate Rust code embedding this manifest
    pub fn generate_embedded_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json = self.to_json()?;

        let mut code = String::new();
        code.push_str("// GENERATED CODE - DO NOT EDIT\n");
        code.push_str("// Wave C: Embedded artifact manifest\n");
        code.push_str("\n");
        code.push_str("/// Embedded artifact manifest (JSON)\n");
        code.push_str("pub const ARTIFACT_MANIFEST_JSON: &str = r###\"\n");
        code.push_str(&json);
        code.push_str("\n\"###;\n");
        code.push_str("\n");
        code.push_str("/// Get the embedded artifact manifest\n");
        code.push_str("pub fn get_artifact_manifest() -> crate::manifest::ArtifactManifest {\n");
        code.push_str("    serde_json::from_str(ARTIFACT_MANIFEST_JSON)\n");
        code.push_str("        .expect(\"embedded manifest must be valid JSON\")\n");
        code.push_str("}\n");

        Ok(code)
    }
}

impl Default for ArtifactManifest {
    fn default() -> Self {
        Self::new()
    }
}
