What tools apply where

Not every project needs every technique. The applicability depends on what the code does:

| Technique                        | Best for                                                                      | Not useful for                          |
| -------------------------------- | ----------------------------------------------------------------------------- | --------------------------------------- |
| Kani (bounded proofs)            | Encoding/decoding, arithmetic, unsafe, data structures                        | Async/network code, anything with I/O   |
| TLA+ (model checking)            | Concurrency protocols, channels, lock hierarchies, state machines             | Pure functions, single-threaded code    |
| Proptest (property-based)        | Serialization roundtrips, invariants over random inputs, differential testing | UI code, anything hard to assert on     |
| Fuzzing (libfuzzer)              | Parsers, decoders, anything that takes untrusted bytes                        | Stateful protocols (needs harness work) |
| DST (simulation testing)         | Distributed systems, anything with network/disk/time non-determinism          | Libraries, pure computation             |
| Stateright (Rust model checking) | State machines, CRDT properties, protocol verification                        | Same as Kani targets                    |

The Datadog guide uses all of these because redis-rust is a distributed stateful system. Rolly is a library with concurrency — so DST is less relevant (no
simulated network/disk), but everything else applies.

How agents should decide what to use

The input shouldn't be the PRD alone. It should be a verification target analysis — the agent reads the code and classifies what it finds:

1. Pure functions with complex logic → Kani + proptest
2. Parsers / encoders taking external input → fuzzing + proptest
3. Concurrent state machines → TLA+ + Stateright
4. Unsafe blocks → Kani (mandatory)
5. Distributed protocols → DST + TLA+ + Maelstrom
6. CRDT / algebraic structures → Kani (commutativity, idempotence) + Stateright

The PRD tells you what the system does and what the failure cost is. The code tells you which techniques apply where. The agent needs both.

Prompt structure suggestion

I'd suggest three prompts, not one monolith:

⏺ User answered Claude's questions:
⎿  · How do you want to structure the verification prompts? → 4+ prompts

⏺ Good choice. Maximum specialization means each prompt can encode deep domain knowledge for its technique. Here's my proposed breakdown:

Prompt ensemble

Phase 1 — Analysis (1 prompt)

1. verify-plan — Reads the codebase + PRD, classifies every module/function into verification targets, outputs a JSON matrix mapping targets → techniques.
   This is the orchestrator's input. It answers: "what needs verifying and how?"

Phase 2 — Implementation (5 technique-specific prompts)

2. verify-kani — Writes #[kani::proof] harnesses. Knows about unwind bounds, symbolic inputs, kani::any() + kani::assume() patterns, what CBMC can and can't
   handle (e.g., avoid large loops, String ops are expensive). Targets: encoding, arithmetic, unsafe, data structure invariants.
3. verify-tla — Writes TLA+ / PlusCal specs with .cfg files. Knows about safety vs liveness properties, fairness (WF/SF), state space explosion, small model
   parameters. Targets: concurrency protocols, channels, lock hierarchies, state machines.
4. verify-proptest — Writes proptest strategies and property tests. Knows about shrinking, strategy composition, metamorphic/roundtrip/differential
   patterns, how to write good prop_assert! invariants. Targets: serialization, pure functions, any input→output mapping with checkable properties.
5. verify-fuzz — Writes libfuzzer targets with arbitrary or raw byte input. Knows about corpus management, crash triage, coverage-guided mutation,
   structured fuzzing vs raw bytes. Targets: parsers, decoders, anything taking untrusted input.
6. verify-dst — Writes deterministic simulation harnesses. Knows about shadow-state oracles, buggify-style fault injection, seeded RNGs,
   SimulatedRng/SimulatedClock patterns, Zipfian workloads, the extract-then-assert borrow checker pattern. Targets: distributed/stateful systems with
   network/disk/time non-determinism.

Phase 3 — Execution (1 prompt)

7. verify-run — Executes all verification artifacts, collects results, produces a unified pass/fail report with the verification pyramid table. Knows how to
   run each tool (cargo kani --harness, tla-checker, cargo test, cargo fuzz run), interpret output, and classify failures (real bug vs. tool limitation like
   unwind bounds).

What each prompt needs as input

| Prompt          | Primary input                | Secondary input                        |
| --------------- | ---------------------------- | -------------------------------------- |
| verify-plan     | Codebase scan                | PRD (failure cost informs rigor level) |
| verify-kani     | Target list from verify-plan | Source code of target functions        |
| verify-tla      | Target list from verify-plan | Architecture docs, sequence diagrams   |
| verify-proptest | Target list from verify-plan | Source code, existing test patterns    |
| verify-fuzz     | Target list from verify-plan | Source code, public API surface        |
| verify-dst      | Target list from verify-plan | Architecture docs, I/O boundaries      |
| verify-run      | All artifacts from phase 2   | Previous run results (for regression)  |

Key design decisions for the prompts

Each prompt should encode:

- When to use this technique (and when NOT to)
- Tool-specific pitfalls (e.g., Kani: CBMC timeouts with String ops; TLA+: tla-checker lacks WF support)
- The output format (harness code location, naming conventions, cfg gates)
- How to size the problem (small bounds first, escalate if needed)
- What "passing" means vs "tool limitation" vs "real bug"

The verify-plan prompt is the most critical one — it prevents wasted effort by matching techniques to targets. A bad plan means the Kani prompt tries to
verify async code (waste) or the DST prompt targets a stateless library (waste).

Want me to start writing these prompts? I'd suggest starting with verify-plan since it's the orchestrator, then the two most relevant to rolly: verify-kani
and verify-proptest.
