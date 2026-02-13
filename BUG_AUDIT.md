# Bug & Error-Handling Audit

This file lists concrete bugs and incorrect error-handling patterns found during a quick code review, with a scenario and a practical fix for each one.

## 1) Silent partial failure while syncing IMDb metadata

- **Location:** `src-tauri/src/fetch_imdb.rs` + `src-tauri/src/fetch_imdb/imdbot.rs` + `src-tauri/src/lib.rs` (`sync_files` path)
- **What is wrong:** `set_imdb_data` returns `()` and logs errors internally. `sync_files` cannot distinguish full success from partial IMDb failure.
- **Bug scenario:** IMDb API is rate-limited or temporarily down. File scan/import still succeeds, but many items are saved without IMDb. User sees incomplete metadata with no actionable error.
- **How to fix:** Make `set_imdb_data` return structured status (e.g. `Result<ImdbSyncStats>` with counts + failed items), then surface warning/error to UI.

## 2) Sync progress can misreport inserted count

- **Location:** `src-tauri/src/lib.rs` (`sync_files`)
- **What is wrong:** Progress increments with `chunk.len()` whenever `insert_medias` succeeds, even if internal dedupe/merge means fewer new rows are actually inserted.
- **Bug scenario:** UI shows "inserted 100/100" while DB may have inserted fewer rows due to duplicates. Users trust wrong progress metrics.
- **How to fix:** Make DB layer return inserted/new/updated counts and emit those real numbers in progress events.

---

## Suggested execution order

1. Improve #1 and #2 for sync reliability and progress accuracy.
