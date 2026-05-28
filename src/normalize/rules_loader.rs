// Normalization Rules Loader
//
// PURPOSE:
// Load and parse normalization rules YAML files
//
// SCOPE:
// - YAML parsing for normalization rules
// - Rule ordering extraction
// - Pattern matching rules extraction
// - Replacement rules extraction
// - Strategy configuration extraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRules {
    pub version: String,
    pub target_nf_version: String,
    pub description: String,

    #[serde(default)]
    pub strategy: NormalizationStrategy,

    #[serde(default)]
    pub rule_order: Vec<String>,

    #[serde(default)]
    pub rules: HashMap<String, RewriteRule>,

    #[serde(default)]
    pub fresh_name_strategy: FreshNameStrategy,

    #[serde(default)]
    pub error_conditions: Vec<ErrorCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationStrategy {
    #[serde(default = "default_application_order")]
    pub application_order: String,

    #[serde(default = "default_traversal_order")]
    pub traversal_order: String,

    #[serde(default = "default_behavior")]
    pub default_behavior: String,

    #[serde(default = "default_failure_mode")]
    pub failure_mode: String,
}

fn default_application_order() -> String {
    "sequential".to_string()
}
fn default_traversal_order() -> String {
    "bottom_up".to_string()
}
fn default_behavior() -> String {
    "identity".to_string()
}
fn default_failure_mode() -> String {
    "fail_loud".to_string()
}

impl Default for NormalizationStrategy {
    fn default() -> Self {
        NormalizationStrategy {
            application_order: default_application_order(),
            traversal_order: default_traversal_order(),
            default_behavior: default_behavior(),
            failure_mode: default_failure_mode(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    pub description: String,
    pub pattern: RulePattern,
    pub replacement: RuleReplacement,

    #[serde(default)]
    pub constraints: Vec<String>,

    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RulePattern {
    SingleKind {
        node_kind: String,
        #[serde(default)]
        fields: HashMap<String, FieldConstraint>,
    },
    MultipleKinds {
        node_kind_matches: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    pub exists: bool,

    #[serde(default)]
    pub is_token: bool,

    #[serde(default)]
    pub is_node: bool,

    #[serde(default)]
    pub is_list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleReplacement {
    pub nf_node: String,

    #[serde(default)]
    pub nf_fields: ReplacementFields,

    #[serde(default)]
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReplacementFields {
    Literal(String),
    Fields(HashMap<String, FieldMapping>),
}

impl Default for ReplacementFields {
    fn default() -> Self {
        ReplacementFields::Literal("preserve_all".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub from: String,

    #[serde(default)]
    pub recurse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshNameStrategy {
    pub prefix: String,
    pub uniqueness: String,
    pub format: String,
    pub collision_handling: String,
}

impl Default for FreshNameStrategy {
    fn default() -> Self {
        FreshNameStrategy {
            prefix: "_nf_tmp".to_string(),
            uniqueness: "position_based".to_string(),
            format: "{prefix}_{position}".to_string(),
            collision_handling: "fail_loud".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCondition {
    pub condition: String,
    pub action: String,
}

impl NormalizationRules {
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read normalization rules file: {}", e))?;

        let rules: NormalizationRules = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse normalization rules YAML: {}", e))?;

        // Validate version
        if rules.version.is_empty() {
            return Err("Normalization rules version cannot be empty".to_string());
        }

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_ai2_normalization_rules() {
        let path = Path::new("axis-surface-ai2-config/ai2-explicit-normalize.yaml");
        if path.exists() {
            let result = NormalizationRules::load_from_file(path);
            assert!(
                result.is_ok(),
                "Failed to load AI2 normalization rules: {:?}",
                result.err()
            );

            let rules = result.unwrap();
            assert_eq!(rules.version, "0.1");
            assert_eq!(rules.target_nf_version, "0.1");
            assert!(
                !rules.rule_order.is_empty(),
                "Rule order should not be empty"
            );
        }
    }

    #[test]
    fn test_load_h1_normalization_rules() {
        let path = Path::new("axis-surface-h1-config/surface-h1-normalize.yaml");
        if path.exists() {
            let result = NormalizationRules::load_from_file(path);
            assert!(
                result.is_ok(),
                "Failed to load H1 normalization rules: {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn test_load_surface0_normalization_rules() {
        let path = Path::new("axis-surface-0-config/surface-0-normalize.yaml");
        if path.exists() {
            let result = NormalizationRules::load_from_file(path);
            assert!(
                result.is_ok(),
                "Failed to load Surface-0 normalization rules: {:?}",
                result.err()
            );
        }
    }

    #[test]
    fn test_load_ai1_normalization_rules() {
        let path = Path::new("axis-surface-ai1-config/ai1-rpn-normalize.yaml");
        if path.exists() {
            let result = NormalizationRules::load_from_file(path);
            assert!(
                result.is_ok(),
                "Failed to load AI1 normalization rules: {:?}",
                result.err()
            );

            let rules = result.unwrap();
            assert_eq!(rules.version, "0.1");
            assert_eq!(rules.target_nf_version, "0.1");
            assert!(
                !rules.rule_order.is_empty(),
                "Rule order should not be empty"
            );
        }
    }
}
