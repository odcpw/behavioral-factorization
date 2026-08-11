# Behavioral Factorization repository instructions

This file adds only repository-specific rules to
`/home/oliver/.codex/AGENTS.md`.

## Purpose and authority

- `/home/oliver/Projects/odcpw/behavioral-factorization` is the canonical
  local checkout. Git identity is `odcpw <odcpw@users.noreply.github.com>` and
  the branch is `main`. The private canonical remote is
  `git@github-odcpw:odcpw/behavioral-factorization.git`.
- This is a private research tool and Codex skill. There is no public release
  authority or release license; do not make it public or publish a release
  without Oliver's instruction.
- The repository is independently worded from public literature and contains
  no remc2 source, owner data, private traces, or MagicCarpet implementation
  details.

## Product boundary

- `SKILL.md` is the concise routing and operating manual. Detailed theory lives
  under `references/`; do not turn the skill entry point into a literature dump.
- `kerf` proves properties only of supplied finite deterministic models and
  represented-feasible supplied candidate sets. It never claims equivalence of an original
  Rust program, causal identification, or a globally optimal real architecture.
- Hard semantic, contract, and policy constraints are feasibility conditions.
  They are never finite penalties. Unknown evidence is never equivalent to
  positive proof.
- Cost coordinates retain their units and use Pareto order. Do not introduce
  hidden weights, floating-point tolerances, or an "architecture energy."
- Exact outputs are deterministic, versioned, and bound to the input hash. LTS
  exploration is explicitly bounded; a resource limit returns `unknown`, never
  success. No output is implementation-revision-bound until the envelope gains
  a checked kernel identity.

## Architecture

- `src/lts.rs` owns finite machines, context monitors, validation, products,
  equivalence witnesses, and observational quotienting.
- `src/factor.rs` checks supplied context-relative factor maps and emits their
  induced partial boundary representations or shortest conflict witnesses.
- `src/candidates.rs` owns explicit architecture assignments, executable hard
  constraints, exact metric validation, and Pareto dominance.
- `src/transport.rs` owns arbitrary-precision exact rational arithmetic for
  declared static routes.
- `src/report.rs` owns typed domain errors and input hashing.
- `src/main.rs` owns argument routing, file I/O, and versioned result/error
  envelopes with execution context.
- `examples/` are discriminating microcases; `references/` separates theorem,
  engineering inference, conjecture, and broken metaphor.

## Toolchain and verification

- Rust stable is the only runtime dependency. Keep the dependency set small and
  commit `Cargo.lock` for reproducibility.
- Focused checks:

  ```bash
  cargo fmt --all -- --check
  cargo test --all-targets --locked
  cargo clippy --all-targets --all-features --locked -- -D warnings
  jsm validate . --json
  ```

- For each semantic algorithm, test a nearby positive case, a shortest negative
  witness, invalid input, unreachable state handling, deterministic ordering,
  and a configured resource limit.
- New claims require either an executable finite certificate or a primary-source
  citation plus an explicit evidence label. Natural-system analogies never count
  as software evidence.
