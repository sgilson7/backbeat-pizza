# Glossary

The source of truth for what each word in this tool means. Strings, tests, the plan and the README use a term in the sense given here. The page shows a definition when a reader hovers the term.

Write each entry as a `### Term` heading, then one or two plain sentences, then an optional `Not:` line listing words that must not be used for it. A number a rule decides is a `{placeholder}` the core fills, so the glossary cannot disagree with the rules. Keep entries in alphabetical order. `crates/core/tests/copy.rs` checks the format, the placeholders and the `Not:` words; `TONE.md` governs the prose.

<!--
An example entry. It sits inside this comment, so it does not count.

### Spout

Where slush comes out. Each line has up to {line.spouts} spouts.

Not: tap, dispenser, nozzle
-->

### Close

The verdict on a pizza with nothing wrong on it that still lacks a topping of the order. Shown as ▲.

Not: almost, warm

### Drummer

A member of the band on stage. Each drummer holds one secret order and plays a riff after tasting a pizza.

Not: customer, diner

### Hint

What a drummer says after his riff. A level's hints are named (they count the wrong things and name the first, or count the missing toppings), counted (they give the counts only), vague (the verdict only) or rated (each layer right or wrong, and the toppings perfect, close or miss on their own).

Not: clue, tip

### Layer

A part of the pizza with exactly one choice: crust or sauce. Later levels add layers.

### Level

One show: a set number of drummers, layers, toppings and patience. There are {levels.count} levels.

Not: round

### Maskling

One of the small robed, masked cooks who build pizzas at the counter and carry them to the drummers.

### Miss

The verdict on a pizza with at least one wrong thing on it: a crust, a sauce or a topping the order does not have. Shown as ✕.

Not: fail, wrong answer

### Order

The one pizza a drummer wants: a choice for each layer and between {level.min} and {level.max} toppings.

Not: recipe, request

### Patience

How many pizzas the band will taste in a level, shared by every drummer. Each serve uses one.

Not: lives, tries, turns

### Perfect

The verdict on a pizza that matches the order exactly. Shown as ★.

### Riff

The short drum pattern a drummer plays after tasting: one kind for perfect, one for close, one for a miss.

### Serve

Sending the pizza on the counter to one drummer. A drummer still tasting cannot be served again until he has given his hint.

### Stage

The platform the band plays on, drawn above the kitchen. A level's name may name a stage, as a gig would.

### Topping

Anything on a pizza besides its layers. An order may hold several.

