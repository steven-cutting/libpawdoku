---
title: "Decision 0008: A pure no_std core"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_no_std_core]
requires: []
---

# Decision 0008: A pure no_std core

This record will explain the constraints that keep bindings possible: no clock, threads
or filesystem, the randomness trait, thread-safe public types, non-exhaustive public
types, no panics, the feature policy and the two WebAssembly targets. It is a new
record, written by lane T09.
