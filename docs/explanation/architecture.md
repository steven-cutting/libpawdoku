---
title: "Architecture"
kind: "explanation"
audience: [contributor, maintainer, operator, agent]
canonical_for: [system_architecture]
requires: []
---

# Architecture

This page will explain how the engine is put together: the crate layout, the randomness
boundary through which the only effect enters, the bindings crates that will wrap the
core later, and what is kept out of the core so that it stays portable. It is rewritten
from the Pawdoku page of the same path by lane T08.
