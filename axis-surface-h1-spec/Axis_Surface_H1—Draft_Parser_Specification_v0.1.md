## 1) H1 Parser Grammar (EBNF) — v0.1

Notes:

* This is **expression-oriented**: blocks and `if/match` yield values.
* Statements are only `let`, intent directives, and expression statements.
* `run` is a **top-level** item (`run <ident> ;`).
* Function parameter types and return types are **omitted** in v0.1 (keep it simple).
* Comparisons / boolean ops are optional; included below as a minimal precedence ladder (you can drop if you want).

### Lexical terminals (assumed)

* Keywords: `let fn if else loop for in match spawn join channel send recv controller on run`
* Punct: `(` `)` `{` `}` `[` `]` `,` `;` `:` `.` `@` `<` `>`
* Ops: `=` `=>` `==` `!=` `<=` `>=` `+` `-` `*` `/`
* Literals: `INT`, `STRING`, `RAW_STRING`
* `IDENT` and `_` (underscore token may be lexed as IDENT "_" or a distinct token)

### EBNF

```ebnf
Program        ::= (Item | NL)* EOF ;

Item           ::= FnDecl
                 | ControllerDecl
                 | RunStmt
                 | StmtTop ;               (* optional: allow top-level statements in v0.1 *)

RunStmt        ::= "run" IDENT ";" ;

FnDecl         ::= "fn" IDENT "(" ParamList? ")" Block ;

ParamList      ::= Param ("," Param)* (",")? ;
Param          ::= IDENT ;

ControllerDecl ::= "controller" IDENT "{" ControllerMember* "}" ;
ControllerMember ::= "on" EventSig "=>" Expr "," ;
EventSig       ::= IDENT "(" ParamList? ")" ;

StmtTop        ::= Stmt ;                  (* if you allow statements at top-level *)

Block          ::= "{" Stmt* Expr? "}" ;   (* Expr? allows empty block; if absent => Unit *)

Stmt           ::= LetStmt
                 | IntentDirectiveStmt
                 | ExprStmt ;

LetStmt        ::= "let" IDENT "=" Expr ";" ;

IntentDirectiveStmt ::= IntentDirective Terminator ;
Terminator     ::= ";" | NL ;              (* NL is conceptual; parser can treat NL as terminator token or equivalent *)

ExprStmt       ::= Expr ";" ;

IntentDirective ::= "@" "intent" "." IDENT ;

Expr           ::= IfExpr
                 | MatchExpr
                 | ForExpr
                 | LoopExpr
                 | SpawnExpr
                 | JoinExpr
                 | SendExpr
                 | RecvExpr
                 | ChannelExpr
                 | BinaryExpr ;            (* precedence ladder *)

IfExpr         ::= "if" Expr Block "else" Block ;

MatchExpr      ::= "match" Expr "{" MatchArm+ "}" ;
MatchArm       ::= Pattern "=>" Expr "," ;

Pattern        ::= "_" 
                 | Literal
                 | IdentPat
                 | CtorPat ;

IdentPat       ::= IDENT ;
CtorPat        ::= IDENT "(" PatList? ")" ;
PatList        ::= Pattern ("," Pattern)* (",")? ;

ForExpr        ::= "for" IDENT "in" Expr Block ;
LoopExpr       ::= "loop" Block ;

SpawnExpr      ::= "spawn" Block ;

JoinExpr       ::= "join" Expr ;

ChannelExpr    ::= "channel" GenericArgs? "(" ")" ;    (* channel<T>() or channel() *)
GenericArgs    ::= "<" TypeName ">" ;
TypeName       ::= IDENT ;                              (* v0.1 minimal *)

SendExpr       ::= "send" "(" Expr "," Expr ")" ;
RecvExpr       ::= "recv" "(" Expr ")" ;

BinaryExpr     ::= EqualityExpr ;

EqualityExpr   ::= RelExpr (("==" | "!=") RelExpr)* ;
RelExpr        ::= AddExpr (("<=" | ">=" | "<" | ">") AddExpr)* ;
AddExpr        ::= MulExpr (("+" | "-") MulExpr)* ;
MulExpr        ::= UnaryExpr (("*" | "/") UnaryExpr)* ;

UnaryExpr      ::= ("-" | "!") UnaryExpr
                 | CallExpr ;

CallExpr       ::= Primary (CallSuffix)* ;
CallSuffix     ::= "(" ArgList? ")" ;
ArgList        ::= Expr ("," Expr)* (",")? ;

Primary        ::= Literal
                 | IDENT
                 | Block
                 | "(" Expr ")" ;

Literal        ::= INT
                 | STRING
                 | RAW_STRING ;
```

**Implementation note:** If you don’t want comparisons/operators in v0.1, drop `BinaryExpr` ladder and treat everything as function calls only; keep parens and calls.

---

## 2) Intent-attachment rules (parser phase) — v0.1

Goal: `@intent.*` is **first-class syntax** that attaches to the **next construct** (loop or block), becomes **metadata**, and is ignored by lowering.

### 2.1 Allowed intent keys (v0.1 closed set)

* `parallel`
* `serial`

Unknown keys:

* **hard parser error** in v0.1

### 2.2 Where intent directives may appear (v0.1)

Intent directives may precede:

* `for` loops
* `loop` loops
* `{ ... }` blocks (optional in v0.1; include if you want scoped serial regions)

They may appear:

* at statement positions inside blocks
* at top level (if you allow top-level statements)

### 2.3 Termination and shape (enforced)

A directive is:

```
@ intent . <key>  (Terminator)
```

Terminator is:

* newline, or
* semicolon

No inline form:

* `@intent.parallel for ...` is **invalid**

### 2.4 Attachment algorithm (deterministic)

Maintain a short-lived `pending_intents: Vec<Intent>` in the parser.

Process tokens:

1. When you parse an `IntentDirectiveStmt`, push intent into `pending_intents`.
2. The **next parsed attachable construct** consumes `pending_intents` and attaches them as metadata on that node.
3. After attachment, clear `pending_intents`.

Attachable constructs (v0.1):

* `ForExpr`
* `LoopExpr`
* (optional) `Block` when it appears in statement position: `@intent.serial { ... }`

### 2.5 Error rules

* If `pending_intents` is non-empty and you encounter a non-attachable construct:

  * **error**: “intent directive must apply to the next loop/block”
* If multiple intents conflict on the same target (e.g. both parallel and serial):

  * **error**: “conflicting intents”
* If a directive appears at end-of-block / end-of-file without a following attachable construct:

  * **error**: “dangling intent directive”

### 2.6 Metadata emission

On the attachable node, emit metadata like:

* `@intent.parallel = true`
* `@intent.serial = true`

Representation:

* key: `intent.parallel` / `intent.serial`
* value: boolean true

Lowering must ignore it. Bridge may use it.

---

## 3) Reference lexer test corpus — v0.1

Format: each test has **input** and expected **token classes** (not enum names). This is designed to be copied into `tests/lexer/` as `.h1` files plus a golden token dump.

### 3.1 Basic tokens and comments

**T01_basic_let.h1**

```h1
let x = 123;
```

Expect tokens:

* KW(let), IDENT(x), OP(=), INT(123), SEMI, EOF

**T02_line_comment.h1**

```h1
let x = 1; // comment
let y = 2;
```

Expect:

* KW(let) IDENT OP(=) INT SEMI
* KW(let) IDENT OP(=) INT SEMI
* EOF

**T03_block_comment_nested.h1**

```h1
/* outer
   /* inner */
   still outer
*/
let x = 1;
```

Expect:

* KW(let) IDENT OP(=) INT SEMI EOF

**T04_unterminated_block_comment.h1**

```h1
/* no end
let x = 1;
```

Expect:

* LEXER ERROR: unterminated block comment

---

### 3.2 Strings (including multiline)

**T10_string_single_line.h1**

```h1
let s = "hello";
```

Expect:

* KW(let) IDENT OP(=) STRING("hello") SEMI EOF

**T11_string_multiline.h1**

```h1
let s = "hello
world";
```

Expect:

* KW(let) IDENT OP(=) STRING(multiline) SEMI EOF
  Notes: STRING token span includes newline.

**T12_string_escapes.h1**

```h1
let s = "a\nb\t\"c\"\\";
```

Expect:

* KW(let) IDENT OP(=) STRING SEMI EOF

**T13_string_bad_escape.h1**

```h1
let s = "\q";
```

Expect:

* LEXER ERROR: unknown escape

**T14_raw_string_multiline.h1**

```h1
let s = r"
line1
line2
";
```

Expect:

* KW(let) IDENT OP(=) RAW_STRING SEMI EOF

**T15_raw_string_contains_quotes.h1**

```h1
let s = r"he said: "yo"";
```

Expect (with simple `r"..."` raw strings):

* This will terminate early and then error (because `"` ends raw string).
  Recommendation:
* If you want Rust-like robustness, implement `r#"..."#` forms.
  If you implement `r#"..."#`, then the same test becomes:

```h1
let s = r#"he said: "yo""#;
```

Expect:

* KW(let) IDENT OP(=) RAW_STRING SEMI EOF

---

### 3.3 Intent directives

**T20_intent_parallel_for.h1**

```h1
@intent.parallel
for x in items { emit(x); }
```

Expect tokens:

* AT IDENT(intent) DOT IDENT(parallel) NL
* KW(for) IDENT(x) KW(in) IDENT(items) LBRACE IDENT(emit) LPAREN IDENT(x) RPAREN SEMI RBRACE
* EOF

**T21_intent_serial_loop_semicolon.h1**

```h1
@intent.serial;
loop { work(); }
```

Expect:

* AT IDENT(intent) DOT IDENT(serial) SEMI
* KW(loop) LBRACE IDENT(work) LPAREN RPAREN SEMI RBRACE
* EOF

**T22_intent_dangling.h1**

```h1
@intent.parallel
```

Expect:

* tokens then (parser) error: dangling intent directive
  (lexer should succeed)

---

### 3.4 Forbidden keywords

**T30_forbidden_async.h1**

```h1
async fn f() { 1 }
```

Expect:

* LEXER ERROR at `async`

**T31_forbidden_await.h1**

```h1
let x = await y;
```

Expect:

* LEXER ERROR at `await`

---

### 3.5 Operators / punctuation / match arms

**T40_match_tokens.h1**

```h1
match v {
  A(x) => f(x),
  _    => g(),
}
```

Expect:

* KW(match) IDENT(v) LBRACE
* IDENT(A) LPAREN IDENT(x) RPAREN FATARROW IDENT(f) LPAREN IDENT(x) RPAREN COMMA
* UNDERSCORE(or IDENT "_") FATARROW IDENT(g) LPAREN RPAREN COMMA
* RBRACE EOF

---

### 3.6 Generics for channel

**T50_channel_generic.h1**

```h1
let ch = channel<Int>();
let tx = get_sender(ch);
let rx = get_receiver(ch);
```

Expect:

* KW(let) IDENT(ch) OP(=) KW(channel) LT IDENT(Int) GT LPAREN RPAREN SEMI
* KW(let) IDENT(tx) OP(=) IDENT(get_sender) LPAREN IDENT(ch) RPAREN SEMI
* KW(let) IDENT(rx) OP(=) IDENT(get_receiver) LPAREN IDENT(ch) RPAREN SEMI
* EOF