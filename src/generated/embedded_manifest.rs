// GENERATED CODE - DO NOT EDIT
// Wave C: Embedded artifact manifest

/// Embedded artifact manifest (JSON)
pub const ARTIFACT_MANIFEST_JSON: &str = r###"
{
  "manifest_version": "1.0.0",
  "build_timestamp": "2026-01-24T03:42:14.370258589+00:00",
  "git_commit": "ae9540489a031195d11239a0ea8d9edf2a79a0ad",
  "git_branch": "wave6",
  "compiler_version": "rustc 1.87.0 (17067e9ac 2025-05-09)",
  "specs": {
    "parser": {
      "path": "lang-lab-poc-userfiles/parsing.yaml",
      "content_hash": "41939257bc33ed003b26be3df469cb7f53471ba01fc9def249d2067571c97228",
      "size_bytes": 1156,
      "version": null
    },
    "lowering": {
      "path": "lang-lab-poc-userfiles/lowering.yaml",
      "content_hash": "28638cf313a4bd962c94fc51f928f526fd001aae557ff134887ae3ed09d3dc3c",
      "size_bytes": 5980,
      "version": null
    },
    "lexer": {
      "path": "lang-lab-poc-userfiles/lexer.yaml",
      "content_hash": "2c023eb4e55afc7f51b0e36fea2e5dde36701a3013ec6abd1bfa6d3930812230",
      "size_bytes": 754,
      "version": null
    },
    "schema": {
      "path": "lang-lab-poc-userfiles/ast_schema.yaml",
      "content_hash": "bcc8065462469729480480a730016ddd77d20498ec7a0322e5fe42888d646d30",
      "size_bytes": 673,
      "version": null
    }
  },
  "metadata": {
    "wave": "C",
    "mode": "sealed"
  }
}
"###;

/// Get the embedded artifact manifest
pub fn get_artifact_manifest() -> crate::manifest::ArtifactManifest {
    serde_json::from_str(ARTIFACT_MANIFEST_JSON)
        .expect("embedded manifest must be valid JSON")
}
