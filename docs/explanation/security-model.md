---
title: "Security model"
kind: "explanation"
audience: [user, contributor, maintainer, operator, agent]
canonical_for: [security_model]
requires: []
---

# Security model

This page will describe what a library of pure computation defends and what it leaves to
its callers: the supply chain, the ban on unsafe code, the licence and source checks,
and the absence of any network, clock or filesystem access at run time. It is a shorter
adaptation of the Pawdoku page of the same path, written by lane T08.
