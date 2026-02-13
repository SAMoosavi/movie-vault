# Bug & Error-Handling Audit

This list captures remaining open bugs and high-impact improvements after fixing previous #1 and #2.

## 1) Type filter radio group is wired incorrectly

- **Location:** `src/component/home_page/FilterMedias.vue:293`, `src/component/home_page/FilterMedias.vue:295`
- **What is wrong:** Type radios use `name="watch_list"` (same group as watch-list filter), and "All" checks `filters.type === null` while type uses `'all' | 'movie' | 'series'`.
- **Bug scenario:** Selecting type can interfere with watch-list radio state; default "All" type is not visually selected correctly.
- **How to fix:** Use a dedicated radio group name for type (e.g. `name="type"`) and check with `filters.type === 'all'`.

## 2) File watch callback can trigger sync storms

- **Location:** `src/App.vue:121`, `src/App.vue:125`
- **What is wrong:** On one FS event, code loops all changed paths and runs full `sync_files(...)` per path with no dedupe or in-flight guard.
- **Bug scenario:** Copying/extracting many files emits many events and starts repeated expensive sync runs, slowing app responsiveness and increasing failure probability.
- **How to fix:** Normalize/dedupe directories per event batch, debounce sync calls, and guard with single-flight queue/mutex.

## 3) IMDb search can show stale results due to out-of-order responses

- **Location:** `src/component/media_page/SearchMediaImdb.vue:114`, `src/component/add_media_page/AddMediaManual.vue:135`
- **What is wrong:** Debounce exists, but old network requests are not canceled/ignored.
- **Bug scenario:** User types quickly (`bat` -> `batman`); slow response for old query arrives late and overwrites new results.
- **How to fix:** Add request token/version check or abort previous request (`AbortController`) and ignore stale responses.

## 4) Multiple mutation actions lack local error handling

- **Location:** `src/component/media_page/ManageSection.vue:117`, `src/component/media_page/FilesSection.vue:112`, `src/pages/settings/TagSetting.vue:106`
- **What is wrong:** Several async UI actions call invokers without `try/catch`, so failures can surface as unhandled promise rejections and give poor user feedback.
- **Bug scenario:** Backend validation/network failure leaves controls appearing non-responsive with no actionable toast.
- **How to fix:** Wrap each action with `try/catch`, call `handleFrontendError(...)`, and add pending-state disable to prevent repeat submissions.

## 5) Media scanning duplicate check is O(N*M)

- **Location:** `src-tauri/src/media_scanner.rs:32`
- **What is wrong:** For each discovered file, code checks all DB file rows with `files.iter().all(...)`.
- **Improvement scenario:** Large libraries suffer avoidable scan slowdowns as DB file count grows.
- **How to fix:** Build a `HashSet<String>` of DB paths once and check membership in O(1).

---

## Suggested execution order

1. Fix #1 and #4 first (filter correctness + user-visible error handling).
2. Fix #2 and #3 next (sync burst protection + search response ordering).
3. Fix #5 for large-library scan performance.
