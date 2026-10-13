# Handoff

13 October 2026. Where Backbeat Pizza stands.

## What exists

| What | Where |
| --- | --- |
| Repository | https://github.com/sgilson7/backbeat-pizza (public, MIT) |
| The game | https://sgilson7.github.io/backbeat-pizza/ (Pages, deployed by `.github/workflows/deploy.yml` after the check passes) |
| Website page | https://sgilson7.github.io/projects/backbeat-pizza/, from `~/Documents/website` commit `729dc39` (record in `content/projects.yaml`, `content/projects/backbeat-pizza.md`, tile and hover clip, `/backbeat-pizza/` in `OFF_BUILD_ROUTES`) |

## Done, against the brief

- A 3D stage in WebGL 2: drummers at drum kits on risers, speakers, a curtain with string lights, the counter with the pizza being built, and robed, masked Masklings who carry each pizza up.
- A served pizza is carried over, tasted, and answered with a riff (perfect, close and miss riffs, drum-machine grids in `data/riffs.json`, synthesized drums) and a hint in surfer-drummer slang in a speech bubble and the drummer's log.
- Ten levels: one to four drummers in parallel, three to seven toppings, toppings only, then sauce, then crust and sauce, and hints going from named to counted to vague.
- A solver that sees only the hints. `levels.rs` plays it against every possible order of every level and checks it always finishes within the level's patience; `game.rs` plays all ten levels through the page's own actions.
- No file in the repository names the source game (`source_names.rs`).

## Commands run, and results

| Command | Result |
| --- | --- |
| `./scripts/check.sh` (local) | BUILDER CHECK PASSED: fmt, clippy, 97 tests, release wasm (about 490 KB), import audit, browser check in Chromium, Firefox and WebKit |
| `python3 scripts/break_tests.py` | 14 of 14 planted breaks caught (after strengthening one test, SECOND-ORDER R1.8) |
| GitHub Actions on the first push | check passed (Chromium) and deployed |
| `BUILDER_ORIGIN=https://sgilson7.github.io/backbeat-pizza tests/browser/check.py` | BROWSER CHECK PASSED in three engines against the live site |
| website `make check`, `make packet`, `make gate`, `publish.sh`, `make gate-live` | no findings; all packet checks pass; gate passed locally and live (build c451f47b). Built from a clean git worktree, so another session's uncommitted edit to `content/projects/packet-alley.md` stayed unpublished and untouched |

## Next

1. Play it by hand: the riffs' sound, the feel of the timings, and whether patience is fair (SECOND-ORDER R1.5, R1.6).
2. Read the hint lines and names (SECOND-ORDER R1.1, strings table).
3. The website's source repository (`scholar-site`) is ahead of its origin by two commits, the earlier one from another session; it was not pushed.
