@0x8a4f2d9c3e7b1a02;

# Axis Core IR 0.2
# Canonical binary schema
# Semantic authority is defined by the Core IR 0.2 specification

# using Text = import "capnp/text.capnp".Text;

struct CoreBundle {
  version       @0 :Text;
  coreTerm      @1 :CoreTerm;

  # Optional, non-authoritative fields
  entrypointName @2 :Text;
  entrypointId   @3 :UInt64;

  annotations   @4 :List(Annotation);
}

# -----------------------------
# Core Terms (closed node set)
# -----------------------------

struct CoreTerm {
  nodeId @0 :UInt64;
  span   @1 :Span;

  union {
    cIntLit   @2 :CIntLit;
    cBoolLit  @3 :CBoolLit;
    cUnitLit  @4 :CUnitLit;
    cLam      @5 :CLam;
    cLet      @6 :CLet;
    cIf       @7 :CIf;
    cVar      @8 :CVar;
    cApp      @9 :CApp;
    cCall     @10 :CCall;
  }
}

struct CIntLit {
  value @0 :Int64;
}

struct CBoolLit {
  value @0 :Bool;
}

struct CUnitLit {
}

struct CLam {
  param @0 :Text;
  body  @1 :CoreTerm;
}

struct CLet {
  name  @0 :Text;
  value @1 :CoreTerm;
  body  @2 :CoreTerm;
}

struct CIf {
  cond @0 :CoreTerm;
  then @1 :CoreTerm;
  else @2 :CoreTerm;
}

struct CVar {
  name @0 :Text;
}

struct CApp {
  fn   @0 :CoreTerm;
  arg  @1 :CoreTerm;
}

struct CCall {
  target @0 :UInt64;
  args   @1 :List(CoreTerm);
}

# -----------------------------
# Annotations (non-authoritative)
# -----------------------------

struct Annotation {
  id      @0 :UInt64;
  target  @1 :UInt64;   # nodeId this annotation attaches to
  payload @2 :Text;     # opaque to Core IR
}

# -----------------------------
# Spans (diagnostic only)
# -----------------------------

struct Span {
  file  @0 :Text;
  start @1 :UInt32;
  end   @2 :UInt32;
}
