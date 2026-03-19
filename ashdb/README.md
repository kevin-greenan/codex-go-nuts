# AshDB

AshDB is the planned embedded database engine for Hearthlight and future Noema applications.

The target is a local, file-backed database library that feels closer to SQLite than to a client/server database: one file per database, embedded use from application code, schema-managed tables, indexes, and simple transactional behavior. The first versions should prioritize reliability and clarity over full SQL compatibility.

## Mission

AshDB should give Noema applications a durable local storage layer with:

1. A single-file database format.
2. Tables with typed columns.
3. Secondary indexes for common lookups.
4. Read and write transactions.
5. Crash recovery strong enough for application use.

The system does not need to implement all of SQL. It only needs enough query and storage power to support a serious local-first application.

## Product Shape

AshDB should be delivered as:

1. A Noema runtime library under this folder.
2. A file format specification for on-disk compatibility.
3. A small query API that Noema programs can call directly.
4. Validation and recovery tooling for development.

The long-term goal is for Hearthlight to use AshDB as its primary local data store.

## Repository Placement

AshDB should stay as a root-level project for now.

That split keeps responsibilities clear:

1. `ashdb/` owns the database engine, file format, examples, and tests.
2. `codex-lang/lib/` should hold only reusable low-level Noema standard-library style helpers that emerge from this work.
3. `codex-lang/` owns compiler, backend, and runtime changes required to support AshDB.

If parts of AshDB become small generic utilities later, we can promote those pieces into `codex-lang/lib/` without collapsing the whole database project into the language tree.

## Non-Goals For Early Versions

1. Full SQL parsing and compatibility.
2. Multi-process concurrent writes.
3. Advanced query optimization.
4. Replication or networking.
5. Arbitrary user-defined functions.

## Proposed Repository Layout

This folder should grow into:

1. `ashdb/lib/` for core Noema library code.
2. `ashdb/examples/` for smoke-test database programs.
3. `ashdb/docs/` for file format and query API docs.
4. `ashdb/tests/` for integration fixtures and recovery cases.
5. `ashdb/tools/` for inspection or debug utilities.

## Current Status

What is already working on `codex/ashdb-foundation`:

1. The Noema compiler now has the file and bytes support AshDB needs.
2. The canonical self-hosted compiler in [codex-lang/selfhost/compiler.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler.noe) is now native-only for its direct path.
3. AshDB has a working pager foundation with:
4. fixed-size pages
5. a file header page
6. page allocation
7. free-page reuse
8. simple page-kind markers
9. AshDB also has a first leaf-page storage layer with:
10. keyed rows
11. sorted inserts
12. duplicate-key update or reject behavior
13. leaf splitting
14. AshDB also has a first internal-node and table layer with:
15. root promotion from leaf to internal page
16. internal-node child routing
17. named-table catalog metadata
18. a first secondary-index layer
19. coarse rollback-journal transactions with recovery on reopen
20. point lookup by key
21. ordered scan across the current tree shape
22. row-field helpers for record-like payloads
23. schema metadata stored inside AshDB itself
24. typed-schema checked row insert, update, and put helpers for `text`, `i64`, and `bool`
25. small structured query helpers for secondary-index lookup and field-equality scans
26. validation reports with human-readable failure reasons
27. primary-key range scans
28. catalog and page-map inspection helpers
29. secondary-index rebuild helpers from row data
30. cursor-style first/next key traversal primitives
31. nullable-field and default-value support for row inserts
32. schema-declared unique `i64` fields with automatic index maintenance
33. page-aware rollback journal records instead of a single whole-file snapshot blob
34. schema-declared `i64` foreign-key style reference checks and delete protection
35. additive schema migration for new fields with backfill
36. unique secondary-index range scans for base-row retrieval
37. rowset-style query helpers that include primary keys in result rows
38. backup and restore copy helpers for committed snapshots
39. whole-database unique-index repair helper
40. changed-page rollback journaling instead of journaling every page up front
41. larger-tree stress coverage with rollback recovery checks
42. structural corruption-detection coverage
43. schema-aware index-assisted field equality lookups for unique `i64` fields
44. commit ordering now flushes database pages before clearing the rollback journal
45. stable cursor APIs for incremental table iteration
46. table compaction/rebuild helpers with freelist recovery for long-lived trees
47. tagged result-record conventions for lookup and cursor APIs
48. `i64` field-range predicate scans and rowset helpers
49. large-data stress coverage with 1,200-row insert/delete/compact/reopen validation
50. WAL-style commit recovery layered on top of rollback journaling
51. scalar coercion and canonicalization for `i64` and `bool` schema fields
52. field-rename schema migration for non-indexed/non-reference columns
53. table-layout and raw-page inspection helpers for debugging trees and page contents
54. smoke-test coverage through the direct self-hosted compiler

What is not done yet:

1. stronger transactional semantics beyond the current hybrid rollback-journal plus WAL recovery path
2. richer schema constraints beyond scalar types, nullable fields, defaults, and unique `i64` fields
3. broader corruption tooling and repair workflows beyond index rebuild
4. broader query shapes beyond point lookup, equality filtering, primary-key ranges, unique secondary-index ranges, key-by-key traversal, and rowset-style result helpers
5. any SQL surface

## Production Readiness Checklist

AshDB is no longer in the “blank engine” stage, but it is not production-ready yet. To call it production-ready for a serious local-first app, we still need all of the following:

### Storage Engine

1. better delete/rebalance behavior beyond explicit table compaction

### Schema and Data Model

1. broader schema evolution support beyond additive field migration and safe field renames

### Query Surface

1. broader predicate support beyond equality and `i64` field ranges
2. better result-shape APIs than joined text output

### Integrity and Recovery Tooling

1. broader repair-oriented helpers beyond unique-index repair
2. corruption fixtures and negative tests
3. clear recovery behavior documentation for interrupted writes

### Operational Safety

1. documented single-process writer guarantees or real file-locking support
2. backup/export story for application snapshots
3. import/restore story for development and migration
4. file-format documentation stable enough to preserve compatibility intentionally

### Testing and Confidence

1. more reopen-and-verify integration tests
2. randomized or generative mutation tests
3. crash-recovery simulations around commit boundaries
4. regression coverage for large trees and many splits
5. performance sanity checks so obvious pathologies show up early

### Current Priority Order

The best path from here is:

1. broader repair-oriented validation helpers
2. WAL-style durability and deeper commit-boundary recovery work
3. richer schema constraints and defaults
4. richer query helpers and cursor APIs
5. larger negative and recovery test coverage

## Architecture Overview

AshDB should be built in layers:

### 1. Pager

Responsible for opening the database file, reading and writing fixed-size pages, tracking dirty pages, and flushing durable changes.

### 2. Storage Format

Defines page headers, freelists, root pages, table metadata, row encoding, and index encoding.

### 3. B-Tree Layer

Provides table and index storage with page splitting, search, insertion, deletion, and cursor traversal.

### 4. Transaction Layer

Coordinates page caching, journaling or write-ahead logging, commit, rollback, and crash recovery.

### 5. Query API

Exposes a practical Noema-facing interface for:

1. Opening a database.
2. Defining tables and indexes.
3. Inserting rows.
4. Updating rows.
5. Deleting rows.
6. Scanning and filtering results.

## Recommended Query Model

The first version should not begin with SQL parsing. Instead it should expose structured Noema APIs such as:

1. `db_open(path)`
2. `db_begin(db)`
3. `db_create_table(tx, schema)`
4. `db_insert(tx, table, row)`
5. `db_select(tx, query)`
6. `db_update(tx, query, patch)`
7. `db_delete(tx, query)`
8. `db_commit(tx)`
9. `db_rollback(tx)`

If this API proves solid, a SQL-like parser can be added later as a separate layer.

## Data Model Direction

The first useful type system for AshDB should include:

1. `i64`
2. `bool`
3. `text`
4. nullable values
5. record-like rows assembled from named fields

Binary large objects can wait until the storage engine itself is stable.

## Required Noema Language and Runtime Support

AshDB cannot be built cleanly with the current Noema runtime alone. The current language has whole-file text reads and writes, but a SQLite-like engine needs page-oriented storage and durable updates.

The minimum additions we should plan for are:

### File Handle Support

Add an opaque file type and builtins for:

1. `file_open(path, mode)` or more explicit read/write variants.
2. `file_close(file)`.
3. `file_size(file)`.
4. `file_read(file, offset, len)`.
5. `file_write(file, offset, text_or_bytes)`.
6. `file_sync(file)`.

### Binary Storage Support

A page database wants byte-precise encoding. Text alone is too limiting and error-prone.

We should add:

1. A `bytes` type.
2. Literal and construction support for byte sequences.
3. `bytes_count`.
4. `bytes_slice`.
5. `bytes_concat`.
6. Integer encoding and decoding helpers for fixed-width values.

### Error Signaling

The runtime needs a better story for fallible system operations. We should add either:

1. tagged result records by library convention, or
2. a lightweight language-supported result type pattern.

The plan should not depend on exceptions.

### Optional Locking Support

If we want SQLite-like safety even in a single-host environment, we may later need:

1. advisory file locking, or
2. a clearly documented single-process writer limitation for phase one.

## Compiler Work Required

Every runtime addition above must be implemented in both compiler tracks:

1. Rust stage-1 compiler in `codex-lang/compiler/src/main.rs`.
2. Canonical self-hosted compiler in `codex-lang/selfhost/compiler.noe`.

That means updating:

1. Type recognition.
2. Builtin typing rules.
3. C backend lowering.
4. Native `arm64` backend lowering.
5. Runtime support code emitted by both compilers.
6. Example and test coverage for bootstrap safety.

## Bootstrap Safety Plan

Compiler work must not strand the self-hosting path. For every new builtin or type:

1. Add it to the Rust compiler first.
2. Add a tiny retained example that uses it.
3. Add the same typing and lowering support to the Noema compiler.
4. Rebuild the direct compiler.
5. Run the bootstrap fixed-point checks.

No AshDB core code should begin until the supporting builtins work in both compiler tracks.

## Delivery Phases

### Phase 0: Runtime Foundations

Deliver the minimum language/runtime support AshDB needs:

1. `file` handle type.
2. `bytes` type.
3. positioned file read and write operations.
4. file sync.
5. tests in both C and native backends.

Exit criteria:

1. A Noema example can create a file, write bytes at a chosen offset, read them back, and verify the result.
2. Bootstrap checks still pass.

Status:

1. Complete enough for AshDB feature work.
2. `file` support is implemented.
3. `bytes` support is implemented.
4. AshDB examples compile and run with the self-hosted direct compiler.

### Phase 1: Pager and File Format

Build the persistent storage skeleton:

1. fixed-size pages
2. file header
3. page allocation
4. freelist management
5. page checksum or sanity markers

Exit criteria:

1. A Noema program can create a database file, allocate pages, reopen it, and recover page metadata correctly.

Status:

1. In progress, mostly complete for the first storage slice.
2. Header page, allocation, free-page reuse, and page-kind markers are implemented.
3. We still need the file format to settle around table roots, page headers, and invariants that later B-tree code can rely on.

### Phase 2: Table Storage

Implement a B-tree table structure:

1. leaf pages
2. internal pages
3. row encoding
4. sorted insertion
5. point lookup by primary key
6. page split mechanics
7. root promotion when a tree grows

Exit criteria:

1. A test database can insert, persist, reopen, and read hundreds of rows correctly.

Status:

1. In progress.
2. The current branch has keyed leaf storage, internal-node routing, root promotion, table scans, record-like row helpers, and a small named-table catalog with schema metadata.
3. The next concrete milestones are:
4. richer schema constraints and defaults
5. richer query helpers
6. validation and repair tooling
7. more index/query integration
8. cursor-style scans

### Phase 3: Transactions and Recovery

Add durability semantics:

1. rollback journal or WAL
2. atomic commit path
3. recovery after interrupted writes
4. corruption detection tools

Exit criteria:

1. Simulated interrupted writes recover to a valid prior state.

Status:

1. Not started.
2. A first rollback-journal path now exists.
3. The next durability work should improve from whole-file snapshots toward page-aware journaling or WAL-style behavior.

### Phase 4: Secondary Indexes and Query API

Make the database application-ready:

1. secondary indexes
2. table scans
3. predicate filtering
4. update and delete
5. schema metadata

Exit criteria:

1. Hearthlight-style data models can be expressed and queried through the AshDB API.

Status:

1. Not started.
2. The API now has a first structured shape through `table_*` and `db_*` calls, including row helpers, schema lookup, typed row validation, index lookup, field-equality scans, validation reports, primary-key range scans, inspection helpers, and index rebuild helpers.
3. It should continue growing as structured Noema calls, not SQL.

## Testing Strategy

AshDB needs more than happy-path unit tests.

We should maintain:

1. small deterministic file-format tests
2. reopen-and-verify integration tests
3. simulated crash recovery tests
4. corruption detection tests
5. compiler regression tests for every supporting builtin

## First Implementation Targets

Completed:

1. a `bytes`-oriented runtime smoke test
2. a random-access file I/O smoke test
3. a page allocator prototype
4. keyed leaf-page smoke tests

Next:

1. richer schema constraints and defaults
2. richer field-aware query helpers
3. page-aware journaling or WAL-style durability
4. repair-oriented validation tooling
5. cursor-style scans

## Definition of Success

AshDB succeeds when a Noema application can depend on it for local structured storage without shelling out to another database engine, and when the compiler/runtime changes required for it remain stable enough to bootstrap the language cleanly.
