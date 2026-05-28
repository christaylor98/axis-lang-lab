
# Wave 4 Freeze Note — Configuration and Profile Selection

## Status

**FROZEN**

Wave 4 is complete.
No further changes are permitted without opening a new wave.

---

## What Wave 4 Introduced (Authoritative)

Wave 4 introduced **configuration-driven selection**, and nothing else.

Specifically:

* Selection of **active registries** via configuration
* Selection of **active profile** via configuration
* Enforcement of profile admission during lowering
* Deterministic registry ID assignment across multiple registries

All semantics remain exactly as defined in Waves 1–3.

---

## What Configuration CAN Do

Configuration may:

* Select which registry files are active
* Define the order registries are searched
* Select the active profile name
* Influence **lowering-time admission checks**

Configuration is read **once**, before compilation.

---

## What Configuration CANNOT Do

Configuration must NOT:

* Introduce new language constructs
* Change Core IR structure or encoding
* Affect runtime execution semantics
* Enable or disable effects dynamically
* Modify registry contents
* Override lowering rules
* Embed itself into Core IR

Configuration is **policy**, not **semantics**.

---

## Locked Invariants (Non-Negotiable)

The following are now **hard invariants**:

1. **Lowering is the sole semantic authority**
2. **All registry resolution happens during lowering**
3. **Core IR is configuration-free**
4. **Core IR contains numeric handles only**
5. **Registry format is unchanged**
6. **Profiles are enforced, not interpreted**
7. **Earlier wave goldens remain unchanged**

Any violation requires a new wave.

---

## Explicit Non-Features (Deferred)

The following are **explicitly deferred** to future waves:

* Runtime execution
* Effect scheduling
* Profile hierarchies
* Profile inference
* Configuration conditionals
* Dynamic registry loading
* Policy languages

These were not “forgotten”; they were intentionally excluded.

---

## Boundary with Wave 5

After Wave 4:

* Compilation authority is complete
* Policy selection is externalised
* Core IR is stable and executable-ready

**Wave 5 is the first wave permitted to execute Core IR.**

---

## Change Control

Any proposal that:

* touches Core IR
* changes lowering semantics
* adds new AST constructs
* alters registry meaning

**must start a new wave with a new plan.**

---

## End of Document
