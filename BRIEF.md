# Brief

Stage 1. The person's own words come first; the rest was written by the agent from them. `C1 Frame the problem and set the rules`, `B1 Establish context`.

## The description, word for word (2026-10-13)

The person asked that nothing in the game or its human-facing text name the source game. Three names are replaced in square brackets below; everything else is as written. (SECOND-ORDER R1.2)

> Use sgilson7.github.io builder setting, which can be found one directory up. Use the shared technical setting within to create the following game for me: 3d game, based on the [name of the source game removed] logical adventure level where you have to make pizzas for a tree man, and he gives you back vague hints you have to use to figure out what is the correct toppings for his pizza. I want the [source game] type characters to be little [masked, robed character from another game] looking fellas, and the pizza tree man is instead a drummer who plays a lil riff based on whether he liked it or not, and then gives you some vague hint using radical cool guy prose. i want the game to have 10 levels of increasing difficulty and complexity, to the number of tree people analogues you need to feed in parallel, also the number of toppings and layers available. there should always be some logical way to solve every problem based on the inputs and outputs from the player and tree people respectively, so a solver that always wins based on feedback is necessary for testing. ok, build out the mvp, I give you explicit approval to create a public github repo and pages deployment, and i give explicit approval to update my website with the game once the mvp is finished. now build out the mvp, dont stop until the mvp is deployed. there  should be no explicit reference to [the source game] or the source material in the game and human facing strings to avoid potential licensing issues

## The tool

**Backbeat Pizza** (name chosen by the agent; SECOND-ORDER R1.1): a single-player 3D deduction game in a desktop or phone browser. A crew of small robed, masked cooks (the **Masklings**) builds pizzas for one to four **drummers** on a stage. Each drummer has a secret order. When a pizza is served, the drummer plays a short riff that says whether he liked it, then gives a hint in surfer-drummer slang. The player works out every order from the hints before the drummers run out of patience.

## Done means

- The page loads from `https://sgilson7.github.io/backbeat-pizza/` with no off-origin request.
- The stage is drawn in 3D (WebGL 2): drummers behind drum kits, the Masklings, the counter and the pizza being built.
- A served pizza is carried to the chosen drummer; he plays a riff (one kind for a perfect pizza, one for "close, needs more", one for "something on here is wrong") and a hint appears in a speech bubble and in that drummer's log.
- Ten levels. Difficulty grows in the number of drummers served in parallel (one to four), the number of toppings, the number of layers (toppings; then sauce; then crust), and how vague the hints are (names an item; gives counts; says only which way).
- Every level is solvable by reasoning from the hints: a solver in the core that sees only the hints wins every possible order of every level within that level's patience. A core test checks every possible order.
- No reader-visible string and no file in the repository's public pages names the original game, its characters or its publisher.
- The game is listed on Sam's website.

## Decisions that stay with the person

- The name, the drummers' names and every hint line (listed in SECOND-ORDER for reading).
- Every number that tunes the feel: patience per level, riff tempos.

## Accessibility

A verdict is carried by a symbol and words as well as colour, and by the riff. Every ingredient has a name and a distinct shape on its button. Everything is played with buttons, so mouse, touch and keyboard (Tab, Enter) all work.

## Style

Bright toon look: flat colour bands and a dark ink outline, like Sam's other Builder games. No external art; every shape is drawn by the core.

## Words

See `GLOSSARY.md`.

## Autonomy

Sam approved in advance (2026-10-13): the plan, a public GitHub repository, GitHub Pages deployment, and updating the website with the game once the MVP is finished, and asked the agent not to stop until the MVP is deployed. Where this brief is silent, the agent decides, records the decision in `SECOND-ORDER.md`, and continues. The agent may not loosen a Builder requirement or a boundary test, and may not add a server, an account, analytics or any off-origin request. The agent may create a public repository named `backbeat-pizza` and publish it once `./scripts/check.sh` passes.

## Allowed core dependencies

`serde`, `serde_json` (reference default).
