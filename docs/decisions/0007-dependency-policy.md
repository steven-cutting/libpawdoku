---
title: "Decision 0007: Dependency policy for a library"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_dependency_policy]
requires: []
---

# Decision 0007: Dependency policy for a library

This record will explain why manifests here write caret ranges while the committed
lockfile is the real pin, why every gate runs locked, and what the dependency policy
checks enforce. It departs from the games' exact-pin rule and is a new record, written
by lane T09.
