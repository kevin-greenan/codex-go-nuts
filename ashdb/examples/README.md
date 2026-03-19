# AshDB Example Suites

AshDB keeps its smoke programs small and focused on purpose. They are easier to debug this way than one giant fixture.

Recommended grouping:

## Core

1. `pager_smoke.noe`
2. `leaf_smoke.noe`
3. `leaf_split_smoke.noe`
4. `internal_smoke.noe`
5. `table_smoke.noe`
6. `db_smoke.noe`

## Schema

1. `schema_smoke.noe`
2. `defaults_smoke.noe`
3. `coercion_smoke.noe`
4. `unique_smoke.noe`
5. `foreign_key_smoke.noe`
6. `migrate_smoke.noe`
7. `rename_smoke.noe`
8. `drop_field_smoke.noe`
9. `patch_smoke.noe`

## Query

1. `query_smoke.noe`
2. `range_smoke.noe`
3. `predicate_smoke.noe`
4. `compound_predicate_smoke.noe`
5. `text_prefix_smoke.noe`
6. `index_smoke.noe`
7. `index_range_smoke.noe`
8. `field_lookup_smoke.noe`
9. `row_smoke.noe`
10. `rowset_smoke.noe`
11. `result_smoke.noe`
12. `cursor_smoke.noe`

## Recovery And Operations

1. `tx_smoke.noe`
2. `journal_smoke.noe`
3. `commit_smoke.noe`
4. `wal_smoke.noe`
5. `recovery_boundary_smoke.noe`
6. `recovery_state_smoke.noe`
7. `lock_smoke.noe`
8. `backup_smoke.noe`
9. `compact_smoke.noe`
10. `repair_smoke.noe`
11. `repair_all_smoke.noe`
12. `repair_database_smoke.noe`
13. `reclaim_smoke.noe`
14. `inspect_smoke.noe`
15. `validate_smoke.noe`

## Stress And Corruption

1. `stress_smoke.noe`
2. `large_smoke.noe`
3. `reopen_regress_smoke.noe`
4. `corruption_smoke.noe`
5. `freelist_corruption_smoke.noe`
6. `orphan_validate_smoke.noe`
7. `root_collapse_smoke.noe`

## PR Acceptance Set

For merge confidence on this branch, the highest-signal set is:

1. `repair_database_smoke.noe`
2. `reopen_regress_smoke.noe`
3. `lock_smoke.noe`
4. `validate_smoke.noe`

Use [tools/run_direct_smokes.sh](/Users/kevin/Documents/Projects/AI/codex-go-nuts/ashdb/tools/run_direct_smokes.sh) to run grouped suites with the pure direct compiler path.
