# Audit reader attempts

1. Attribution reader initially expected `outputs` in historical fresh_001 registry; actual schema uses `files`. It failed with KeyError before writing a review result. Corrected only this audit-local reader. No runtime execution or target change.

2. Attribution reader initially treated every file newly included in the broader final archive as newly created by R04. Four tool files were already present: three independently match the original R03 baseline and one the original R03 audit target. The reader now verifies this supplemental prior identity before excluding them from R04 changes. Patch headers were inspected and matched to the actual before/after and /dev/null format before comparison. No source or owner evidence changed.

3. Adapted evidence reader assumed the older command key; current206–213 use command_args. Output-reader path substitution also changed its intended fresh_002 output to nonexistent fresh_003 and failed at write (no file was created there). Corrected both audit-local reader assumptions; all checks recomputed, owner bytes untouched.

4. Historical reader mechanics used a mean-difference permutation statistic. The actual unchanged final_performance.rs:765 guard uses pairwise U (twice), producing concurrent p=0.999939191243539. The initial146-check result with one reader mismatch is retained as evidence_review_reader_initial.json. Corrected the audit reader to reproduce the actual original statistic; no threshold, sample or runtime code changed.
