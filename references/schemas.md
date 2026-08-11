# `kerf` v0.1 schemas and evidence contract

## Output contract

All successful analysis commands write one JSON object to stdout:

```json
{
  "schema_version": "kerf.result.v1",
  "tool_version": "0.1.0",
  "operation": "quotient",
  "input_sha256": "...",
  "status": "ok",
  "limits": null,
  "result": {}
}
```

Errors are one typed `kerf.error.v1` object on stderr. Errors discovered after
input hashing retain the operation and input hash; LTS analysis errors also
retain the applied limits. Reaching a configured analysis bound has
`status: "unknown"`.
Exit codes:

- `0`: valid analysis completed, including an infeasible candidate report;
- `2`: invalid input, unsupported schema, or I/O failure;
- `3`: resource bound reached, result unknown;
- `4`: internal invariant/serialization failure.

IDs and actions are canonically sorted where order affects output. All semantic
observations and emissions are opaque UTF-8 strings. Rates and costs use exact
unsigned integer or rational strings; no binary floating point or tolerance is
part of v0.1.

## `kerf.lts.v1`

Used by `validate-lts` and `quotient`.

```json
{
  "schema_version": "kerf.lts.v1",
  "machine": {
    "initial": "s0",
    "actions": ["tick"],
    "states": [
      {
        "id": "s0",
        "observation": "ready",
        "transitions": {
          "tick": {"to": "s0", "emission": "done"}
        }
      }
    ]
  },
  "context": {
    "initial": "c0",
    "states": [
      {"id": "c0", "transitions": {"tick": "c0"}}
    ]
  },
  "limits": {
    "max_product_states": 1000000,
    "max_pairs": 1000000,
    "max_witness_length": 100000
  }
}
```

Rules:

- State and action IDs are unique and non-empty.
- The machine is total and deterministic over every declared state/action pair.
- Failure, terminal, disabled-action, and deadlock behavior must be explicit;
  the tool never invents a sink.
- Context is optional, finite, and deterministic. A missing context transition
  prohibits the action. Without a context, every action is permitted forever.
- The context is an action-language monitor. It does not react to outputs.
- Quotienting uses only reachable `(machine_state, context_state)` products and
  reports unreachable machine states separately.
- Residual admissible-action domains are behavior: product states with different
  future action languages remain distinct.
- The stable signature contains current observation and each enabled action's
  emission plus successor class.

Exact claim: observational quotient of this finite deterministic product.

Nonclaims: source-program equivalence; infinite-trace acceptance; liveness;
fairness; timing; or any effect not encoded in the model.

## `kerf.compare.v1`

Used by `distinguish`.

```json
{
  "schema_version": "kerf.compare.v1",
  "left": {"initial": "...", "actions": [], "states": []},
  "right": {"initial": "...", "actions": [], "states": []},
  "context": null,
  "limits": {}
}
```

The machines must declare the same action set. Breadth-first search explores
`(left_state, right_state, context_state)`. A current observation mismatch has
an empty action word. Otherwise each enabled action exposes its transition
emission and successor observation at the same word length. Canonical
breadth-first action-word order yields a shortest word with deterministic
lexicographic tie-breaking.

If a configured pair or witness-depth bound prevents a decision, exit `3`
reports unknown. It never reports equivalent after truncation.

## `kerf.factor.v1`

Used by `factor` to check a supplied context-relative boundary map.

```json
{
  "schema_version": "kerf.factor.v1",
  "machine": {"initial": "...", "actions": [], "states": []},
  "context": {"initial": "...", "states": []},
  "factor_map": [
    {
      "machine_state": "m0",
      "context_state": "c0",
      "boundary_state": "z0"
    }
  ],
  "limits": {"max_product_states": 1000000}
}
```

The map domain must be exactly the reachable `(machine_state, context_state)`
product. Duplicate, missing, unreachable, unknown, and empty entries are
invalid. Without a context, `context_state` is `null`. The only operation-
specific bound is positive `max_product_states`; reaching it before exploration
closes returns unknown.

For each fibre of the supplied map `q`, the checker requires:

- one protected observation;
- for an action enabled at two fibre members, one emitted value; and
- for that jointly enabled action, one successor `q` state.

Different fibre members may have different enabled-action domains because the
context monitor is retained externally. A lawful result emits a canonical
`kerf.factor-representation.v1`: the initial boundary state, observation per
state, and a constrained transition for each action enabled anywhere in the
fibre. An action enabled nowhere in the reachable fibre is explicitly
`unconstrained`; this means the supplied context provides no reachable evidence
for that coordinate, not that an implementation may ignore the context.

An unlawful map is a completed analysis (`status: "ok"`, `lawful: false`), not
invalid input. Its witness names the broken law and a deterministic pair of
shortest access words ordered left then right, plus the offending action for emission or successor
conflicts. Witnesses minimize maximum exposure length, then each access word's
length and lexical order, followed by action, law kind, and boundary ID.

`context_retained: true` is an operation-mode marker. It also applies when the
input omits `context`, because that case uses the synthesized all-actions unit
context.

Exact claim: the reachable finite product behavior does or does not factor
through `(q, id_context)` using the emitted boundary representation and the
supplied output-blind context monitor, relative to the declared protected
observations and emissions.

Nonclaims: minimum boundary cardinality; validity under larger contexts;
source-program equivalence; liveness; fairness; timing; causality; or any
unmodeled effect.

## `kerf.candidates.v1`

Used by `validate-candidates` and `pareto`.

```json
{
  "schema_version": "kerf.candidates.v1",
  "elements": ["world", "view"],
  "modules": ["core", "ui"],
  "constraints": {
    "co_locate": [["a", "b"]],
    "separate": [["world", "view"]],
    "pinned": {"world": "core"},
    "authority_groups": [
      {"id": "world-state", "members": ["world"]}
    ]
  },
  "metric_definitions": [
    {"id": "bits", "unit": "bits_per_step", "direction": "minimize"},
    {"id": "headroom", "unit": "bytes_per_second", "direction": "maximize"}
  ],
  "candidates": [
    {
      "id": "candidate-a",
      "assignment": {"world": "core", "view": "ui"},
      "authority": {"world-state": "world"},
      "metrics": {"bits": "128", "headroom": "4096"}
    }
  ]
}
```

Native feasibility checks:

- every element is assigned exactly once to a declared module;
- pins, co-location, and separation;
- one selected member for every authority group; and
- complete exact unsigned-integer metrics with declared units.

The schema deliberately has no `semantic_equivalence: true` escape hatch.
Semantic evidence must come from a separate proof/check and be connected by the
consumer's evidence ledger.

Pareto dominance uses every declared metric. Missing metrics are invalid input,
not zero. Equal vectors remain nondominated. The supplied-set certificate means
only that every supplied candidate was validated and every represented-feasible
candidate was pairwise compared. It does not claim candidate-space
exhaustiveness.

## `kerf.transport.v1`

Used by `score-routes`.

```json
{
  "schema_version": "kerf.transport.v1",
  "workload_id": "step-v1",
  "rate_unit": "bytes_per_second",
  "weighted_hop_traffic_unit": "byte_hops_per_second",
  "resources": ["cpu", "dram"],
  "links": [
    {
      "id": "cpu-dram",
      "from": "cpu",
      "to": "dram",
      "capacity": {"num": "1000000", "den": "1"},
      "latency_ns": "80"
    }
  ],
  "commodities": [
    {
      "id": "read",
      "source": "cpu",
      "sink": "dram",
      "demand": {"num": "64000", "den": "1"},
      "route": ["cpu-dram"]
    }
  ]
}
```

Rules:

- At least one link is required. Links are directed; full duplex requires two
  independently accounted edges.
- Routes are explicit, contiguous, unsplittable, simple directed paths: they
  may not repeat links or revisit resources.
- Equal source and sink require an empty route and add no routed load.
- Multicast must be expanded into explicit sink commodities before input.
- Capacity is positive; demand is nonnegative; rational arithmetic uses
  arbitrary-precision unsigned integers.

Computed values:

```text
load(link) = sum demand of routed commodities
utilization = load / capacity
weighted hop traffic = sum demand * route hops
demand-weighted link latency = sum demand * sum link latency
```

The latency omits queueing, arbitration, overlap, pipelining, and backpressure.
Average route load does not prove queue stability or burst tolerance.
`rate_unit` and `weighted_hop_traffic_unit` are opaque user-declared labels;
v0.1 does not check dimensional consistency. Lower-bound certificates are not
accepted because v0.1 has no mechanically checked lower-bound proof language.

## v0.1 exclusions

The executable does not ship automatic source extraction, causal inference,
probabilistic state reconstruction, hypergraph clustering, architecture search,
routing optimization, queue simulation, cache/NUMA prediction, physical energy,
Rent fitting, passivity checking, or migration validation. These require
separate evidence and certificate checkers rather than unverified flags.
