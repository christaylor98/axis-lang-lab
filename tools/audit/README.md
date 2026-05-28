# Analysing Core IR — load-bearing dataflow checker

This is a worked example of the kind of **decidable structural analysis** that a
small, closed IR makes possible. It is a demonstration tool, not part of the
compiler.

## What it does

Given a Core IR bundle, `axis_audit.py`:

1. **Validates structure** — closed 9-node set, exactly one variant per term,
   unique node ids, and bare single-token call names (no dotted namespaces).
2. **Runs a load-bearing audit** — for each call you nominate as "generative"
   (e.g. a model-backed call), it traces — by taint propagation over the nine
   nodes — whether that call's output reaches:
   - a branch condition (`cIf.cond`), or
   - an argument to a call you nominate as a "consequential effect".

   If it does → **load-bearing** (the call drives a decision or an effect).
   If it only reaches benign sinks → **side-effect** (provably detachable).

The classification of which calls are "generative" and which are
"consequential" is an explicit input, not a heuristic — so the verdict is a
function of a stated policy plus a mechanical, terminating propagation. The
analysis is the contribution; the policy is yours to state and inspect.

This is decidable here precisely because the IR is closed and control flow is
explicit. The same question over an unrestricted language is undecidable in
general.

## Run it

```bash
python3 axis_audit.py     # structural + audit demo on built-in example bundles
python3 aml_screen.py     # a worked AML transaction-screening example, two variants
```

`aml_screen.py` shows the point most clearly: two screening workflows that read
identically in prose — both "use an ML risk score" — are mechanically
distinguished. In one, the score gates a hold (load-bearing); in the other it
only annotates an analyst alert (side-effect). Same description, different
dataflow, different verdict.

## Scope and honest limitations

- **Input form.** The checker operates on the Core IR **JSON-mirror**
  representation (a 1:1 JSON encoding of the schema). A converter from the
  bridge's native `.coreir` (Cap'n Proto) output to this JSON mirror is the
  documented next step; until then, bundles are supplied in JSON-mirror form
  (see the example fixtures inside the scripts).
- **First-order dataflow.** Taint is tracked through lets, vars, applications
  and calls. A value captured in a lambda and applied elsewhere
  (higher-order capture) is conservatively under-tracked. The agentic/process
  workflows this targets are first-order pipelines.
- **It audits the IR, not the runtime.** A verdict is a statement about the
  lowered structure. Its usefulness depends on the IR faithfully representing
  the original workflow.

These boundaries are stated rather than hidden; the analysis is sound within
them.
