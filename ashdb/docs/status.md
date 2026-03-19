# AshDB Status Inventory

This document captures the detailed implementation inventory for AshDB.

What is already working:

1. The Noema compiler now has the file and bytes support AshDB needs.
2. The canonical self-hosted compiler in [codex-lang/selfhost/compiler.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler.noe) is now native-only for its direct path.
3. fixed-size pages
4. a file header page
5. page allocation
6. free-page reuse
7. simple page-kind markers
8. keyed rows
9. sorted inserts
10. duplicate-key update or reject behavior
11. leaf splitting
12. root promotion from leaf to internal page
13. internal-node child routing
14. named-table catalog metadata
15. a first secondary-index layer
16. coarse rollback-journal transactions with recovery on reopen
17. point lookup by key
18. ordered scan across the current tree shape
19. row-field helpers for record-like payloads
20. schema metadata stored inside AshDB itself
21. typed-schema checked row insert, update, and put helpers for `text`, `i64`, and `bool`
22. small structured query helpers for secondary-index lookup and field-equality scans
23. validation reports with human-readable failure reasons
24. primary-key range scans
25. catalog and page-map inspection helpers
26. secondary-index rebuild helpers from row data
27. cursor-style first/next key traversal primitives
28. nullable-field and default-value support for row inserts
29. schema-declared unique `i64` fields with automatic index maintenance
30. page-aware rollback journal records instead of a single whole-file snapshot blob
31. schema-declared `i64` foreign-key style reference checks and delete protection
32. additive schema migration for new fields with backfill
33. unique secondary-index range scans for base-row retrieval
34. rowset-style query helpers that include primary keys in result rows
35. backup and restore copy helpers for committed snapshots
36. whole-database unique-index repair helper
37. changed-page rollback journaling instead of journaling every page up front
38. larger-tree stress coverage with rollback recovery checks
39. structural corruption-detection coverage
40. schema-aware index-assisted field equality lookups for unique `i64` fields
41. commit ordering now flushes database pages before clearing the rollback journal
42. stable cursor APIs for incremental table iteration
43. table compaction/rebuild helpers with freelist recovery for long-lived trees
44. tagged result-record conventions for lookup and cursor APIs
45. `i64` field-range predicate scans and rowset helpers
46. large-data stress coverage with 1,200-row insert/delete/compact/reopen validation
47. WAL-style commit recovery layered on top of rollback journaling
48. scalar coercion and canonicalization for `i64` and `bool` schema fields
49. field-rename schema migration for non-indexed/non-reference columns
50. table-layout and raw-page inspection helpers for debugging trees and page contents
51. unreachable-page detection and reclaim helpers for broader repair workflows
52. field-drop schema migration for non-indexed/non-reference columns
53. compound equality predicate scans across multiple fields
54. validation now flags unreachable pages as a corruption signal
55. a recovery-state inspection helper for journal and WAL state
56. written recovery and durability documentation in `ashdb/docs/recovery.md`
57. text-prefix predicate scans for `text` fields
58. an operations guide covering single-writer assumptions and backup/restore workflows
59. a file-format specification in `ashdb/docs/file-format.md`
60. negative recovery-boundary coverage for tampered WAL headers
61. freelist validation for cycles, bad markers, and unlinked free pages
62. freelist corruption coverage
63. limited root collapse after deletes when a two-leaf root empties one side
64. advisory single-writer file locking on the database handle
65. partial row patch helpers for app-style field updates
66. one-shot repair reporting that combines unique-index rebuild, unreachable-page reclaim, compaction, and validation
67. heavier reopen-and-verify regression coverage across commit, rollback, delete, patch, and compaction flows
68. smoke-test coverage through the direct self-hosted compiler
