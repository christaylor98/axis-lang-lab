// GENERATED CODE - DO NOT EDIT
// Generated from: lang-lab-poc-userfiles/lowering.yaml
// Source of truth: YAML spec file
// This code embeds the lowering spec for sealed builds

/// Embedded lowering specification
pub const LOWERING_SPEC: &str = r###"
lowering:
  rules:

    # ------------------------------------------------------------
    # Literals
    # ------------------------------------------------------------

    - match:
        node: IntLit
        value: n
      emit:
        node: CIntLit
        value: n

    - match:
        node: BoolLit
        value: b
      emit:
        node: CBoolLit
        value: b

    - match:
        node: StringLit
        value: s
      emit:
        node: CStringLit
        value: { intern: s }

    - match:
        node: UnitLit
      emit:
        node: CUnitLit


    # ------------------------------------------------------------
    # Identifiers
    # ------------------------------------------------------------

    - match:
        node: Ident
        name: x
      emit:
        node: CVar
        name: x


    # ------------------------------------------------------------
    # Projection (surface `proj`)
    # ------------------------------------------------------------

    - match:
        node: ProjExpr
        expr: e
        index: i
      emit:
        node: CProj
        expr:
          lower: e
        index: i


    # ------------------------------------------------------------
    # If expression
    # ------------------------------------------------------------

    - match:
        node: IfExpr
        cond: c
        then_branch: t
        else_branch: e
      emit:
        node: CIf
        cond:
          lower: c
        then:
          lower: t
        else:
          lower: e


    # ------------------------------------------------------------
    # Function calls
    # ------------------------------------------------------------

    # Struct literal desugaring:
    # Call("__struct_lit__", [TypeName, field1, val1, field2, val2, ...])
    - match:
        node: CallExpr
        name: "__struct_lit__"
        args: [ type_name, rest... ]
      emit:
        fold_apply:
          function:
            var_from_ident: type_name
          args:
            select_even_indices:
              from: rest
              lower: true


    # Zero-argument constructor call
    - match:
        node: CallExpr
        name: f
        args: []
      where:
        is_constructor_name: f
      emit:
        node: CVar
        name: f


    # Zero-argument non-constructor call
    - match:
        node: CallExpr
        name: f
        args: []
      emit:
        node: CApp
        fn:
          node: CVar
          name: f
        arg:
          node: CUnitLit


    # N-ary call (left-associated)
    - match:
        node: CallExpr
        name: f
        args: [a1, rest...]
      emit:
        fold_apply:
          function:
            node: CVar
            name: f
          args:
            - lower: a1
            - for_each:
                in: rest
                lower: true


    # ------------------------------------------------------------
    # Lambda (functions are curried)
    # ------------------------------------------------------------

    # Zero-parameter function
    - match:
        node: Function
        name: _
        params: []
        body: b
      emit:
        node: CLam
        param: "_unit"
        body:
          lower: b

    # One or more parameters (curried)
    - match:
        node: Function
        name: _
        params: [p1, rest...]
        body: b
      emit:
        fold_lam:
          params: [p1, rest...]
          body:
            lower: b


    # ------------------------------------------------------------
    # Blocks
    # ------------------------------------------------------------

    # Empty block
    - match:
        node: Block
        statements: []
      emit:
        node: CUnitLit

    # Single statement block
    - match:
        node: Block
        statements: [s]
      emit:
        lower_stmt: s

    # Let statement in block
    - match:
        node: Block
        statements:
          - LetStmt:
              name: x
              expr: e
          - rest...
      emit:
        node: CLet
        name: x
        value:
          lower: e
        body:
          lower_block: rest

    # Pattern let in block
    - match:
        node: Block
        statements:
          - LetPatternStmt:
              ctor: ctor
              fields: vars
              expr: rhs
          - rest...
      emit:
        desugar_pattern_let:
          ctor: ctor
          vars: vars
          rhs:
            lower: rhs
          body:
            lower_block: rest

    # Expression statement in block
    - match:
        node: Block
        statements:
          - ExprStmt:
              expr: e
          - rest...
      emit:
        node: CLet
        name: "_discard"
        value:
          lower: e
        body:
          lower_block: rest


    # ------------------------------------------------------------
    # Statements (single-statement context)
    # ------------------------------------------------------------

    - match:
        node: LetStmt
        name: x
        expr: e
      emit:
        node: CLet
        name: x
        value:
          lower: e
        body:
          node: CUnitLit

    - match:
        node: LetPatternStmt
        ctor: ctor
        fields: vars
        expr: rhs
      emit:
        desugar_pattern_let:
          ctor: ctor
          vars: vars
          rhs:
            lower: rhs
          body:
            node: CUnitLit

    - match:
        node: ExprStmt
        expr: e
      emit:
        lower: e


    # ------------------------------------------------------------
    # Match expression
    # ------------------------------------------------------------

    - match:
        node: MatchExpr
        scrutinee: s
        arms: arms
      require:
        non_empty: arms
      emit:
        node: CMatch
        scrutinee:
          lower: s
        arms:
          for_each:
            in: arms
            pattern: raw_string
            body:
              lower: expr
"###;

/// Get embedded lowering spec content
pub fn get_embedded_lowering_spec() -> &'static str {
    LOWERING_SPEC
}

