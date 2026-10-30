# Patent sweep: capant 0.0.x @ ab26b3c (library code as of 0.0.7)

Date: 2026-10-29; Run by: capant lane, Sonnet 5.5 (the lane ran the searches and read the claims itself, serially, with `tools/patent-claims.py`); Method: AI-assisted search and read; **NOT legal advice; NOT a freedom-to-operate opinion.**

**Read this first.** One pass. Each idea got one query (a few ideas share one), all through a web search restricted to patents.google.com. No USPTO, Espacenet or WIPO query was run. Search results are a summariser's text and were used only to choose what to read; verdicts rest on the claim text from `patent-claims.py` (machine-converted; Google's status and expiry are estimates, not the official record). Nine patents were read; the neighbours listed under "Not read" were not. "Outside as read" means: on the independent claims as read, at least one required element is absent from what capant does. It is not "clear", and a patent that was never found is not excluded by this record. Covers the version line 0.0.x (a library only: no network, and it executes nothing except a caller-supplied `Prober`). `src/` last changed in 0.0.9 (a doctest hook for the README); the engine and file format are as of 0.0.7.

## Ideas swept
1. A provider's capability counts as `yes` only with recorded probe evidence; no row, a `yes` without a probe, or a row from another environment is `unverified`, never `yes` (three states: yes / no / unverified).
2. Selection by NEEDS: required capability ids (all must be `yes`) plus preferred ids in order of importance; providers returned ranked, each with a per-need explanation; no "closest match": only fully satisfying providers may be opened, and a rejected one is reported with each unmet id and why.
3. Staleness: a `yes` older than a window, or recorded by a different tool version than the current one, degrades to `unverified`; a row that cannot show its age is stale when a window is set.
4. Combination across rows and surfaces: `no` beats `yes` beats `unverified`; the most actionable unverified reason is reported.
5. A plain-text, tab-separated matrix file with a canonical writer; `check` compares the committed file with a freshly generated one, ignoring only the probe timestamp, so CI can fail when the file drifts.
6. Generation by running caller-supplied probes per provider and capability, carrying over committed rows that cannot be re-probed on this machine.
7. Carried rows: rows copied from the committed file are marked inherited, count only with the consumer's consent, and are listed for review without failing `check`.
8. A probe whose own known-bad control fails reports `unverified(no control)` instead of an answer.
9. Capability ids for hardware or protocol targets compared by exact string equality, with no ordering or implication between ids (`docs/KISS.md`).

## Queries (all: Google Patents via web search restricted to patents.google.com, 2026-10-29)
- Q1 (ideas 1, 2): `patent capability matrix software component selection requirements evidence of tested capability provider ranking`
- Q2 (idea 2): `patent selecting service provider matching required capabilities ordered preferences unmet requirements explanation`
- Q3 (ideas 1, 3): `patent capability verified by probe test result expires stale age threshold tool version feature support determination`
- Q4 (idea 5): `patent compatibility matrix file committed regenerated automatically continuous integration fails when differs from generated`
- Q5 (ideas 5, 6): `patent device capability discovery automated probing feature detection result recorded per environment hardware software configuration`
- Q6 (ideas 1, 9): `patent unknown capability treated as unsupported tri-state supported unsupported unverified negotiation capabilities client server`
- Q7 (ideas 6, 7): `patent test result recorded from different environment not trusted until reverified inherited result carried over previous run marked for review`
- Q8 (idea 8): `patent known-bad control test must fail to validate probe positive control negative control automated feature test validity`
- Q9 (idea 5): `patent generating machine-readable support matrix from automated tests for plurality of implementations backends text file canonical order`

## Hits (independent claims read from `patent-claims.py`)
| # | patent | title | assignee | status (Google) | idea | claims read | overlap | our difference | verdict |
|---|---|---|---|---|---|---|---|---|---|
| 1 | US7684962B2 | Hardware/software capability rating system | Microsoft | expired | 1, 2 | 1 and 11 (the only independents) | compares tested features and performance results against required features and performance criteria | claim 1 needs an inventory module, a performance testing module and a standard integer-valued capability rating level; claim 11 needs a rating level for a computer system from that comparison. capant has no performance results and no rating level; it answers yes / no / unverified per capability | outside as read |
| 2 | US8185536B2 | Rank-order service providers based on desired service properties | HP / Conundrum IP | active | 2 | 1, 13, 19 | ranks candidate providers against requirements | every independent claim needs singular value decomposition over term and document matrices | outside as read |
| 3 | US20170220966A1 | Method and system for on-demand customized services | Operr | expired | 2 | 1 in full; 59 (its first 1,500 characters) | precludes incompatible providers; mandatory preferences | claim 1 needs a customer request, a blacklist, a matrix of preferences and providers, display of availability, current location, ETA and a route-familiarity level, then dispatch and route tracking; claim 59 adds location and clock hardware. capant is a library with none of these | outside as read |
| 4 | US9215150B2 | Feature set negotiation framework | SAP | active | 1, 6 | 1, 7, 13 (opening of each, about 1,800 characters) | a response carries the feature indicators a server supports; an excluded feature is detected | needs a service request from a first server to a second server carrying feature indicators, a response with modified data and a second set of indicators, differing software versions of the two servers, and a notification; capant exchanges nothing over a network | outside as read |
| 5 | US20040101125A1 | Capability negotiation in a telecommunications network | individual | expired | 2 | 1, 11, 12, 13, 14, 21, 25, 26, 27, 34 | an originating point sends a prioritised list of capability preferences and the other side accepts one | every independent claim is on signalling points, media gateways, call control or codec negotiation in a telecommunications system | outside as read |
| 6 | US20120102462A1 | Parallel test execution | Microsoft | abandoned | 6, 7 | 1, 10, 18 | results from an auxiliary environment are taken from a shared data structure instead of re-running | claim 1 needs tests executed in parallel within a test environment; claims 10 and 18 need a primary and an auxiliary test environment and a shared results data structure. capant copies committed rows and marks them inherited, and runs nothing in an auxiliary environment | outside as read |
| 7 | US9588875B2 | Probationary software tests | IBM | expired | 3, 7 | 1 (the only independent) | marks a test as less reliable by recency and presents its results differently | needs a continuous integration environment, a promoted status from consistent successful runs over builds, and distinct presentation by status; capant has no promoted or probationary state | outside as read |
| 8 | US8151247B2 | Test data management | SAP | active | 8 | 1, 9, 13 | a test system with an expected-error check (dependent claim 15 only) | the independents need posted business-process transaction data converted into test data by semantic mappings | outside as read |
| 9 | US20120191404A1 | Method and system for testing configuration of environments | IBM | expired | 6 | 1 (the only independent in the text read; the output was cut after claim 7) | runs tests from a probe and logs results | needs a probe for connection to a network, a configuration analyser tool launched from it, and externally stored tests loaded at run time, before connecting to the network | outside as read |

## Not read (no verdict)
US8930167B2 and US20150094984A1 (family of US7684962B2; the continuations may claim differently); US6986125B2; US20050166094A1 / US7490319B2 / JP2007535723A (test matrices); EP2040212A1 (preferences over non-functional properties in service selection; a non-US document); US20070050481A1; US9043219; US8924573 / US9246949 (secure capability negotiation); US7536393; US20040249907A1; US9515883B2; US8244924B2; EP3945527A1 / US8112232B2 / US8155886B2 / EP2118796A2 (laboratory autoverification); US20070117637A1; US7020803B2; EP0557628B1; US8072901B1. Each was seen only as a search result. Their claims are unread and carry no verdict. The remaining dependent claims of the nine read patents were mostly not read.

## Needs review (to the PM)
None as read. The two nearest readings: US20040101125A1 claim 1 (a prioritised list of capability preferences, but only at a telecommunications Call Control level) and US20120102462A1 (results taken from another environment, but by executing tests in two environments). If capant ever sends preferences or results between two processes, re-read both.

## Limitations
- Ideas 4, 5 and 8 (no-beats-yes-beats-unverified, the committed matrix file regenerated and compared in CI, the known-bad control) returned no relevant patent in one query each. That is an absence from one search with no positive control. It does not show that no patent exists. USPTO full text, Espacenet and WIPO were not searched.
- Nine claim sets were read out of many results. The unread list is longer than the read list.
- Google Patents status and expiry are its own estimates, not legal expiry.
- AI-assisted reading of machine-converted claim text. Check a quote against the official record before relying on it.

## Addendum: capant 0.1.0 (2026-10-30)

The sweep above was not re-run. Between `ab26b3c` and `e75eaa5` (0.1.0, published as `v0.1.0`) the `src/` diff is 11 added `#[non_exhaustive]` attributes on result types, `Role` and `Outcome`. No engine logic, file format or dependency changed, so none of the verdicts above depend on a changed behaviour. This is a statement about the diff, not a new search; the limitations below apply unchanged.
