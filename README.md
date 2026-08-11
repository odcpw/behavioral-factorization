# Behavioral Factorization

Behavioral Factorization is a research method and Rust checker for finding
lawful system boundaries. Its root claim is deliberately narrow:

> Factor an open system through the smallest boundary that preserves its
> declared behavior, then optimize ownership, transport, placement, and
> migration only inside that feasible set.

The `kerf` binary provides exact checks for supplied finite deterministic
models. LTS exploration has explicit state, pair, and witness-depth limits:

- validate a total transition system and finite context monitor;
- compute the reachable observational quotient;
- compare two machines and return a shortest distinguishing action word;
- check whether a supplied context-relative map is a lawful behavioral factor;
- validate explicit assignment, separation, pin, and authority constraints;
- compute the exact Pareto frontier of the represented-feasible supplied set; and
- account checked rational load on declared static routes.

It does not infer semantics from Rust source or claim a globally optimal real
architecture. The surrounding skill explains how to build evidence, derive
applicable lower bounds, and keep the theorem/inference/metaphor boundary
honest.

The name is literal. The irreducible operation found by the research swarm was
not energy minimization or graph clustering, but factorization of an open
behavior through an interface that preserves declared observations. The binary
is called `kerf`: the material a lawful cut must consume. Its measured width is
contextual state, protocol, and routed cost—not a universal scalar.

## Quick start

```bash
cargo test --all-targets --locked
cargo run --quiet -- quotient examples/toggle.lts.json | jq
cargo run --quiet -- distinguish examples/rng-order.compare.json | jq
cargo run --quiet -- factor examples/context-side-information.factor.json | jq
cargo run --quiet -- pareto examples/architecture.candidates.json | jq
cargo run --quiet -- score-routes examples/memory.transport.json | jq
```

All analysis output is versioned JSON on stdout. Errors are typed JSON on
stderr. Exact observations and emissions are opaque strings, avoiding hidden
JSON numeric or canonicalization semantics.

## Why there is no simulator dependency

The v0.1 proof kernel is finite and discrete. FrankenSim was evaluated as a
possible foundation for continuum and physical-system experiments, but adopting
it here would add a large simulation surface without strengthening finite trace
equivalence, exact Pareto dominance, or static route arithmetic. A future
continuum benchmark may integrate it when a named model and falsification test
require it.

## Status

This is a private `odcpw` repository with no public release authority or
release license. The repository contains no MagicCarpet source, remc2
material, owner data, or private traces.
