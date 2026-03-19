# AshDB File Format

This document describes the current on-disk layout used by AshDB.

The format is intended to be simple, inspectable, and stable enough to reason about during engine development. It is not yet declared permanently frozen, but it is now documented intentionally.

## Files

A database rooted at `app.db` may use:

1. `app.db` for the main database file
2. `app.db.journal` for rollback journal state
3. `app.db.wal` for committed page images awaiting replay

## Main Database File

The main database file is page-oriented.

### Page 0: Pager Header

Page `0` stores the pager header:

1. page kind: `H`
2. magic: `ASHDB001`
3. page size: 8 decimal digits
4. total page count: 8 decimal digits
5. freelist head page id: 8 decimal digits

### Page 1: Catalog

Page `1` is the AshDB catalog page:

1. page marker: `C`
2. entry count: 8 decimal digits
3. fixed-width catalog entries

Each catalog entry stores:

1. used flag
2. root page id
3. slot width
4. normalized table name

The catalog maps:

1. base tables
2. schema metadata table
3. secondary-index tables

## Data Pages

Allocated pages are typically wrapped as pager data pages with outer marker `D`.

The first byte of the wrapped payload determines the logical page kind.

### Leaf Pages

Leaf pages use logical marker `L`.

They store:

1. slot count
2. slot width
3. fixed-width row slots

Rows are keyed by the table primary key, and AshDB currently uses fixed-width payload storage per table.

### Internal Pages

Internal pages use logical marker `I`.

They store:

1. key count
2. left child page id
3. ordered separator cells

Each separator cell stores:

1. separator key
2. right child page id

Internal pages route lookups and inserts through the table or index B-tree.

### Free Pages

Free pages use raw marker `F`.

They store:

1. next free page id

Free pages are linked together through the freelist head recorded in the pager header.

## Row Storage

Rows are stored as text payloads using named fields. At the logical API level, rows look like:

1. `name=alice;role=cook;active=true;`

Schema helpers interpret these row payloads for:

1. `text`
2. `i64`
3. `bool`
4. nullable values
5. defaults
6. unique `i64` fields
7. foreign-key style `i64` references

## Sidecar Recovery Files

### Rollback Journal

The journal stores:

1. active flag
2. original page count
3. entry count
4. journal entries of:
5. page id
6. original page image

The rollback journal is used to restore pre-transaction page images.

### WAL

The WAL stores:

1. committed flag
2. entry count
3. WAL entries of:
4. page id
5. committed page image

The WAL is replayed first during recovery when marked committed.

## Table Naming

Base tables use their declared names.

Secondary indexes currently use derived names:

1. `idx.<table>.<index>`

The schema metadata table uses:

1. `__schema`

## Stability Notes

The current documented format should be treated as:

1. intentionally described
2. suitable for debugging and compatibility planning
3. still open to carefully managed evolution while AshDB hardens
