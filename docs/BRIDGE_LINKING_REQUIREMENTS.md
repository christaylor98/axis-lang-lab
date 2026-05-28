### 1. Scope (explicit)

* Applies **only** to the Rust ejection bridge
* Emitters are assumed correct
* Registry is semantic-only
* CoreIR is semantic-only

### 2. Non-goals (important)

* Not a long-term package system
* Not certified linking
* Not hermetic builds
* Not reproducible builds
* Not runtime discovery

### 3. Link table

* Bridge-owned
* Cache, not truth
* Safe to delete
* Safe to edit
* Auto-updated only on successful builds

### 4. Library scanning

* Manual trigger only
* Recursive directory scan
* Opportunistic
* Non-destructive merge
* Never wipes unknown entries

### 5. Authority

* Rust linker is the final authority
* Bridge does not attempt to be clever

### 6. Failure model

* Missing / wrong links fail loudly
* Recovery via scan or manual edit
* No hidden fallback paths

