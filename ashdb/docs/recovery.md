# AshDB Recovery And Durability

AshDB currently uses a hybrid durability model:

1. A rollback journal records original page images for pages touched inside an open transaction.
2. A WAL file records committed page images before the journal is cleared.

This is not a full multi-version WAL design. It is a pragmatic embedded recovery path aimed at single-process application use.

## Files

For a database at `app.db`, AshDB may use:

1. `app.db`
2. `app.db.journal`
3. `app.db.wal`

## Transaction States

### Idle

In the idle state:

1. the journal is not active
2. the WAL is not marked committed
3. the database file is the current source of truth

### Transaction Open

After `db_begin(...)`:

1. the journal header is marked active
2. the journal entry count starts at `0`
3. page images are appended lazily as pages are first touched

### Dirty Transaction

After writes inside an open transaction:

1. the journal remains active
2. journal entries reflect the original page images needed for rollback
3. the database file may contain newer page content than the committed state

### Commit Boundary

During `db_commit(...)`:

1. committed page images are written to the WAL
2. the WAL is flushed
3. the WAL header is marked committed
4. database pages are flushed
5. the rollback journal is cleared
6. the WAL is cleared

This ordering means recovery can prefer committed WAL replay if a crash happens after the WAL is marked committed but before the journal is cleared.

## Recovery Rules On Open

When `db_open(...)` runs, recovery proceeds in this order:

1. If the WAL exists and is marked committed, replay WAL pages into the main database file.
2. Flush the database file.
3. Clear the WAL.
4. Clear the rollback journal.
5. If no committed WAL is present but the rollback journal is active, restore original page images from the journal.
6. Flush the database file and clear the journal.

## Expected Outcomes For Interrupted Writes

### Crash Before WAL Commit

If a crash happens while a transaction is open and before the WAL is marked committed:

1. the rollback journal should still be active
2. reopening should restore the pre-transaction page images
3. uncommitted updates should disappear

### Crash After WAL Commit But Before Journal Clear

If a crash happens after the WAL is marked committed:

1. reopening should replay committed WAL pages first
2. the committed transaction should survive
3. the WAL and journal should both be cleared after recovery

### Explicit Rollback

After `db_rollback(...)`:

1. original page images are restored from the journal
2. the journal is cleared
3. the WAL should remain uncommitted

## Inspection Helpers

AshDB now exposes `db_recovery_state_text(...)` for smoke tests and debugging. It reports:

1. `journal_active`
2. `journal_entries`
3. `wal_committed`
4. `wal_entries`
5. `page_count`
6. `free_head`
7. `lock_held`

## Current Limitations

1. This design assumes a single process writing the database.
2. There is no file-locking protocol yet.
3. WAL entries are whole-page images for the touched set, not logical records.
4. The design is aimed at correctness and debuggability before throughput.
