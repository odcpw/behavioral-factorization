---
name: behavioral-factorization
description: Validate or falsify explicit finite behavioral models and finite candidate or route records with kerf. Use when a task supplies a deterministic LTS/context monitor, asks for kerf, needs an exact quotient or shortest distinguishing trace, or compares a finite supplied candidate set or static routes. Use optimize-program-seam-topology for repository discovery, migration planning, or architecture work involving folding or biology metaphors; return here only after it yields an explicit finite model.
---

# Behavioral Factorization

## Governing principle

Treat architecture as an observation-preserving factorization of an open
behavioral system. A proposed seam is lawful only when the whole system factors
through its boundary while preserving every declared observation in every
declared context.

For histories on one side of a boundary, the exact deterministic ideal is:

```text
h ~ h'  iff  the residual admissible continuation domains after h and h'
             are equal, and every word in that domain produces the same
             protected observations from h and h'

Z* = histories / ~
```

`Z*` is the minimal sufficient boundary state only in the model for which the
contexts, observations, and equivalence relation were declared. It may be
infinite or uncomputable for a real program. Never launder a bounded model into
a universal source-code proof.

After feasibility, optimize authority, placement, scheduling, routing, build,
coordination, and migration as separate dimensioned costs. The target is a
robust Pareto frontier with lower-bound evidence, not a scalar ground state.

## Choose one task mode

Do not run the whole method by default.

- **Finite behavior check:** use workflow steps 1–4 and 8. Read
  [schemas.md](references/schemas.md) and the quotient section of
  [causal-boundaries.md](references/causal-boundaries.md).
- **Finite candidate or route check:** read [schemas.md](references/schemas.md)
  plus the applicable section of
  [search-and-migration.md](references/search-and-migration.md) or
  [evidence-and-flow.md](references/evidence-and-flow.md). For candidates use
  steps 1, 2, 5, 6, and 8; for routes use steps 1, 2, 5, 7, and 8.
- **Standalone natural analogy test after repository evidence extraction:** use
  steps 1, 2, and 8. Read
  [natural-systems.md](references/natural-systems.md) and translate the analogy
  into a variable, model, or falsifier before using it.
- **Real repository investigation:** stop and use
  `optimize-program-seam-topology`; return here only after an explicit finite
  model or finite candidate record exists.

## Reference map

- Read [causal-boundaries.md](references/causal-boundaries.md) for contextual
  equivalence, minimal realizations, lumpability, memory kernels, and exact
  versus bounded claims.
- Read [evidence-and-flow.md](references/evidence-and-flow.md) for typed seams,
  evidence extraction, conservation, communication and hardware lower bounds,
  queues, placement, and literal resource flow.
- Read [rust-systems.md](references/rust-systems.md) for ownership, borrowing,
  effects, async cancellation, memory layout, scheduling, ABI, build topology,
  and agent work surfaces.
- Read [search-and-migration.md](references/search-and-migration.md) for
  feasibility, Pareto search, MDL, candidate enumeration, migration paths, and
  optimality language.
- Read [natural-systems.md](references/natural-systems.md) when protein,
  self-assembly, soil, root, fungal, thermodynamic, quantum, criticality, or
  renormalization ideas appear.
- Read [schemas.md](references/schemas.md) before using or extending `kerf`.
- Consult [sources.md](references/sources.md) only when a claim needs its primary
  source or when extending the method.

Load only the references needed for the current decision.

## Workflow

### 1. Name the decision and its proof scope

State:

```text
decision:
named consumer and decision deadline:
retirement condition for this analysis:
system and revision:
admissible contexts/interventions:
protected observations:
equivalence/refinement strength:
workloads and machines:
candidate language:
evidence threshold:
```

Protected observations may include values, trace order, RNG draws, allocation
identity, arithmetic width, failure class, reset/persistence, ABI bytes,
latency bounds, or security properties. If it is not named, it is not protected
by the analysis.

### 2. Model the open behavior before drawing modules

Record state carriers, transitions, effects, identities, ordering, lifetimes,
boundary ports, laws, observations, and provenance. Treat files and crates as
physical/build evidence, not automatic semantic units.

Classify every boundary coordinate independently:

```text
semantic | spatial | temporal | ownership | protocol | proof | build | organizational
```

A textual API can realize several of these at once. Do not sum them before
their units and constraints are understood.

### 3. Quotient only proved distinctions

Use exact partition refinement, bisimulation, translation validation, or a
declared bounded conformance relation. For a proposed merge, actively search
for the shortest distinguishing continuation. Unknown means unresolved, never
equivalent.

For finite deterministic models use `kerf`:

```bash
kerf quotient model.lts.json
kerf distinguish pair.compare.json
kerf factor proposed.factor.json
kerf recompose proposed.recompose.json
```

These commands certify only the supplied finite models. A resource limit
returns unknown. `minimum_product_monitor_state_bits` counts the joint
machine×context realization; it is not a machine- or seam-owned state lower
bound when the context monitor is external side information.

### 4. Factor through explicit ports

A component boundary must expose all and only the state, actions, order,
authority, failure, and conservation obligations needed by its admissible
contexts. Ask:

- Does removed state reappear as callbacks, caches, invalidation, retries,
  adapter state, or hidden history?
- Can local contracts glue into one global state?
- Is the boundary state Markov-sufficient at the declared horizon?
- Does the objective itself factor, or do shared allocators, locks, caches,
  quotas, or build roots create cross-terms?

Mori-Zwanzig supplies the warning: eliminating relevant state generally
reappears as memory and unresolved forcing. A small stateless API with large
coordination machinery is evidence against the seam.

When a finite proposed map `q` is already explicit, `kerf factor` checks that
observations and every jointly enabled action's emission and successor
boundary state factor through `q`. The context monitor remains external side
information and may select different enabled-action domains. Passing proves
only that `(q, id_context)` factors that supplied reachable finite product; it
does not infer or minimize `q`.

`kerf recompose` checks the inverse supplied finite direction. Each factor
representation carries the context language under which its coordinates were
certified. The checker synchronizes those monitors with the proposed context,
uses only constrained in-language transitions, applies explicit observation
and emission readouts, and compares the proved-reachable product with a total
reference. Leaving a certified language or reaching an unconstrained
coordinate is `unproved`, never an implicit completion. A definite mismatch
remains an inequivalence even if another word is unproved; the result reports
both and states whether witness minimality is completion-robust or only within
the proved region. This checks supplied representations, not their producer
provenance; locally checkable proof bundles are a separate layer.

### 5. Separate irreducible flow from demonstrated waste

For each cost coordinate maintain:

```text
proved floor L <= unknown optimum <= best feasible U <= current measured C
```

- `C - U` is demonstrated removable cost.
- `L` is certified irreducible cost under the model.
- `U - L` is unresolved opportunity.
- `C - L` is not automatically waste.

Use communication complexity, contextual state cardinality, I/O bounds,
capacity cuts, queue/service bounds, or physical measurement only where their
assumptions match. Keep bits, bytes, messages, cache lines, seconds, cycles,
joules, build seconds, and developer time separate.

### 6. Optimize feasible candidates without mystical weights

Reject semantic, contract, policy, capacity, ordering, and liveness violations
before scoring. Preserve the raw vector and Pareto frontier. Use a scalar only
when a real decision-maker supplies explicit exchange rates and sensitivity is
reported.

For a supplied finite candidate set:

```bash
kerf validate-candidates architecture.candidates.json
kerf pareto architecture.candidates.json
```

“Exact frontier” means exact among that list, not globally optimal software.

### 7. Co-design logical seams and physical realization

After the semantic quotient, choose authority, schedule, placement, routing,
buffering, and materialization together. A good module graph can still cause
bad cache, NUMA, serialization, or queue behavior.

For declared unsplittable routes:

```bash
kerf score-routes memory.transport.json
```

This is exact route arithmetic, not a queue, cache, or energy simulation.

### 8. Falsify prospectively

Before recommending a seam, state:

```text
verdict: proved_finite | supported_bounded | underdetermined | falsified
witness or certificate:
remaining blind spot:
next discriminating test:
```

Use nearby positive and negative controls. Perturb rare paths, workloads,
evidence layers, placement, and migration order. Prefer a truthful
underdetermined result to architectural astrology.

## Non-negotiable guardrails

- Hard semantics and missing proof are not finite costs.
- A graph cut, cluster, high traffic edge, or co-change pattern is not a seam
  proof.
- Causal language requires interventions or explicit causal assumptions;
  contextual trace equivalence alone is observational.
- Rust ownership proves safety properties, not semantic authority, locality,
  determinism, deadlock freedom, or performance.
- Replication is lawful only when its convergence, derivation, consistency, and
  failure protocol is part of the semantics.
- “Energy” means joules, a valid storage/Hamiltonian/Dirichlet function, or an
  explicitly dimensioned objective. Otherwise say cost, action, potential, or
  regret.
- Equilibrium, free energy, entropy production, absolute zero, BEC, criticality,
  and Markov blankets do not become software laws by analogy.
- A natural-system analogy is a hypothesis generator only. It earns a place
  after producing a variable, assumption, algorithm, or falsifiable prediction.
- No universal architecture optimum exists for arbitrary programs. Use
  “globally optimal” only for a completely specified bounded model and report
  the candidate set or solver gap.
