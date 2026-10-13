# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

| # | Kind | Row |
| --- | --- | --- |
| R1.1 | worklist | The name **Backbeat Pizza**, the cooks' name **Masklings**, the drummers' names and the level names were chosen by the agent. All are in `data/` and can be changed there. *person* |
| R1.2 | divergence | Builder pastes the description word for word into BRIEF.md. The description names the source game and a character from another game, and asks that nothing human-facing name them. Those three names are replaced in square brackets in BRIEF.md, and `crates/core/tests/source_names.rs` fails if any text file in the repository names the source game, its characters or its publisher (the list is kept in ROT13 so the test does not name them). The local working folder keeps the name it was started under, which names the source game; it is not part of the repository, which is `backbeat-pizza`. *done* |
| R1.3 | finding | With named hints that gave only the first wrong item, the solver needed more pizzas than with counted hints (8 against 5 on a sauce-and-five-toppings menu): naming one item says less than counting all of them. Named hints now give the count as well as the first item, so each clarity says strictly more than the next. *done* |
| R1.4 | divergence | The source game punishes each wrong guess. Here a wrong pizza costs one patience from a pool shared by the band, sized from the solver's worst case. Retrying a lost level deals new orders. *done* |
| R1.5 | finding | The solver's worst case for the final level (four drummers, crust, sauce, seven toppings, vague) is 15 pizzas a drummer, 60 in all; patience is 80. A level like that takes several minutes at about four seconds a serve. Serving several drummers at once keeps it moving. *open: the person's first play decides whether it is too long* |
| R1.6 | worklist | Every patience number and the serve timings (`WALK_MS`, `TASTE_MS`, `TAIL_MS` in `game.rs`, `step_ms` in `band.json`) are guesses. *person* |
| R1.7 | finding | The glossary lint caught "stage" in reader strings while Level's `Not:` line banned it. The game has a literal stage, so Stage got its own entry and Level's line lost the word. *done* |
| R1.8 | finding | A planted break (`any` for `all` in the hint filter) survived the first version of `hints_rule_out_every_order_that_disagrees`, which used one hint. The test now uses two. 12 of 12 breaks caught. *done* |
| R1.9 | divergence | No 2D art: the template's TikZ art pipeline (`tools/art`, `styles/`, `tests/art_check.py`) was removed. Ingredient icons are CSS shapes in the ingredient's colour. *done* |
| R1.10 | finding | `window.backbeat` exposes the core's `act` and `ui` so the browser check can ask for the solver's pizza (`solver.suggest`) and press the real serve button. A player could call it from the console; it gives nothing the buttons cannot, except the solver's choice. *open* |
| R1.11 | finding | The hint `{count}` is a numeral ("2 things on here are bogus"), filled by the core. One and many have separate lists so the grammar fits. *done* |

## Strings written by the agent

Every reader-visible string was written by the agent, for the person to read:

| File | What |
| --- | --- |
| `data/copy.json` | the title, how to play, buttons, the level card's rules, results |
| `data/hints.json` | every drummer line, by verdict, clarity and count |
| `data/levels.json` | the ten level names |
| `data/band.json` | the four drummers' names |
| `data/menu.json` | ingredient names |
| `GLOSSARY.md` | every definition |
