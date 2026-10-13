# Plan

Stage 2, approved in advance (BRIEF.md, Autonomy).

## The rule, in one paragraph

Each drummer holds an order: one option per pick-one layer (crust, sauce) and between `min` and `max` toppings. A served pizza is a **miss** if anything on it is wrong (a wrong layer option or a topping not in the order), else **close** if a topping of the order is missing, else **perfect**. Named hints add the number of wrong things and the first of them (layers before toppings, menu order), or the number missing; counted hints add the numbers only; vague hints add nothing. Each clarity says everything the next one says, so the solver's worst case never grows as hints get clearer (`named_hints_need_no_more_pizzas_than_vague_ones`).

## Solvable by construction

`solver.rs` keeps the orders consistent with every hint and serves the pizza minimising the largest group of orders that would give the same hint (ties: a pizza that could itself be the order, then menu order). Serving a still-possible order always rules out at least that order when it is wrong, so the solver always ends. `levels.rs` builds the solver's whole decision tree for each level, checks every order is reached within the tree's depth, and checks `drummers × depth ≤ patience`. Patience is about 1.35 to 1.6 times the solver's need, so a person reasoning well but not perfectly can still finish.

## Ten levels

Difficulty grows along four axes, one or two at a time: drummers (1 → 4), layers (none → sauce → crust and sauce), toppings (3 → 7), and clarity (named → counted → vague). A test checks the solver's need never falls from one level to the next.

## The screen

- A WebGL 2 stage in the top half: drummers at kits on risers, a speaker stack each side, a curtain with string lights, the counter with the pizza being built, two cook Masklings and one runner Maskling per drummer.
- A serve: the runner walks the pizza up (1.1 s), the drummer bites (0.6 s), plays his riff (the riff's steps × his `step_ms`), and the hint appears in a speech bubble over him and in his log. His sticks follow the riff's hits; drums squash and cymbals wobble on each hit; the speakers thump on each kick.
- The kitchen below: a button per layer option and per topping (shape icon plus name), Clear, and a Serve button per drummer. The order log lists every pizza each drummer tasted with its verdict, hint and a Rebuild button.
- Title (how to play, ten level buttons), level card (rules built by the core from the level), cleared, lost (new orders on retry), finale.

## Architecture

Builder as given. Core: `puzzle.rs`, `solver.rs`, `levels.rs`, `content.rs`, `game.rs`, `scene.rs`, `render.rs`, plus `fx.rs`, `geom.rs`, `mesh.rs`, `rng.rs` from Gatecrasher. Shim: one `Game` class whose methods each make one core call. Page: `app.js` (screens from `ui()`), `gl.js` (draws the core's instances), `sound.js` (synthesizes the drum hits the core names).

## Defaults kept, and changed

All reference defaults kept, including D17 (no browser storage: every level is open from the title screen, so nothing needs saving). D18 does not apply: there is no 2D art, so the art pipeline was removed from the copy of the template.
