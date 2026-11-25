# Open ADR items

Every action item in `docs/adr/` that is not done, as of 2026-11-22. **Partial** means some of it is built; the note says what's missing. Tick the item in its ADR (with a note on what delivered it) and remove it here in the same change.

Order of work agreed on 2026-11-22: ADR-0022's second set (done 2026-11-22) → ADR-0024 (copy and paste of cells, then tabs, then agents across documents) → the ADR-0014 pilot → the recent-version time-travel measurement (ADR-0021 item 5) → the GPU investigation → everything else.

## Next up

| ADR | Item | Status | What's left |
| --- | --- | --- | --- |
| 0024 | 5. (rest) Agent-panel conversations listed per tab (the panel shows the running one and notes its tab) | **Partial** | |
| 0014 | 2. `cruncha-templates`: the format, validation, preview, the test runner | Partial | Done: format, validation, test runner (calcs and Python). Left: preview, Rust cells in tests, CI |
| 0014 | 7. Convert Lacewing Lab section by section; the gap templates (uncertainty, atmosphere and moist air, chemistry) | Started | Mechanical first, a group per change: Stress and failure, Strain measurement, Fastening and Springs done (61 templates); next Pressure vessels. Research 02 §5 tracks it |
| 0021 | 5. Recent-version time travel: tombstone expiry (needs its own ADR) against checkpointing more often, on a quiet machine | Not started | |

## Later

| ADR | Item | Status | What's left |
| --- | --- | --- | --- |
| 0001 | 2. Contribution policy (DCO or a CLA) before the first outside pull request | Not started | |
| 0001 | 4. Verify the licence wording of Maxima, Giac, Octave and libqalculate before bundling any | Not started | Only if one is bundled |
| 0002 | 2. Port the unit table (defects fixed) and merge it with Numbat's units | **Partial** | Ported and corrected; the Numbat merge is not done |
| 0002 | 3. Hurmet `.md` importer | Not started | ADR-0007's import rule depends on it |
| 0003 | 3. `results/`, `data/<name>` tables; `assets/` | **Partial** | `results/` done (ADR-0018); `data/<name>` and `assets/` not |
| 0003 | 4. Text-mirror export/import; `.crunchaz` zip | **Partial** | The mirror exists for agents; file export, import and the zip don't |
| 0004 | 1. Spike S2: graph over calc and Python cells, rename invalidation, cancellation | **Partial** | Built in the engine; no write-up and no rename-invalidation test |
| 0004 | 2. Section scopes, after user testing | Not started | |
| 0005 | 3. User environments with extra packages; the lockfile stored in the document | Not started | |
| 0005 | 4. Column units for Polars frames; calling Python functions from calcs; SymPy | Not started | |
| 0006 | 3. Calling Rust functions from calcs; crates beyond the prelude; data frames | Not started | |
| 0007 | 1. Always unit-aware in `cruncha-eval`; the import-report rule | **Partial** | The engine is always unit-aware; the import rule waits for the importer |
| 0010 | 4. Python `read_doc` helper and the DuckDB recipe in the docs | Not started | |
| 0011 | 4. `data/*` tables follow the retention policy; pins carry their versions | Not started | Needs `data/*` tables (0003 item 3) |
| 0013 | 3. The `read_doc` helper and DuckDB recipe resolve `source_hash` | Not started | With 0010 item 4 |
| 0013 | 4. Revisit the deletion-vector layout when delta-rs writes them | Not started | Waits on delta-rs |
| 0015 | 4. User report templates (ADR-0014) and a report preview in the app | Not started | |
| 0021 | 4. Remove checkpoints older than the newest few (disk only) | Not started | |
| 0025 | (deferred) ∞ as a number; assigning elements inside a function | Not started | Revisit when a template can't be written without them |
