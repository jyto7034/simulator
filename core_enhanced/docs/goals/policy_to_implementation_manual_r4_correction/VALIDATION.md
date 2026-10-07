# R4 Manual Validation

Evidence ID: `PIP-MANUAL-R4-VALIDATION-20260813-01`  
Candidate revision: `PIP-MANUAL-2026-08-13-R4`  
Candidate SHA-256: `a310ed3d465f4cf94b753e3bcbfe9e3d1dc259bdae6f7465c6fbdd394fc4983d`  
Candidate size: 1,118 lines / 9,006 words

## Results

| Check | Executed | Result |
| --- | ---: | --- |
| START/END markers | 20 | pass — 10 START, 10 END |
| Required local-contract headings | 100 | pass — 10 headings in each of 10 partitions |
| Routing hints vs marker ranges | 10 | pass — PIP-00 `39–209`, PIP-10 `213–279`, PIP-20 `283–358`, PIP-30 `362–432`, PIP-40 `436–561`, PIP-50 `565–663`, PIP-60 `667–765`, PIP-70 `769–831`, PIP-80 `835–943`, PIP-90 `947–1118` |
| PIP-80 event IDs | 34 | pass — no duplicates |
| Whitespace checks | 2 | pass — no trailing blanks or tabs |
| Markdown table inline-code pipe check | 1 full-file pass | pass — no pipe remains inside an inline-code table cell |
| State-machine assertions | 15 | pass |

The 15 state assertions cover the Single same-pass Campaign gate and `CC=N/A`, Master-only PIP-70, exclusive EV-07A–E attribution guards, EV-10C1 previous-root recovery, EV-10C2 exact-candidate idempotent recovery, EV-10C3 missing-intent/unknown-root correction, and the PIP-50/PIP-60 mutation-owner split.

No runtime, data, server, client, release, or deployment validation applies to this governance-only single-file correction.
