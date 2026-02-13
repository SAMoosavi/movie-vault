# Bug & Error-Handling Audit

This file lists concrete bugs and incorrect error-handling patterns found during a quick code review, with a scenario and a practical fix for each one.

## 1) Silent partial failure while syncing IMDb metadata

- **Location:** `src-tauri/src/fetch_imdb.rs` + `src-tauri/src/fetch_imdb/imdbot.rs` + `src-tauri/src/lib.rs` (`sync_files` path)
- **What is wrong:** `set_imdb_data` returns `()` and logs errors internally. `sync_files` cannot distinguish full success from partial IMDb failure.
- **Bug scenario:** IMDb API is rate-limited or temporarily down. File scan/import still succeeds, but many items are saved without IMDb. User sees incomplete metadata with no actionable error.
- **How to fix:** Make `set_imdb_data` return structured status (e.g. `Result<ImdbSyncStats>` with counts + failed items), then surface warning/error to UI.

## 2) API typing bug on frontend for `get_medias_by_tag`

- **Location:** `src/functions/invoker.ts`
- **What is wrong:** Function is declared `Promise<Media>` but backend returns a list (`Vec<Media>`).
- **Bug scenario:** Caller treats return as one media object (`result.id`) while actual runtime value is array. This causes runtime bugs and TypeScript trust mismatch.
- **How to fix:** Change signature to `Promise<Media[]>` and update call sites if they assumed a single item.

## 3) Debug-build-only name parsing failure can produce empty titles

- **Location:** `src-tauri/src/data_model/media.rs` (`detect_name`)
- **What is wrong:** In debug builds, if input is not lowercase, function logs warning and returns empty string.
- **Bug scenario:** A refactor or new call site passes mixed-case filename in debug/dev mode; parsed media name becomes empty, causing bad search queries, duplicate grouping issues, and confusing UI entries.
- **How to fix:** Remove the early return in debug branch. Keep warning log, but continue processing with normalized lowercase.

## 4) Ranking update has no domain validation

- **Location:** `src-tauri/src/lib.rs` + `src-tauri/src/db/sqlite.rs` (`update_media_my_ranking`)
- **What is wrong:** Accepts any `u8` value and stores it directly.
- **Bug scenario:** A malformed frontend payload sends `255`; DB stores impossible ranking. Sorting/analytics relying on expected range (e.g. 0-10) become inconsistent.
- **How to fix:** Validate allowed range in command handler (and optionally DB constraint), returning validation error for out-of-range values.

## 5) Import data flow does not refresh frontend state

- **Location:** `src/pages/settings/DataSetting.vue`
- **What is wrong:** After successful `import_data`, UI shows success toast but does not refresh media/tag stores.
- **Bug scenario:** User imports backup, sees success message, but current screen still shows stale old data until manual reload/navigation.
- **How to fix:** Trigger relevant store reload actions (e.g. media and tags) immediately after successful import.

## 6) Sync progress can misreport inserted count

- **Location:** `src-tauri/src/lib.rs` (`sync_files`)
- **What is wrong:** Progress increments with `chunk.len()` whenever `insert_medias` succeeds, even if internal dedupe/merge means fewer new rows are actually inserted.
- **Bug scenario:** UI shows "inserted 100/100" while DB may have inserted fewer rows due to duplicates. Users trust wrong progress metrics.
- **How to fix:** Make DB layer return inserted/new/updated counts and emit those real numbers in progress events.

---

## Suggested execution order

1. Fix #2 and #3 first (typing correctness + debug parsing bug).
2. Fix #4 and #5 next (data integrity + import UX consistency).
3. Improve #1 and #6 for sync reliability and progress accuracy.
