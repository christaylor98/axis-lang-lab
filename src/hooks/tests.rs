// WAVE 4: Compiler Hook Framework Tests
//
// Tests for hook registration, ordering, conflict detection, and execution.

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;

    // ═══════════════════════════════════════════════════════════════════════════
    // TEST HOOKS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Simple test hook that does nothing
    #[derive(Debug)]
    struct IdentityHook {
        name: String,
        stage: HookStage,
    }

    impl Hook for IdentityHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn stage(&self) -> HookStage {
            self.stage
        }

        fn permission(&self) -> HookPermission {
            HookPermission::ReadOnly
        }

        fn execute(&self, input: HookInput, _context: &HookContext) -> HookExecutionResult {
            Ok(HookResult {
                output: match input {
                    HookInput::Text(text) => HookOutput::Text(text),
                    HookInput::Ast(ast) => HookOutput::Ast(ast),
                },
                metadata: None,
            })
        }
    }

    /// Test hook for normalisation stage
    #[derive(Debug)]
    struct NormalisationTestHook {
        name: String,
        pass: Option<String>,
    }

    impl Hook for NormalisationTestHook {
        fn name(&self) -> &str {
            &self.name
        }

        fn stage(&self) -> HookStage {
            HookStage::Normalisation
        }

        fn permission(&self) -> HookPermission {
            HookPermission::Rewrite
        }

        fn normalisation_pass(&self) -> Option<&str> {
            self.pass.as_deref()
        }

        fn execute(&self, input: HookInput, _context: &HookContext) -> HookExecutionResult {
            // Identity transformation
            Ok(HookResult {
                output: match input {
                    HookInput::Ast(ast) => HookOutput::Ast(ast),
                    _ => {
                        return Err(HookError {
                            message: "Expected AST input".to_string(),
                            hook_name: self.name.clone(),
                            stage: HookStage::Normalisation,
                            span: None,
                        })
                    }
                },
                metadata: Some(serde_json::json!({
                    "hook": self.name,
                    "pass": self.pass,
                })),
            })
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // REGISTRATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_empty_registry() {
        let registry = HookRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_hook() {
        let mut registry = HookRegistry::new();

        let hook = Arc::new(IdentityHook {
            name: "test-hook".to_string(),
            stage: HookStage::PreLex,
        });

        let result = registry.register(hook, OrderingHint::None);
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_duplicate_name_error() {
        let mut registry = HookRegistry::new();

        let hook1 = Arc::new(IdentityHook {
            name: "duplicate".to_string(),
            stage: HookStage::PreLex,
        });

        let hook2 = Arc::new(IdentityHook {
            name: "duplicate".to_string(),
            stage: HookStage::PostParse,
        });

        registry.register(hook1, OrderingHint::None).unwrap();
        let result = registry.register(hook2, OrderingHint::None);

        assert!(result.is_err());
        match result {
            Err(HookConflict::DuplicateName { name, .. }) => {
                assert_eq!(name, "duplicate");
            }
            _ => panic!("Expected DuplicateName error"),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ORDERING TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_stage_ordering() {
        let mut registry = HookRegistry::new();

        // Register hooks in reverse stage order
        registry
            .register(
                Arc::new(IdentityHook {
                    name: "post-norm".to_string(),
                    stage: HookStage::PostNormalisation,
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "pre-lex".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "post-parse".to_string(),
                    stage: HookStage::PostParse,
                }),
                OrderingHint::None,
            )
            .unwrap();

        // Verify hooks are retrievable by stage
        let pre_lex = registry.get_hooks_for_stage(HookStage::PreLex).unwrap();
        assert_eq!(pre_lex.len(), 1);
        assert_eq!(pre_lex[0].name(), "pre-lex");

        let post_parse = registry.get_hooks_for_stage(HookStage::PostParse).unwrap();
        assert_eq!(post_parse.len(), 1);
        assert_eq!(post_parse[0].name(), "post-parse");
    }

    #[test]
    fn test_normalisation_pass_filtering() {
        let mut registry = HookRegistry::new();

        // Register hooks for different normalisation passes
        registry
            .register(
                Arc::new(NormalisationTestHook {
                    name: "n1-hook".to_string(),
                    pass: Some("N1".to_string()),
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(NormalisationTestHook {
                    name: "n2-hook".to_string(),
                    pass: Some("N2".to_string()),
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(NormalisationTestHook {
                    name: "all-passes-hook".to_string(),
                    pass: None,
                }),
                OrderingHint::None,
            )
            .unwrap();

        // Get hooks for N1
        let n1_hooks = registry.get_hooks_for_normalisation_pass("N1").unwrap();
        assert_eq!(n1_hooks.len(), 2); // n1-hook + all-passes-hook

        // Get hooks for N2
        let n2_hooks = registry.get_hooks_for_normalisation_pass("N2").unwrap();
        assert_eq!(n2_hooks.len(), 2); // n2-hook + all-passes-hook

        // Get hooks for N3 (only all-passes-hook)
        let n3_hooks = registry.get_hooks_for_normalisation_pass("N3").unwrap();
        assert_eq!(n3_hooks.len(), 1);
        assert_eq!(n3_hooks[0].name(), "all-passes-hook");
    }

    #[test]
    fn test_name_ordering() {
        let mut registry = HookRegistry::new();

        // Register hooks with lexicographically unordered names
        registry
            .register(
                Arc::new(IdentityHook {
                    name: "zebra".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "apple".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "mango".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::None,
            )
            .unwrap();

        // Get hooks - should be ordered by name
        let hooks = registry.get_hooks_for_stage(HookStage::PreLex).unwrap();
        assert_eq!(hooks.len(), 3);
        assert_eq!(hooks[0].name(), "apple");
        assert_eq!(hooks[1].name(), "mango");
        assert_eq!(hooks[2].name(), "zebra");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VALIDATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validation_missing_ordering_target() {
        let mut registry = HookRegistry::new();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "first".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::After("nonexistent".to_string()),
            )
            .unwrap();

        let result = registry.validate();
        assert!(result.is_err());
        match result {
            Err(HookConflict::MissingOrderingTarget {
                hook_name,
                target_name,
            }) => {
                assert_eq!(hook_name, "first");
                assert_eq!(target_name, "nonexistent");
            }
            _ => panic!("Expected MissingOrderingTarget error"),
        }
    }

    #[test]
    fn test_validation_cross_stage_ordering() {
        let mut registry = HookRegistry::new();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "pre-lex-hook".to_string(),
                    stage: HookStage::PreLex,
                }),
                OrderingHint::None,
            )
            .unwrap();

        registry
            .register(
                Arc::new(IdentityHook {
                    name: "post-parse-hook".to_string(),
                    stage: HookStage::PostParse,
                }),
                OrderingHint::Before("pre-lex-hook".to_string()),
            )
            .unwrap();

        // Should fail - ordering hints only apply within same stage
        let result = registry.validate();
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXECUTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_hook_execution() {
        let hook: Arc<dyn Hook> = Arc::new(IdentityHook {
            name: "test".to_string(),
            stage: HookStage::PreLex,
        });

        let context = HookContext {
            stage: HookStage::PreLex,
            normalisation_pass: None,
            permission: HookPermission::ReadOnly,
        };

        let input = HookInput::Text("test input".to_string());
        let (result, execution) = HookExecutor::execute(&hook, input, &context);

        assert!(result.is_ok());
        assert_eq!(execution.hook_name, "test");
        assert_eq!(execution.stage, "pre-lex");
        assert!(!execution.modified); // Identity hook doesn't modify
    }

    #[test]
    fn test_sequence_execution() {
        let hooks = vec![
            Arc::new(IdentityHook {
                name: "first".to_string(),
                stage: HookStage::PreLex,
            }) as Arc<dyn Hook>,
            Arc::new(IdentityHook {
                name: "second".to_string(),
                stage: HookStage::PreLex,
            }) as Arc<dyn Hook>,
        ];

        let context = HookContext {
            stage: HookStage::PreLex,
            normalisation_pass: None,
            permission: HookPermission::ReadOnly,
        };

        let input = HookInput::Text("test".to_string());
        let (result, trace) = HookExecutor::execute_sequence(hooks, input, &context);

        assert!(result.is_ok());
        assert_eq!(trace.executions.len(), 2);
        assert_eq!(trace.executions[0].hook_name, "first");
        assert_eq!(trace.executions[1].hook_name, "second");
    }
}
