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

AshDB now acquires an advisory exclusive lock on the main database file during `db_open(...)`.

Current lock behavior:

1. the first writer process acquires the database-file lock
2. a second open in another process or another file handle will observe `lock_held=false`
3. mutating APIs reject writes when `lock_held=false`
4. the lock is released during `db_close(...)`

This is a real OS-level advisory lock, but it is still best treated as a single-writer system rather than a fully concurrent database.

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
5. run `db_repair_database_report(...)` for a broad repair-and-validate pass when the goal is to normalize the file quickly
6. or run targeted maintenance helpers:
7. `db_repair_unique_indexes(...)`
8. `db_reclaim_unreachable_pages(...)`
9. `db_compact_database(...)` when long-lived fragmentation is a concern

## Recovery Inspection

To inspect the recovery state around an interrupted write or crash boundary:

1. use `db_recovery_state_text(...)`
2. inspect table shape with `db_table_layout_text(...)`
3. inspect raw pages with `db_page_text(...)`
4. confirm structural health with `db_validate_report(...)`

`db_recovery_state_text(...)` now also reports `lock_held`.

## Stability Notes

What is stable enough for application experimentation:

1. file-backed tables and indexes
2. backup/restore copy workflow
3. rollback + WAL-assisted reopen recovery
4. schema-managed row storage

What still needs caution:

1. concurrent multi-process access beyond the current advisory single-writer model
2. long-lived heavy-delete workloads without periodic compaction
3. assuming a frozen long-term file format without consulting the current docs
