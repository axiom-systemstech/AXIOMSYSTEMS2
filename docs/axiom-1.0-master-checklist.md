# AXIOM 1.0 — Master Completion Checklist

**Purpose:** operational source of truth for the work required to declare the AXIOM language itself complete as **AXIOM 1.0**.

**Status:** active  
**Last reviewed:** 2026-10-01  
**Development branch:** `local/axiom`

---

## How to use this document

This file is deliberately **not organized by roadmap phases**. It is a point-by-point completion checklist.

- `[x]` = implemented and verified enough to count as complete for the current AXIOM 1.0 gate.
- `[ ]` = not yet complete.
- A partially implemented feature remains `[ ]` until its semantics, implementation, diagnostics, tests, documentation, and required backend/runtime behavior agree.
- Historical prototype functionality does **not** automatically count as AXIOM 1.0.
- When a point becomes complete, mark it `[x]` and record the relevant commit or evidence when useful.
- Do not mark a point complete merely because it is documented or conceptually designed.
- This document does not replace the canonical specification. It tracks implementation completion against it.
- `Documento sin título.txt` remains the wider AXIOM SYSTEMS master roadmap and must not be modified.

## Definition of AXIOM 1.0

AXIOM 1.0 is complete only when the language has:

1. a closed canonical syntax and semantics;
2. a complete compiler pipeline;
3. a stable AXIOM IR;
4. a stable AXIOM ABI;
5. an AXIOM-owned runtime and native backend;
6. a sufficiently complete standard library for general-purpose use;
7. a complete package/build/test toolchain;
8. conformance and reproducibility evidence;
9. a compiler capable of building itself from canonical AXIOM;
10. no fundamental dependency on the Python or Rust transition implementations.

---

# A. Language identity and specification

- [x] A1. Define AXIOM's identity as a universal general-purpose language.
- [x] A2. Define the principle "Natural in expression, formal in meaning."
- [x] A3. Define the semantic pipeline: intention → entities → relations → resources → capabilities → effects → restrictions → contracts → plan → execution.
- [x] A4. Define the core entity categories: VALUE, DATA, RESOURCE, CAPABILITY, KNOWLEDGE.
- [x] A5. Define the core semantic relation model.
- [x] A6. Define Resource Flow as the primary resource-management model.
- [x] A7. Define Information Flow as a separate analysis from Resource Flow.
- [x] A8. Define effects, capabilities, resources, restrictions, preferences, and contracts as distinct concepts.
- [x] A9. Define compiler explainability as a native compiler responsibility.
- [x] A10. Define the language as independent of AI models, network services, accounts, and remote compilation services.
- [x] A11. Close the conceptual architecture for fault tolerance and recovery.
- [x] A12. Close the conceptual architecture for time, real-time behavior, and determinism.
- [x] A13. Close the conceptual architecture for distribution.
- [x] A14. Close the conceptual architecture for metaprogramming and reflection.
- [x] A15. Close the conceptual architecture for interoperability and AXIOM ABI.
- [x] A16. Close the conceptual architecture for language evolution and migration.
- [x] A17. Close the conceptual architecture for the complete ecosystem boundary.
- [x] A18. Perform the architectural identity audit.
- [ ] A19. Publish a final AXIOM 1.0 language specification with every normative semantic rule resolved.
- [ ] A20. Freeze the AXIOM 1.0 language-design change process.

# B. Canonical source language

- [x] B1. Establish one canonical human-facing AXIOM syntax.
- [x] B2. Establish English as the canonical AXIOM 0.1 frontend language.
- [x] B3. Establish future multilingual frontends as semantic frontend layers, not separate languages.
- [x] B4. Establish four-space indentation as the canonical block representation.
- [x] B5. Implement canonical expressions.
- [x] B6. Implement canonical assignment.
- [x] B7. Implement canonical conditional blocks.
- [x] B8. Implement canonical while/repetition control.
- [x] B9. Implement canonical collection repetition.
- [x] B10. Implement canonical when/event form.
- [x] B11. Implement canonical structure definitions.
- [x] B12. Implement typed entity instances.
- [x] B13. Implement canonical function definitions.
- [x] B14. Implement function calls.
- [x] B15. Implement final-expression functions.
- [x] B16. Implement canonical top-level directives.
- [ ] B17. Complete the normative grammar for every AXIOM 1.0 construct.
- [ ] B18. Remove ambiguity from every grammar production.
- [ ] B19. Define all lexical edge cases and invalid forms.
- [ ] B20. Retire historical syntax from the AXIOM 1.0 compatibility boundary.
- [ ] B21. Define canonical formatting rules for the complete language.
- [ ] B22. Define canonical source encoding and line-ending behavior.

# C. Lexer and parser

- [x] C1. Implement the canonical lexer/frontend path.
- [x] C2. Implement canonical parsing for the current source kernel.
- [x] C3. Preserve source locations through parsing.
- [x] C4. Produce a canonical AST.
- [x] C5. Cover the canonical frontend with executable tests.
- [ ] C6. Complete lexical diagnostics for all invalid token forms.
- [ ] C7. Complete parser diagnostics for all invalid grammar forms.
- [ ] C8. Implement robust parser error recovery.
- [ ] C9. Define and test source-span behavior for every AST construct.
- [ ] C10. Define parser compatibility guarantees for AXIOM 1.0.
- [ ] C11. Remove remaining dependence on transition-only parser behavior.
- [ ] C12. Make lexer/parser conformance exhaustive against the final grammar.

# D. Types and values

- [ ] D1. Finalize the primitive type system.
- [ ] D2. Implement boolean semantics completely.
- [ ] D3. Implement integer semantics completely.
- [ ] D4. Implement real/floating-point semantics completely.
- [ ] D5. Implement decimal semantics completely.
- [ ] D6. Implement text/string semantics completely.
- [ ] D7. Implement character semantics completely.
- [ ] D8. Implement bytes semantics completely.
- [ ] D9. Implement unit and dimensional types completely.
- [ ] D10. Implement time and duration types completely.
- [ ] D11. Implement optional/absence semantics without a universal null model.
- [ ] D12. Implement error/failure value semantics.
- [ ] D13. Implement arrays/collections with complete type semantics.
- [ ] D14. Implement structures/data types completely.
- [ ] D15. Implement enumeration/sum-style data where required by the final design.
- [ ] D16. Implement function values/closures.
- [ ] D17. Implement generic/generalized abstractions according to AXIOM's semantic model.
- [ ] D18. Implement type inference.
- [ ] D19. Implement deterministic type checking.
- [ ] D20. Implement valid and invalid implicit conversions.
- [ ] D21. Implement explicit conversions.
- [ ] D22. Implement compile-time representation/layout knowledge.
- [ ] D23. Implement scientific numeric types required by the final specification.
- [ ] D24. Implement vector/matrix/tensor semantics where part of the language core.
- [ ] D25. Define exact versus approximate arithmetic semantics.
- [ ] D26. Define overflow, underflow, NaN, infinity, and numeric edge behavior.
- [ ] D27. Define value identity, equality, ordering, and hashing semantics.
- [ ] D28. Test the complete type/value system through conformance tests.

# E. Names, scopes, modules, and packages

- [ ] E1. Complete lexical and semantic scopes.
- [ ] E2. Complete symbol-table implementation.
- [ ] E3. Complete deterministic name resolution.
- [ ] E4. Resolve local definitions deterministically.
- [ ] E5. Resolve parameters and local entities deterministically.
- [ ] E6. Resolve semantic spaces/modules deterministically.
- [ ] E7. Implement explicit module use/import semantics.
- [ ] E8. Implement public semantic interfaces.
- [ ] E9. Implement package dependency resolution.
- [ ] E10. Detect and explain ambiguous names.
- [ ] E11. Detect and explain unresolved names.
- [ ] E12. Implement module initialization semantics.
- [ ] E13. Implement package/module visibility and access rules.
- [ ] E14. Complete project/package/module conformance tests.

# F. Functions and execution semantics

- [ ] F1. Finalize function semantics.
- [ ] F2. Finalize parameter semantics.
- [ ] F3. Finalize return/final-expression semantics.
- [ ] F4. Implement closures and captured entities.
- [ ] F5. Implement recursion semantics.
- [ ] F6. Implement overload/specialization rules if retained.
- [ ] F7. Implement deterministic call resolution.
- [ ] F8. Implement function purity/effect classification.
- [ ] F9. Implement function contracts.
- [ ] F10. Implement complete call-graph analysis.
- [ ] F11. Implement interprocedural semantic propagation.
- [ ] F12. Test function semantics across all control-flow constructs.

# G. Resource Flow

- [x] G1. Define resource identity roots.
- [x] G2. Detect resource-like entities.
- [x] G3. Record resource creation.
- [x] G4. Track resource consumers.
- [x] G5. Track resource aliases.
- [x] G6. Track resource release provenance.
- [x] G7. Detect use-after-release.
- [x] G8. Handle conservative branch joins.
- [x] G9. Handle conservative loop behavior.
- [x] G10. Detect multiple independent consumers.
- [x] G11. Represent proven resource sharing.
- [x] G12. Track indexed resource views/subresources.
- [x] G13. Emit VIEW relations for indexed subresources.
- [x] G14. Define the complete Resource Flow state lattice.
- [x] G15. Track resource flow through function parameters interprocedurally.
- [x] G16. Track resources returned from functions.
- [ ] G17. Track resources captured by closures.
- [x] G18. Track resources through nested/composed views.
- [ ] G19. Track slices/subranges/subresources completely.
- [ ] G20. Distinguish aliasing, copying, moving, sharing, reuse, and views semantically.
- [ ] G21. Define sound transfer/move semantics through explicit contracts.
- [x] G22. Handle resource fields and structured resources.
- [ ] G23. Handle resource containers and collections.
- [ ] G24. Handle resource flow across concurrency boundaries.
- [ ] G25. Handle resource flow across process/machine boundaries.
- [ ] G26. Handle device/GPU/NPU/QPU resources.
- [ ] G27. Handle temporal resources and resource budgets.
- [ ] G28. Produce complete resource-flow explanations and diagnostics.
- [ ] G29. Verify Resource Flow against the complete normative specification.

# H. Information Flow, effects, capabilities, and restrictions

- [x] H1. Model effects separately from resources.
- [x] H2. Model capabilities separately from effects.
- [x] H3. Infer capabilities from effects.
- [x] H4. Propagate effects through known calls.
- [x] H5. Represent explicit capability declarations.
- [x] H6. Reject inferred capabilities forbidden by explicit capability declarations.
- [x] H7. Preserve requirements/preferences/restrictions/modes in the semantic model.
- [ ] H8. Complete the effect taxonomy.
- [ ] H9. Complete effect inference for every standard-library operation.
- [ ] H10. Complete transitive interprocedural effect propagation.
- [ ] H11. Complete capability provenance.
- [ ] H12. Complete capability conflict analysis.
- [ ] H13. Implement Information Flow provenance.
- [ ] H14. Implement information classifications/policies.
- [ ] H15. Detect prohibited information flows.
- [ ] H16. Explain the complete provenance of prohibited flows.
- [ ] H17. Complete hard restriction enforcement.
- [ ] H18. Complete soft preference handling.
- [ ] H19. Define precedence between restrictions, contracts, capabilities, and preferences.
- [ ] H20. Verify the complete effects/capabilities/information-flow model.

# I. Control flow, concurrency, time, and determinism

- [ ] I1. Complete control-flow semantic representation.
- [ ] I2. Complete control-flow analysis across all constructs.
- [ ] I3. Detect unreachable code where applicable.
- [ ] I4. Detect impossible control-flow states where applicable.
- [ ] I5. Model compiler-discovered parallelism.
- [ ] I6. Detect resource/data conflicts relevant to parallel execution.
- [ ] I7. Insert required synchronization in execution planning.
- [ ] I8. Preserve explicitly required sequential execution.
- [ ] I9. Complete task/concurrency runtime semantics.
- [ ] I10. Complete cancellation semantics.
- [ ] I11. Complete failure propagation across concurrent operations.
- [ ] I12. Define instant/duration/clock/period semantics.
- [ ] I13. Define deadline/latency/jitter semantics.
- [ ] I14. Implement real-time restrictions.
- [ ] I15. Implement deterministic execution restrictions.
- [ ] I16. Distinguish proven, feasible, unknown, and impossible timing claims.
- [ ] I17. Prevent unsupported compiler guarantees.
- [ ] I18. Test concurrency and determinism semantics reproducibly.

# J. Failure, recovery, and fault tolerance

- [ ] J1. Complete failure as a semantic possibility.
- [ ] J2. Implement retry semantics.
- [ ] J3. Enforce retry-safety/idempotency knowledge.
- [ ] J4. Implement timeout semantics.
- [ ] J5. Implement fallback semantics.
- [ ] J6. Implement restart semantics.
- [ ] J7. Implement isolation semantics.
- [ ] J8. Implement degradation semantics.
- [ ] J9. Implement replacement semantics.
- [ ] J10. Implement cancellation and recovery semantics.
- [ ] J11. Integrate failure cleanup with Resource Flow.
- [ ] J12. Model local, hardware, and distributed fault domains.
- [ ] J13. Produce diagnostics for unsafe recovery plans.

# K. Contracts and verification

- [ ] K1. Implement contract representation.
- [ ] K2. Implement input contracts.
- [ ] K3. Implement output contracts.
- [ ] K4. Implement invariants.
- [ ] K5. Implement resource-state contracts.
- [ ] K6. Implement capability/effect contracts.
- [ ] K7. Implement timing contracts.
- [ ] K8. Implement determinism contracts.
- [ ] K9. Implement security contracts.
- [ ] K10. Implement distribution contracts.
- [ ] K11. Implement evidence states: demonstrated, checked, measured, simulated, observed, assumed, unknown, refuted, not analyzed.
- [ ] K12. Preserve verification hypotheses and evidence provenance.
- [ ] K13. Produce counterexamples when verification fails and a counterexample is derivable.
- [ ] K14. Distinguish tests from proofs.
- [ ] K15. Implement the final verification engine.

# L. Distribution

- [ ] L1. Define distributed entities and nodes.
- [ ] L2. Define network/resource semantics.
- [ ] L3. Implement serialization semantics.
- [ ] L4. Implement distributed resource flow.
- [ ] L5. Implement latency and timeout semantics.
- [ ] L6. Implement partition semantics.
- [ ] L7. Implement consistency constraints.
- [ ] L8. Implement atomicity constraints.
- [ ] L9. Implement distributed idempotency.
- [ ] L10. Implement distributed recovery.
- [ ] L11. Compile local and distributed plans through the same semantic model.
- [ ] L12. Test distributed semantics independently of a particular transport.

# M. Metaprogramming and reflection

- [ ] M1. Define inspectable code entities.
- [ ] M2. Define inspectable type entities.
- [ ] M3. Define inspectable module entities.
- [ ] M4. Define inspectable schema entities.
- [ ] M5. Define inspectable contract entities.
- [ ] M6. Implement type inspection.
- [ ] M7. Implement module inspection.
- [ ] M8. Implement contract inspection.
- [ ] M9. Implement AST/code transformation.
- [ ] M10. Implement code generation.
- [ ] M11. Implement schema-to-code generation.
- [ ] M12. Implement binding generation.
- [ ] M13. Implement documentation generation.
- [ ] M14. Apply effect/capability/resource restrictions to metaprogramming.
- [ ] M15. Verify reflection behavior across compiler/runtime boundaries.

# N. Interoperability and AXIOM ABI

- [ ] N1. Freeze AXIOM value representation rules.
- [ ] N2. Freeze memory layout rules.
- [ ] N3. Freeze alignment rules.
- [ ] N4. Freeze calling conventions.
- [ ] N5. Freeze structure layout.
- [ ] N6. Freeze error representation.
- [ ] N7. Freeze resource representation.
- [ ] N8. Freeze capability representation.
- [ ] N9. Define ABI versioning.
- [ ] N10. Define binary compatibility rules.
- [ ] N11. Implement C interoperability.
- [ ] N12. Implement C++ interoperability.
- [ ] N13. Implement Rust interoperability.
- [ ] N14. Implement Python interoperability.
- [ ] N15. Implement .NET interoperability where supported.
- [ ] N16. Implement WebAssembly interoperability.
- [ ] N17. Implement native platform API interoperability.
- [ ] N18. Implement hardware/device interoperability.
- [ ] N19. Test ABI compatibility across compiler versions.
- [ ] N20. Document the final AXIOM ABI.

# O. AXIOM IR

- [x] O1. Establish AXIOM IR as an intermediate representation concept.
- [ ] O2. Freeze the final AXIOM IR specification.
- [ ] O3. Represent complete type information in IR.
- [ ] O4. Represent complete control flow in IR.
- [ ] O5. Represent memory/resource operations in IR.
- [ ] O6. Represent effects and capabilities where required.
- [ ] O7. Represent contracts and verification metadata where required.
- [ ] O8. Represent concurrency/execution-plan information.
- [ ] O9. Represent device/accelerator placement.
- [ ] O10. Represent distribution boundaries.
- [ ] O11. Define stable IR verification rules.
- [ ] O12. Implement complete AST/semantic-model → IR lowering.
- [ ] O13. Implement complete IR validation.
- [ ] O14. Implement stable IR serialization where required.
- [ ] O15. Test IR compatibility and deterministic generation.

# P. Compiler and optimization

- [ ] P1. Complete the canonical AXIOM compiler driver.
- [ ] P2. Complete semantic analysis pipeline integration.
- [ ] P3. Complete diagnostics pipeline.
- [ ] P4. Complete source-to-IR compilation.
- [ ] P5. Implement optimization pipeline.
- [ ] P6. Implement safe constant evaluation/folding.
- [ ] P7. Implement dead-code elimination where semantically valid.
- [ ] P8. Implement control-flow optimization.
- [ ] P9. Implement resource-aware optimization.
- [ ] P10. Implement effect-aware optimization.
- [ ] P11. Implement target-aware optimization.
- [ ] P12. Implement compiler explainability for major optimization decisions.
- [ ] P13. Implement deterministic compilation.
- [ ] P14. Implement reproducible compiler outputs.
- [ ] P15. Implement cross-compilation.
- [ ] P16. Implement complete compiler diagnostics and actionable fixes.
- [ ] P17. Ensure compiler repair proposals require explicit user approval before source changes.
- [ ] P18. Validate compiler behavior against the final conformance suite.

# Q. Native backends and targets

- [ ] Q1. Complete the AXIOM-owned native backend architecture.
- [ ] Q2. Complete the primary CPU backend.
- [ ] Q3. Support the required x86-64 target.
- [ ] Q4. Support the required ARM target.
- [ ] Q5. Define target-independent backend interfaces.
- [ ] Q6. Implement native executable generation.
- [ ] Q7. Implement native library generation.
- [ ] Q8. Implement object/linking behavior.
- [ ] Q9. Implement debugging metadata.
- [ ] Q10. Implement source mapping.
- [ ] Q11. Implement WebAssembly backend/target.
- [ ] Q12. Define GPU backend boundary.
- [ ] Q13. Define embedded backend boundary.
- [ ] Q14. Define accelerator/device backend boundary.
- [ ] Q15. Verify native output correctness against AXIOM semantics.

# R. AXIOM-owned runtime

- [ ] R1. Define final AXIOM runtime boundary.
- [ ] R2. Implement AXIOM-owned memory runtime.
- [ ] R3. Implement resource runtime.
- [ ] R4. Implement process/runtime primitives.
- [ ] R5. Implement thread/task execution primitives.
- [ ] R6. Implement asynchronous execution primitives where required.
- [ ] R7. Implement I/O runtime.
- [ ] R8. Implement filesystem runtime.
- [ ] R9. Implement networking runtime.
- [ ] R10. Implement timers/clock runtime.
- [ ] R11. Implement failure/recovery runtime.
- [ ] R12. Implement runtime reflection where required.
- [ ] R13. Implement hardware/device runtime boundary.
- [ ] R14. Remove fundamental dependency on the historical runtime implementation.
- [ ] R15. Verify runtime semantics against the canonical language specification.

# S. Standard library

- [ ] S1. Finalize standard-library ownership and compatibility policy.
- [ ] S2. Complete text/string library.
- [ ] S3. Complete collections library.
- [ ] S4. Complete filesystem library.
- [ ] S5. Complete process/system library.
- [ ] S6. Complete time/date library.
- [ ] S7. Complete mathematics library.
- [ ] S8. Complete statistics/linear algebra library.
- [ ] S9. Complete serialization library.
- [ ] S10. Complete JSON/XML/YAML/CSV/binary format support as required.
- [ ] S11. Complete networking library.
- [ ] S12. Complete HTTP/WebSocket/TCP/UDP/DNS/TLS support as required.
- [ ] S13. Complete cryptography/authentication/certificate support.
- [ ] S14. Complete compression support.
- [ ] S15. Complete database/SQL support.
- [ ] S16. Complete concurrency support.
- [ ] S17. Complete testing/benchmarking/fuzzing support.
- [ ] S18. Complete observability/logging support.
- [ ] S19. Complete FFI support.
- [ ] S20. Complete device/hardware/SIMD support.
- [ ] S21. Complete media codec boundaries required by the 1.0 scope.
- [ ] S22. Complete GPU/shader library boundaries required by the 1.0 scope.
- [ ] S23. Complete embedded-system library boundaries required by the 1.0 scope.
- [ ] S24. Ensure every standard-library operation has correct effects/capabilities/resource semantics.
- [ ] S25. Ensure the essential standard library can be built and tested without Python/Rust compiler internals.

# T. Package manager and build system

- [ ] T1. Finalize AXIOM package identity.
- [ ] T2. Finalize package manifests.
- [ ] T3. Implement deterministic dependency resolution.
- [ ] T4. Implement package integrity verification.
- [ ] T5. Implement package signatures/verification.
- [ ] T6. Implement dependency capability/effect metadata.
- [ ] T7. Implement ABI/platform compatibility metadata.
- [ ] T8. Implement reproducibility metadata.
- [ ] T9. Implement local package builds.
- [ ] T10. Implement package publishing workflow.
- [ ] T11. Implement registry interaction.
- [ ] T12. Implement offline/local package operation where required.
- [ ] T13. Ensure package builds do not require external services for compilation itself.
- [ ] T14. Test package resolution and build reproducibility.

# U. CLI and developer tooling

- [x] U1. Establish the `axiom` CLI.
- [x] U2. Establish `axiom new`.
- [x] U3. Establish `axiom check`.
- [x] U4. Establish `axiom run`.
- [x] U5. Establish `axiom build`.
- [x] U6. Establish `axiom test` compatibility with the existing test environment.
- [ ] U7. Complete `axiom test language`.
- [ ] U8. Complete `axiom test conformance`.
- [ ] U9. Complete `axiom test bootstrap`.
- [ ] U10. Complete `axiom test independence`.
- [ ] U11. Complete `axiom test reproducible`.
- [ ] U12. Complete `axiom package`.
- [ ] U13. Complete `axiom add`.
- [ ] U14. Complete `axiom install`.
- [ ] U15. Complete `axiom native-build`.
- [ ] U16. Complete `axiom doctor`.
- [ ] U17. Complete `axiom explain`.
- [ ] U18. Complete `axiom format`.
- [ ] U19. Complete `axiom lint`.
- [ ] U20. Complete `axiom migrate`.
- [ ] U21. Implement final debugger integration.
- [ ] U22. Implement final profiler integration.
- [ ] U23. Implement final compiler diagnostics UX.
- [ ] U24. Implement complete documentation/help discovery from the CLI.

# V. Conformance, testing, and proof of independence

- [x] V1. Establish executable conformance tests for the canonical frontend.
- [ ] V2. Build a complete language conformance suite from the normative specification.
- [ ] V3. Cover syntax conformance.
- [ ] V4. Cover type conformance.
- [ ] V5. Cover semantic conformance.
- [ ] V6. Cover effects/capabilities conformance.
- [ ] V7. Cover Resource Flow conformance.
- [ ] V8. Cover Information Flow conformance.
- [ ] V9. Cover contracts/verification conformance.
- [ ] V10. Cover execution/runtime conformance.
- [ ] V11. Cover ABI conformance.
- [ ] V12. Cover standard-library conformance.
- [ ] V13. Cover package/build conformance.
- [ ] V14. Implement AXIOM-native test reporting.
- [ ] V15. Implement bootstrap verification.
- [ ] V16. Implement self-compilation verification.
- [ ] V17. Implement compiler-output comparison across bootstrap stages.
- [ ] V18. Implement Python-independence verification.
- [ ] V19. Implement Rust-independence verification.
- [ ] V20. Implement no-network/no-external-service independence verification.
- [ ] V21. Implement reproducible-build verification.
- [ ] V22. Make the independence report explicit and machine-verifiable.
- [ ] V23. Reach a state where AXIOM can prove its own compiler chain is independent of Python.
- [ ] V24. Reach a state where AXIOM can prove its own compiler chain is independent of Rust.

# W. Self-hosting and bootstrap

- [x] W1. Establish the bootstrap seed.
- [x] W2. Establish AXIOM-written bootstrap lexer/parser/AST/semantic/compiler experiments.
- [ ] W3. Implement the canonical AXIOM lexer in canonical AXIOM.
- [ ] W4. Implement the canonical AXIOM parser in canonical AXIOM.
- [ ] W5. Implement the canonical AST model in canonical AXIOM.
- [ ] W6. Implement canonical semantic analysis in canonical AXIOM.
- [ ] W7. Implement type/name-resolution analysis in canonical AXIOM.
- [ ] W8. Implement Resource Flow analysis in canonical AXIOM.
- [ ] W9. Implement effect/capability analysis in canonical AXIOM.
- [ ] W10. Implement contract/verification support in canonical AXIOM.
- [ ] W11. Implement AXIOM IR generation in canonical AXIOM.
- [ ] W12. Implement the compiler driver in canonical AXIOM.
- [ ] W13. Compile the AXIOM compiler using AXIOM.
- [ ] W14. Recompile the AXIOM compiler with the compiler it produced.
- [ ] W15. Compare bootstrap outputs.
- [ ] W16. Establish stable self-hosting.
- [ ] W17. Remove Python from the fundamental compiler path.
- [ ] W18. Remove Rust from the fundamental compiler path.
- [ ] W19. Build AXIOM from AXIOM without the historical compiler/runtime foundations.
- [ ] W20. Verify the self-hosted compiler through the complete conformance suite.

# X. Reproducibility, security, and release engineering

- [ ] X1. Define the final AXIOM 1.0 build identity.
- [ ] X2. Make compiler builds reproducible.
- [ ] X3. Make package builds reproducible where supported.
- [ ] X4. Define deterministic artifact naming and paths.
- [ ] X5. Verify deterministic compiler behavior.
- [ ] X6. Define compiler supply-chain trust requirements.
- [ ] X7. Verify package origins and integrity.
- [ ] X8. Verify package signatures where supported.
- [ ] X9. Define capability/effect metadata for dependencies.
- [ ] X10. Define secure FFI boundaries.
- [ ] X11. Define sandbox/isolation boundaries where required.
- [ ] X12. Define security diagnostics.
- [ ] X13. Define release artifact verification.
- [ ] X14. Establish AXIOM 1.0 release gates.

# Y. Documentation and migration

- [ ] Y1. Complete the AXIOM 1.0 language reference.
- [ ] Y2. Complete the AXIOM 1.0 semantic specification.
- [ ] Y3. Complete the canonical syntax reference.
- [ ] Y4. Complete the compiler architecture reference.
- [ ] Y5. Complete the IR reference.
- [ ] Y6. Complete the ABI reference.
- [ ] Y7. Complete the runtime reference.
- [ ] Y8. Complete the standard-library reference.
- [ ] Y9. Complete the package/build reference.
- [ ] Y10. Complete the FFI reference.
- [ ] Y11. Complete the Resource Flow reference.
- [ ] Y12. Complete the effects/capabilities/Information Flow reference.
- [ ] Y13. Complete the contracts/verification reference.
- [ ] Y14. Complete the self-hosting/bootstrap documentation.
- [ ] Y15. Complete the independence methodology documentation.
- [ ] Y16. Complete the migration/versioning rules.
- [ ] Y17. Provide first-day learning material for the final language.
- [ ] Y18. Ensure examples compile with the final AXIOM 1.0 compiler.

# Z. Final AXIOM 1.0 gate

These are the final conditions. They are intentionally separate from individual implementation points.

- [ ] Z1. Canonical syntax is frozen.
- [ ] Z2. Canonical semantics are frozen.
- [ ] Z3. Type system is complete and tested.
- [ ] Z4. Name/module/package semantics are complete and tested.
- [ ] Z5. Resource Flow is complete and tested.
- [ ] Z6. Information Flow is complete and tested.
- [ ] Z7. Effects and capabilities are complete and tested.
- [ ] Z8. Contracts and verification are complete and tested.
- [ ] Z9. Concurrency/time/determinism semantics are complete and tested.
- [ ] Z10. Failure/recovery semantics are complete and tested.
- [ ] Z11. Distribution semantics are complete and tested.
- [ ] Z12. Metaprogramming/reflection are complete and tested.
- [ ] Z13. AXIOM IR is frozen and validated.
- [ ] Z14. AXIOM ABI is frozen and validated.
- [ ] Z15. Native backend is AXIOM-owned and validated.
- [ ] Z16. Runtime is AXIOM-owned and validated.
- [ ] Z17. Essential standard library is complete and validated.
- [ ] Z18. Package/build system is complete and reproducible.
- [ ] Z19. CLI/toolchain is complete.
- [ ] Z20. Conformance suite is complete.
- [ ] Z21. `axiom test` proves language conformance.
- [ ] Z22. `axiom test bootstrap` proves self-compilation.
- [ ] Z23. `axiom test independence` proves independence from Python.
- [ ] Z24. `axiom test independence` proves independence from Rust.
- [ ] Z25. Reproducible-build checks pass.
- [ ] Z26. Complete documentation matches implementation.
- [ ] Z27. No unresolved AXIOM 1.0 semantic TODO remains.
- [ ] Z28. No historical prototype behavior is required to define AXIOM 1.0.
- [ ] Z29. The AXIOM compiler can compile itself using canonical AXIOM.
- [ ] Z30. The resulting compiler can reproduce the same compiler/toolchain behavior.
- [ ] Z31. AXIOM can build and test itself without Python as a compiler foundation.
- [ ] Z32. AXIOM can build and test itself without Rust as a compiler foundation.
- [ ] Z33. AXIOM 1.0 passes the complete conformance suite.
- [ ] Z34. AXIOM 1.0 passes the complete bootstrap suite.
- [ ] Z35. AXIOM 1.0 passes the complete independence suite.
- [ ] Z36. AXIOM 1.0 passes the reproducibility suite.
- [ ] Z37. AXIOM 1.0 release artifacts are verified.
- [ ] Z38. AXIOM 1.0 is formally declared complete.

---

# Current snapshot

**Completed checklist points:** 69  
**Remaining checklist points:** 421  
**Total tracked points:** 490

The completed points represent architecture/specification work, the canonical frontend slice, the current semantic/resource-flow implementation slices, initial IR/runtime/toolchain foundations, and bootstrap foundations. They do **not** imply that AXIOM 1.0 is close to complete in implementation terms.

## Current active implementation frontier

The next implementation priorities are:

1. Complete the semantic model rather than adding superficial language features.
2. Finish Resource Flow semantics, especially interprocedural transfer, views, structured resources, and sound representation decisions.
3. Complete effects/capabilities/Information Flow.
4. Establish the final AXIOM IR.
5. Build the final compiler/backend/runtime path.
6. Build the AXIOM-native test/conformance/independence system.
7. Move the compiler progressively into canonical AXIOM.
8. Remove Python and then Rust from the fundamental compiler chain.
9. Close reproducibility and release gates.

## Evidence policy

A checkbox should normally be marked only when all applicable evidence exists:

- implementation;
- tests;
- diagnostics;
- documentation;
- semantic agreement with the specification;
- reproducibility/compatibility evidence when relevant.

A commit that implements a slice should not automatically mark the whole parent point complete.

## Relationship to other roadmap documents

This checklist is the **operational completion tracker for AXIOM 1.0**.

It complements, but does not replace:

1. `docs/axiom-spec-0.1.md` — semantic authority;
2. `docs/axiom-language.md` — canonical language direction;
3. `docs/architecture.md` — implementation architecture;
4. `docs/self-hosting.md` — self-hosting strategy;
5. `docs/version-roadmap.md` — release history/gates;
6. `ROADMAP.md` — historical implementation roadmap;
7. `Documento sin título.txt` — wider AXIOM SYSTEMS roadmap.

When this checklist conflicts with the specification, the specification wins and this checklist must be corrected.

---

# Completion declaration

When every required point above is complete and the final gates pass:

> **AXIOM 1.0 IS COMPLETE.**

At that point this document becomes the historical record of how AXIOM reached 1.0 rather than a list of future work.
