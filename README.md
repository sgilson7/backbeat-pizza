# Backbeat Pizza

A 3D deduction game that runs in the browser. A band of drummers each want one secret pizza. Your crew of Masklings, small robed cooks in white masks, builds pizzas at the counter and carries them up to the stage. Each drummer tastes, plays a riff that tells you how he felt about it, and drops a hint in surfer-drummer slang. Work out every order before the band runs out of patience.

**Play:** https://sgilson7.github.io/backbeat-pizza/

## How a hint works

A served pizza gets one of three verdicts:

- **★ Perfect**: it is his order.
- **▲ Close**: nothing on it is wrong, but a topping of his order is missing.
- **✕ Miss**: something on it is wrong: a crust, a sauce or a topping he does not want.

Early levels say more than the verdict. Named hints count the wrong things and name the first one, or count the missing toppings. Counted hints give the counts alone. Vague hints give the verdict and nothing else. Later levels rate each part on its own instead: every layer right or wrong, and the toppings perfect, close or a miss.

## The levels

| Level | Drummers | Layers | Toppings | Toppings per order | Hints | Patience | Solver needs at most |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Soundcheck | 1 | none | 3 | 1–2 | named | 5 | 3 |
| Garage Gig | 1 | none | 4 | 1–2 | named | 6 | 3 |
| Battle of the Bands | 2 | none | 4 | 1–2 | named | 10 | 6 |
| Basement Show | 2 | none | 5 | 1–3 | counted | 14 | 10 |
| Open Mic Night | 2 | sauce | 5 | 1–3 | counted | 15 | 10 |
| Power Trio | 3 | sauce | 5 | 1–3 | counted | 21 | 15 |
| Fog Machine | 3 | sauce | 6 | 1–3 | rated | 25 | 18 |
| Late Show | 3 | crust, sauce | 6 | 1–3 | rated | 26 | 18 |
| Festival Main Stage | 3 | crust, sauce | 7 | 1–3 | rated | 30 | 21 |
| Grand Finale | 4 | crust, sauce | 7 | 1–4 | rated | 40 | 28 |

Patience is shared by the whole band, and you can serve one drummer while another is still tasting.

## Every level can be solved

The core holds a solver that sees only the hints. It keeps every order that agrees with every hint so far and serves the pizza whose worst hint leaves the fewest orders standing. A core test plays it against every possible order of every level and checks that it always finishes within the level's patience; the last column above is its worst case. Another test plays all ten levels through the same actions the page's buttons send.

## How it is built

Backbeat Pizza is a [Builder](https://github.com/sgilson7/builder-setting) project, with the 3D approach of Gatecrasher: Q16.16 fixed-point maths and no floats in the core, instanced WebGL 2 draws, an inverted-hull ink outline and banded light.

- **`crates/core`** holds every rule: the verdicts and hints (`puzzle.rs`), the solver (`solver.rs`), the level check (`levels.rs`), the game and its screens (`game.rs`), and the stage with its animated drummers and Masklings (`scene.rs`). No input or output, no floats, no clock.
- **`crates/shim`** moves values across the WebAssembly boundary and decides nothing.
- **`web/`** is a static page with a WebGL 2 canvas. It draws what the core returns and plays the drum sounds the core names.
- **`data/`** holds the menu, levels, band, hint lines, riffs (as drum-machine grids), words, colours, sound recipes and shaders.

```bash
./scripts/setup.sh   # once: Playwright and three browser engines
./scripts/check.sh   # fmt, clippy, the core tests, the wasm build and audit, the browser check
python3 scripts/serve.py 8000
```

A push to `main` runs the same check in Chromium and deploys `dist/` to GitHub Pages if it passes.

`BRIEF.md` holds the request, `PLAN.md` the decisions, `SECOND-ORDER.md` what was found while building, and `HANDOFF.md` where the work stands.
