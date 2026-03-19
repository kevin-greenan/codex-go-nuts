# AshDB Operations Guide

This document describes the current operational contract for AshDB.

## Writer Model

AshDB should currently be treated as:

1. single-process
2. single-writer
3. safe for repeated reopen in the same application lifecycle

What this means in practice:

1. Only one process should mutate a given database file at a time.
2. A second process should not open the same file for concurrent writes.
3. Readers in other processes are not yet a supported compatibility target.

AshDB does not yet implement OS-level file locking. The safety model right now is an application-level guarantee: one owner process per database file.

## Transaction Use

Recommended write flow:

1. `db_begin(...)`
2. perform inserts, updates, deletes, or migration work
3. `db_commit(...)` on success
4. `db_rollback(...)` on failure

Recommended read flow:

1. open the database
2. read using table/query helpers
3. close the database cleanly when finished

## Backup And Restore

AshDB currently supports committed snapshot copy operations:

1. `db_backup(db, backup_path)`
2. `db_restore_copy(backup_path, restore_path)`

Current expectations:

1. Backups should be taken when no write transaction is open.
2. A backup is a committed snapshot, not a live replication stream.
3. Restores produce a separate database file.

## Migration And Repair

For controlled maintenance, the current recommended order is:

1. take a backup snapshot
2. reopen the restored copy in test or staging if possible
3. run schema migration helpers
4. run `db_validate_report(...)`
5. rebuild or reclaim if needed:
6. `db_repair_unique_indexes(...)`
7. `db_reclaim_unreachable_pages(...)`
8. `db_compact_database(...)` when long-lived fragmentation is a concern

## Recovery Inspection

To inspect the recovery state around an interrupted write or crash boundary:

1. use `db_recovery_state_text(...)`
2. inspect table shape with `db_table_layout_text(...)`
3. inspect raw pages with `db_page_text(...)`
4. confirm structural health with `db_validate_report(...)`

## Stability Notes

What is stable enough for application experimentation:

1. file-backed tables and indexes
2. backup/restore copy workflow
3. rollback + WAL-assisted reopen recovery
4. schema-managed row storage

What still needs caution:

1. concurrent multi-process access
2. long-lived heavy-delete workloads without periodic compaction
3. assuming a frozen long-term file format without consulting the current docs
