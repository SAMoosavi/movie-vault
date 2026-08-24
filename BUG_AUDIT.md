# Bug & Error-Handling Audit

This list captures remaining open bugs and high-impact improvements after fixing previous #1, #2, and #4.

## 1) File watch callback can trigger sync storms

- **Location:** `src/App.vue:121`, `src/App.vue:125`
- **What is wrong:** On one FS event, code loops all changed paths and runs full `sync_files(...)` per path with no dedupe or in-flight guard.
- **Bug scenario:** Copying/extracting many files emits many events and starts repeated expensive sync runs, slowing app responsiveness and increasing failure probability.
- **How to fix:** Normalize/dedupe directories per event batch, debounce sync calls, and guard with single-flight queue/mutex.

## 2) IMDb search can show stale results due to out-of-order responses

- **Location:** `src/component/media_page/SearchMediaImdb.vue:114`, `src/component/add_media_page/AddMediaManual.vue:135`
- **What is wrong:** Debounce exists, but old network requests are not canceled/ignored.
- **Bug scenario:** User types quickly (`bat` -> `batman`); slow response for old query arrives late and overwrites new results.
- **How to fix:** Add request token/version check or abort previous request (`AbortController`) and ignore stale responses.

## 3) Media scanning duplicate check is O(N\*M)

- **Location:** `src-tauri/src/media_scanner.rs:32`
- **What is wrong:** For each discovered file, code checks all DB file rows with `files.iter().all(...)`.
- **Improvement scenario:** Large libraries suffer avoidable scan slowdowns as DB file count grows.
- **How to fix:** Build a `HashSet<String>` of DB paths once and check membership in O(1).

---

## Suggested execution order

1. Fix #1 first (sync burst protection).
2. Fix #2 next (search response ordering).
3. Fix #3 for large-library scan performance.
