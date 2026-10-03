---
title: "Documentation map"
kind: "project"
audience: [user, contributor, maintainer, operator, agent]
canonical_for: [documentation_navigation]
requires: []
---

# Documentation map

Every page below is registered in `manifest.yml`, owns at least one topic, and is
reachable from here. That is the whole of the arrangement; the rules behind it are in
the documentation contract, listed under Look up.

Behaviour is specified separately, in Allium, under `docs/specs/`, with the rules of
Sudoku themselves in [`sudoku.allium`](specs/sudoku.allium), a puzzle as it is being
played in [`board.allium`](specs/board.allium), and what decides whether givens are
well-posed in [`solver.allium`](specs/solver.allium). How a person solves one
is [`technique.allium`](specs/technique.allium)'s, with four models of a player built on
it: [`reach.allium`](specs/reach.allium), [`effort.allium`](specs/effort.allium),
[`lapse.allium`](specs/lapse.allium) and
[`human-solving.allium`](specs/human-solving.allium), the one with chance in it: bounded
cognition, fallible attempts and repeated assessment. How a setter that is a program
finds givens is [`generation.allium`](specs/generation.allium)'s, which states the basic
generator in full and the designed one as a skeleton of scope, config and open
questions. Those files are not part of this
handbook; they are its subject. Start at Specifications, under Understand, to see how the
two relate. The game's root module, `pawdoku.allium`, stays in the game and restates what
it needs.

## Start here

- [Purpose and scope](project/purpose-and-scope.md) — what the engine is for, and what it is not.
- [Repository map](project/repository-map.md) — where everything lives.
- [Terminology](project/terminology.md) — the words this repository uses precisely.
- [Make your first change](tutorials/first-change.md) — clone to green gate, once through every layer.

## How to

- [Develop locally](how-to/develop-locally.md)
- [Test and debug](how-to/test-and-debug.md)
- [Work with the specifications](how-to/work-with-the-specs.md)
- [Maintain dependencies](how-to/maintain-dependencies.md)

## Understand

- [Architecture](explanation/architecture.md) — how a portable engine with one effect is put together.
- [Layering and dependency direction](explanation/layering.md) — which module may import which.
- [Specifications](explanation/specifications.md) — why behaviour is written down before it is built.
- [Security model](explanation/security-model.md) — what a library of pure computation does and does not defend.
- [Quality philosophy](explanation/quality-philosophy.md) — why each gate exists.

## Look up

- [Commands](reference/commands.md)
- [Configuration](reference/configuration.md)
- [Testing](reference/testing.md)
- [Quality gates](reference/quality-gates.md)
- [Documentation contract](reference/documentation-contract.md)
- [Agent contract](reference/agent-contract.md)
- [API reference](reference/api.md)

## Run it

- [Maintenance](operations/maintenance.md)
- [Troubleshooting](operations/troubleshooting.md)

## Decisions

- [Architecture decisions](decisions/README.md) — the record of what was chosen and why.

## This library

Pages about the engine's subject rather than its machinery are listed here, after the
handbook's own, and came with the specifications they are cited by.

- [Strategies for solving Sudoku](explanation/solving-sudoku.md) — how people, mathematics and
  programs solve one, and the background to [`solver.allium`](specs/solver.allium).
- [Modelling a human Sudoku solver](explanation/human-solving.md) — cognitive limits,
  experience, mistakes, parameter presets and profile-relative assessment.
- [Puzzle design objectives](explanation/puzzle-design.md) — what a designed puzzle is
  asked to be, which of those asks the specifications already meet, and which are open;
  the ground for [`generation.allium`](specs/generation.allium).
