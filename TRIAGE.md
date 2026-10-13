# Triage

Stage 7. One row per request after the person has used the build: a bug, a change, or a new idea, in the person's own words. Rows are numbered T1, T2 and so on, and each commit that answers one names it ("Fix the timer reset (T4)"). `C4 Verify against hand-computed cases`.

Compare at least one result with a case worked out by hand. **Severity** is *blocks use*, *wrong*, or *wanted*. **Cost** is the agent's estimate. **Disposition** is *done*, *open*, *declined* (with the reason), or *try* (built on a branch and published as a TRY commit that says how to undo it).

| # | Request, in the person's words | Expected | What happened | Severity | Cost | Disposition |
| --- | --- | --- | --- | --- | --- | --- |
| T1 | "the drummers names are a lil weird, like rimshot rex sounds somewhat sexual which i dont like, so check all the lines in the game to ensure no innuendos or anything of the sort are anywhere" | No name or line with a second, adult reading | Two drummer names, four hint lines and three level names could be read that way | wrong | small | done: all four drummers renamed (Hi-Hat Hank, Snare Drum Stan, Cymbal Sal, Tom-Tom Theo); the lines and level names rewritten; the words added to TONE.md's lint, so `no_text_uses_a_phrase_the_tone_file_rules_out` fails if they come back. Sausage and stuffed crust kept as ordinary pizza words |
