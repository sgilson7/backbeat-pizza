// The page draws what the core returns and decides nothing. It loads the data
// files, starts the core through the shim, and each display frame passes the
// clock in, plays the sounds the core names, redraws the screen when the
// core's screen version changes, and draws the stage. Every button sends a
// named action back through the core's one door, act().
import init, { Game, numbers } from "./pkg/tool_shim.js";
import { makeRenderer } from "./gl.js";
import { setup as setupSound, wake, toggleMute, play } from "./sound.js";

const el = (id) => document.getElementById(id);
const text = (r) => r.text();

function make(tag, className, content) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (content !== undefined) node.textContent = content;
  return node;
}

// An ingredient's little picture: its shape, in its colour.
function icon(item) {
  const i = make("span", `icon shape-${item.shape}`);
  i.setAttribute("aria-hidden", "true");
  if (item.css) i.style.setProperty("--c", item.css);
  return i;
}

function chip(item) {
  const c = make("span", "chip");
  c.append(icon(item), make("span", "", item.name));
  return c;
}

async function start() {
  const [menu, levels, band, hints, riffs, copyText, scene, masklings, soundsText, vertex, fragment, build] =
    await Promise.all([
      fetch("data/menu.json").then(text),
      fetch("data/levels.json").then(text),
      fetch("data/band.json").then(text),
      fetch("data/hints.json").then(text),
      fetch("data/riffs.json").then(text),
      fetch("data/copy.json").then(text),
      fetch("data/scene.json").then(text),
      fetch("data/masklings.json").then(text),
      fetch("data/sounds.json").then(text),
      fetch("data/shaders/scene.vert").then(text),
      fetch("data/shaders/scene.frag").then(text),
      fetch("build.txt").then(text),
    ]);
  const copy = JSON.parse(copyText);
  for (const node of document.querySelectorAll("[data-copy]")) node.textContent = copy[node.dataset.copy];
  document.title = copy.title;
  el("build").textContent = build.split("\n")[0];
  el("how").replaceChildren(...copy.how.map((line) => make("li", "", line)));
  setupSound(JSON.parse(soundsText));
  await init();
  const N = JSON.parse(numbers());

  let game;
  try {
    const seed = crypto.getRandomValues(new Uint32Array(1))[0];
    game = new Game(menu, levels, band, hints, riffs, copyText, scene, masklings, seed);
  } catch (e) {
    el("message").textContent = String(e);
    document.body.dataset.ready = "true";
    return;
  }

  const canvas = el("view");
  let renderer = null;
  try {
    renderer = makeRenderer(canvas, N, { vertex, fragment }, (id) => game.mesh(id));
  } catch (e) {
    el("message").textContent = String(e);
  }
  if (!renderer) el("message").textContent = el("message").textContent || copy.no_webgl;
  if (renderer) renderer.setUniforms(game.uniforms(), game.clear_color());

  let muted = false;
  const showSound = () => {
    el("sound").textContent = muted ? copy.sound_off : copy.sound_on;
  };
  showSound();
  el("sound").addEventListener("click", () => {
    wake();
    muted = toggleMute();
    showSound();
  });

  function act(name, value) {
    wake();
    el("message").textContent = game.act(name, value === undefined ? "" : value);
    draw();
  }
  const button = (label, name, value, className) => {
    const b = make("button", className, label);
    b.type = "button";
    b.addEventListener("click", () => act(name, value));
    return b;
  };
  const wire = (id, name) => el(id).addEventListener("click", () => act(name));
  wire("start", "level.start");
  wire("next", "level.next");
  wire("retry", "level.retry");
  wire("clear", "pizza.clear");
  for (const id of ["to-levels", "intro-levels", "won-levels", "lost-levels", "finale-levels"]) wire(id, "screen.title");

  // --- the screen, drawn from the core's description -------------------------
  let shown = -1;
  function draw() {
    const ui = JSON.parse(game.ui());
    shown = ui.version;
    document.body.dataset.mode = ui.mode;
    for (const id of ["title", "intro", "won", "lost", "finale"]) el(id).hidden = ui.mode !== id;
    el("play").hidden = ui.mode !== "play";
    el("bar-level").textContent = ui.mode === "title" ? "" : `${ui.level.label} · ${ui.level.name}`;
    el("bar-patience").hidden = !ui.play;
    el("level-list").replaceChildren(
      ...ui.levels.map((l) => {
        const b = button("", "level.choose", l.value, "level");
        b.append(make("span", "label", l.label), make("span", "name", l.name));
        return b;
      }),
    );
    el("intro-label").textContent = ui.level.label;
    el("intro-name").textContent = ui.level.name;
    el("intro-rules").replaceChildren(...ui.level.rules.map((r) => make("li", "", r)));
    const p = ui.play;
    if (!p) {
      el("bubbles").replaceChildren();
      return;
    }
    el("patience").textContent = p.patience;
    el("won-text").textContent = p.won_text;
    el("lost-text").textContent = p.lost_text;
    el("pizza").replaceChildren(...p.pizza.map(chip));
    el("layers").replaceChildren(
      ...p.layers.map((layer) => {
        const box = make("div", "layer");
        box.append(make("h2", "", layer.name));
        const row = make("div", "options");
        for (const o of layer.options) {
          const b = button("", "pizza.pick", o.value, "option");
          b.append(icon(o), make("span", "", o.name));
          b.setAttribute("aria-pressed", String(o.on));
          row.append(b);
        }
        box.append(row);
        return box;
      }),
    );
    el("toppings").replaceChildren(
      ...p.toppings.map((t) => {
        const b = button("", "pizza.top", t.value, "option");
        b.append(icon(t), make("span", "", t.name));
        b.setAttribute("aria-pressed", String(t.on));
        return b;
      }),
    );
    el("serve-buttons").replaceChildren(
      ...p.drummers.map((d) => {
        const b = button(d.serve_label, "serve", d.value, "serve");
        b.disabled = !d.can_serve;
        return b;
      }),
    );
    el("log-drummers").replaceChildren(
      ...p.drummers.map((d) => {
        const box = make("div", "drummer");
        box.dataset.fed = String(d.fed);
        const head = make("h3", "", d.name);
        if (d.status) head.append(make("span", "status", d.status));
        box.append(head);
        if (!d.log.length) box.append(make("p", "empty", copy.log_empty));
        const list = make("ol", "entries");
        for (const e of d.log) {
          const li = make("li", "entry");
          li.dataset.verdict = e.verdict;
          const chips = make("div", "chips");
          chips.append(...e.chips.map(chip));
          li.append(make("span", "verdict", e.verdict_label), chips, make("p", "line", e.line));
          li.append(button(copy.rebuild, "pizza.rebuild", e.rebuild, "rebuild"));
          list.append(li);
        }
        box.append(list);
        return box;
      }),
    );
    el("bubbles").replaceChildren(
      ...p.drummers
        .filter((d) => d.bubble && d.at)
        .map((d) => {
          const b = make("div", "bubble");
          b.dataset.verdict = d.bubble.verdict;
          b.style.left = d.at[0];
          b.style.top = d.at[1];
          b.style.width = p.bubble_width;
          b.append(make("strong", "", d.name));
          if (d.bubble.label) b.append(make("span", "verdict", d.bubble.label));
          b.append(make("p", "", d.bubble.line));
          return b;
        }),
    );
  }

  function resize() {
    const [w, h] = game.resize(canvas.clientWidth, canvas.clientHeight, window.devicePixelRatio || 1);
    if (renderer) renderer.resize(w, h);
  }
  new ResizeObserver(resize).observe(canvas);
  resize();

  function frame(now) {
    for (const s of JSON.parse(game.advance(now))) play(s);
    if (game.ui_version() !== shown) draw();
    if (renderer) renderer.frame(game.mats(), game.draws(), game.instances());
    requestAnimationFrame(frame);
  }

  // the browser check plays a level through the same door the buttons use
  window.backbeat = { act, ui: () => JSON.parse(game.ui()) };
  draw();
  document.body.dataset.ready = "true";
  requestAnimationFrame(frame);
}

start();
