# Bug & Error-Handling Audit

This file lists concrete bugs and incorrect error-handling patterns found during a quick code review, with a scenario and a practical fix for each one.

## 1) Wrong ID returned after creating media from IMDb
- **Location:** `src-tauri/src/lib.rs` (`create_media_from_imdb`)
- **What is wrong:** The code inserts the media into DB, but returns `media.id` from the local struct instead of the inserted ID returned by `db.insert_media`.
- **Bug scenario:** User adds a new movie from IMDb. Backend inserts row with a real DB ID (e.g. `42`) but command returns default `media.id` (often `0`). Frontend navigates to `/media/0` and fails with "movie not found".
- **How to fix:** Capture the return value from `db.insert_media(&media)` (e.g. `let new_id = ...?;`) and return `Ok(new_id)`.

## 2) Panic risk when updating IMDb for a non-existing media
- **Location:** `src-tauri/src/db/sqlite.rs` (`update_media_imdb`)
- **What is wrong:** Uses `Self::get_media_by_id(...)? .unwrap()`.
- **Bug scenario:** UI sends stale/deleted media ID. `get_media_by_id` returns `None`, `.unwrap()` panics, command crashes instead of returning a controlled error.
- **How to fix:** Replace `unwrap()` with explicit handling:
  - `ok_or_else(|| anyhow!("Media with id {} not found", media_id))?`
  - Propagate as `Result` so frontend gets a clean error.

## 3) Silent partial failure while syncing IMDb metadata
- **Location:** `src-tauri/src/fetch_imdb.rs` + `src-tauri/src/fetch_imdb/imdbot.rs` + `src-tauri/src/lib.rs` (`sync_files` path)
- **What is wrong:** `set_imdb_data` returns `()` and logs errors internally. `sync_files` cannot distinguish full success from partial IMDb failure.
- **Bug scenario:** IMDb API is rate-limited or temporarily down. File scan/import still succeeds, but many items are saved without IMDb. User sees incomplete metadata with no actionable error.
- **How to fix:** Make `set_imdb_data` return structured status (e.g. `Result<ImdbSyncStats>` with counts + failed items), then surface warning/error to UI.

## 4) API typing bug on frontend for `get_medias_by_tag`
- **Location:** `src/functions/invoker.ts`
- **What is wrong:** Function is declared `Promise<Media>` but backend returns a list (`Vec<Media>`).
- **Bug scenario:** Caller treats return as one media object (`result.id`) while actual runtime value is array. This causes runtime bugs and TypeScript trust mismatch.
- **How to fix:** Change signature to `Promise<Media[]>` and update call sites if they assumed a single item.

## 5) Error context is dropped in several command handlers
- **Location:** `src-tauri/src/lib.rs` (multiple `map_err(|e| e.to_string())`)
- **What is wrong:** Errors are stringified immediately; category/context is flattened.
- **Bug scenario:** Frontend receives generic message, cannot decide whether failure is network, validation, not-found, or DB conflict; UX cannot show targeted recovery hints.
- **How to fix:** Use structured error payloads (error code + message), e.g. enum serialized to frontend (`NOT_FOUND`, `VALIDATION`, `NETWORK`, `DB_CONFLICT`) and preserve source context.

## 6) Event listener leak for `sync-progress`
- **Location:** `src/App.vue`
- **What is wrong:** `listen('sync-progress', ...)` is called at module scope and the returned unlisten callback is never awaited/stored/cleaned up.
- **Bug scenario:** During hot reload or remounts in development, multiple listeners accumulate; progress events are handled multiple times and can show duplicated logs/UI updates.
- **How to fix:** Register the listener inside `onMounted`, store returned unlisten function, and call it in `onBeforeUnmount`.

## 7) Debug-build-only name parsing failure can produce empty titles
- **Location:** `src-tauri/src/data_model/media.rs` (`detect_name`)
- **What is wrong:** In debug builds, if input is not lowercase, function logs warning and returns empty string.
- **Bug scenario:** A refactor or new call site passes mixed-case filename in debug/dev mode; parsed media name becomes empty, causing bad search queries, duplicate grouping issues, and confusing UI entries.
- **How to fix:** Remove the early return in debug branch. Keep warning log, but continue processing with normalized lowercase.

## 8) Ranking update has no domain validation
- **Location:** `src-tauri/src/lib.rs` + `src-tauri/src/db/sqlite.rs` (`update_media_my_ranking`)
- **What is wrong:** Accepts any `u8` value and stores it directly.
- **Bug scenario:** A malformed frontend payload sends `255`; DB stores impossible ranking. Sorting/analytics relying on expected range (e.g. 0-10) become inconsistent.
- **How to fix:** Validate allowed range in command handler (and optionally DB constraint), returning validation error for out-of-range values.

## 9) Update setting may not persist across app restart
- **Location:** `src/functions/update.ts` (`setAutoUpdate`)
- **What is wrong:** Writes `store.set('autoUpdate', enabled)` but never explicitly saves store state.
- **Bug scenario:** User enables auto-update, app appears to accept change, but after restart setting falls back to old value (depending on store plugin persistence semantics).
- **How to fix:** Call `await store.save()` after setting the value, and surface save failures to the UI.

## 10) Import data flow does not refresh frontend state
- **Location:** `src/pages/settings/DataSetting.vue`
- **What is wrong:** After successful `import_data`, UI only shows `alert` and does not refresh media/tag stores.
- **Bug scenario:** User imports backup, sees success message, but current screen still shows stale old data until manual reload/navigation.
- **How to fix:** Trigger relevant store reload actions (e.g. media and tags) immediately after successful import.

## 11) Delete-file error handling loses error details
- **Location:** `src/component/media_page/FileRow.vue` (`deleteFile`)
- **What is wrong:** `.catch((e) => toast.error('Error deleting file:', e))` passes error as second argument to toast options slot, so actual error detail is not displayed.
- **Bug scenario:** File deletion fails (permission/path issue), but user only gets a generic message without reason, making troubleshooting hard.
- **How to fix:** Format the message explicitly (e.g. ``toast.error(`Error deleting file: ${String(e)}`)``).

## 12) Sync progress can misreport inserted count
- **Location:** `src-tauri/src/lib.rs` (`sync_files`)
- **What is wrong:** Progress increments with `chunk.len()` whenever `insert_medias` succeeds, even if internal dedupe/merge means fewer new rows are actually inserted.
- **Bug scenario:** UI shows "inserted 100/100" while DB may have inserted fewer rows due to duplicates. Users trust wrong progress metrics.
- **How to fix:** Make DB layer return inserted/new/updated counts and emit those real numbers in progress events.

---

## Suggested execution order
1. Fix #1 and #2 first (functional correctness + crash prevention).
2. Fix #4, #6, and #11 next (frontend correctness and operator visibility).
3. Fix #7, #8, and #9 for better input robustness and setting reliability.
4. Improve #3, #5, #10, and #12 for reliability, observability, and user-facing UX.
