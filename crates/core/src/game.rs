//! The game: which screen is up, the level being played, the pizza on the
//! counter, each drummer's secret order and log, and the serves in flight.
//!
//! The page passes its clock in (`advance`) and the player's actions in
//! (`act`), and reads back the screen (`ui`), the sounds to play, and the
//! scene to draw (`scene.rs`). Everything here is decided from those inputs
//! and the seed, so a test can play the game without a browser.

use serde_json::{json, Value};

use crate::content::{Content, Drum, Files, LevelDef, Riff};
use crate::fx::{self, int, ONE};
use crate::geom::{v3, V3};
use crate::mesh::{self, Mesh};
use crate::puzzle::{judge, Clarity, Detail, Feedback, Item, Pizza, Verdict};
use crate::render::{self, Camera};
use crate::rng::Rng;
use crate::solver;

// --- the feel: how long each part of a serve takes, in milliseconds ----------------

/// A Maskling's walk from the counter to a drummer, and back.
pub const WALK_MS: i64 = 1100;
/// The drummer's bite, before his riff.
pub const TASTE_MS: i64 = 600;
/// A pause after the riff's last step before the hint shows.
pub const TAIL_MS: i64 = 250;
/// How long the drummer shows his reaction after the hint.
pub const REACT_MS: i64 = 1400;
/// After the last order is cracked, before the level-cleared card.
pub const CHEER_MS: i64 = 1800;
/// A frame longer than this is counted as this (a tab put away and back).
pub const LONGEST_FRAME_MS: i64 = 250;

/// The distance between two drummers' seats, and the room either side.
pub const SEAT_SPACING: i32 = int(3) + ONE / 5;
const BAND_MARGIN: i32 = int(1) + ONE * 7 / 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Title,
    Intro,
    Play,
    Won,
    Lost,
    Finale,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Title => "title",
            Mode::Intro => "intro",
            Mode::Play => "play",
            Mode::Won => "won",
            Mode::Lost => "lost",
            Mode::Finale => "finale",
        }
    }
}

/// One pizza a drummer has tasted, and what he said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub pizza: Pizza,
    pub feedback: Feedback,
    pub line: String,
}

/// A serve in flight: carried over, tasted, judged by riff, then revealed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub pizza: Pizza,
    pub feedback: Feedback,
    pub line: String,
    /// when it was served, on the page's clock
    pub at: i64,
    pub riff: Riff,
    /// how many of the riff's hits have sounded
    pub played: usize,
    pub revealed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Waiting,
    Pleased,
    Hopeful,
    Grumpy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub order: Pizza,
    pub log: Vec<Entry>,
    pub fed: bool,
    pub run: Option<Run>,
    pub mood: Mood,
    /// when the mood began
    pub mood_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Play {
    pub seats: Vec<Seat>,
    pub left: u32,
    pub pizza: Pizza,
    /// when the last order was cracked, or patience ran out
    pub ended_at: Option<i64>,
}

pub struct Game {
    pub(crate) c: Content,
    rng: Rng,
    pub(crate) mode: Mode,
    pub(crate) level: usize,
    pub(crate) play: Option<Play>,
    /// the page's clock, in milliseconds since the game began
    pub(crate) clock: i64,
    page_clock: Option<i64>,
    size: (i32, i32),
    sounds: Vec<&'static str>,
    version: u32,
    frame: Option<(Vec<i32>, Vec<u32>)>,
}

/// The pieces of a drum kit the page can play, and two of its own.
pub const SOUNDS: [&str; 4] = ["serve", "win", "lose", "click"];

impl Game {
    pub fn new(files: &Files, seed: u32) -> Result<Game, String> {
        let c = Content::load(files)?;
        Ok(Game {
            c,
            rng: Rng::new(seed as u64),
            mode: Mode::Title,
            level: 0,
            play: None,
            clock: 0,
            page_clock: None,
            size: (1, 1),
            sounds: Vec::new(),
            version: 1,
            frame: None,
        })
    }

    pub fn level_def(&self) -> &LevelDef {
        &self.c.levels[self.level]
    }

    fn touch(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    pub fn ui_version(&self) -> u32 {
        self.version
    }

    // --- the clock --------------------------------------------------------------

    /// The page's clock in milliseconds. Returns the sounds to play now.
    pub fn advance(&mut self, page_ms: i64) -> Vec<&'static str> {
        let step = match self.page_clock {
            Some(p) => (page_ms - p).clamp(0, LONGEST_FRAME_MS),
            None => 0,
        };
        self.page_clock = Some(page_ms);
        self.clock += step;
        self.frame = None;
        self.tick();
        std::mem::take(&mut self.sounds)
    }

    fn tick(&mut self) {
        let now = self.clock;
        let Some(play) = self.play.as_mut() else {
            return;
        };
        if self.mode != Mode::Play {
            return;
        }
        let mut changed = false;
        for (d, seat) in play.seats.iter_mut().enumerate() {
            let step_ms = self.c.band[d].step_ms;
            let Some(run) = seat.run.as_mut() else {
                continue;
            };
            let t = now - run.at;
            let riff_from = WALK_MS + TASTE_MS;
            while run.played < run.riff.hits.len() {
                let (s, drum) = run.riff.hits[run.played];
                if t < riff_from + s as i64 * step_ms {
                    break;
                }
                self.sounds.push(drum.key());
                run.played += 1;
            }
            let reveal = riff_from + run.riff.steps as i64 * step_ms + TAIL_MS;
            if !run.revealed && t >= reveal {
                run.revealed = true;
                seat.log.push(Entry {
                    pizza: run.pizza.clone(),
                    feedback: run.feedback,
                    line: run.line.clone(),
                });
                seat.mood = match run.feedback.verdict {
                    Verdict::Perfect => Mood::Pleased,
                    Verdict::Close => Mood::Hopeful,
                    Verdict::Miss => Mood::Grumpy,
                };
                seat.mood_at = now;
                if run.feedback.verdict == Verdict::Perfect {
                    seat.fed = true;
                }
                changed = true;
            }
            if run.revealed && t >= reveal.max(2 * WALK_MS + TASTE_MS) {
                seat.run = None;
                changed = true;
            }
        }
        let idle = play.seats.iter().all(|s| s.run.is_none());
        let all_fed = play.seats.iter().all(|s| s.fed);
        if idle && play.ended_at.is_none() && (all_fed || play.left == 0) {
            play.ended_at = Some(now);
            changed = true;
        }
        if let Some(end) = play.ended_at {
            if now - end >= CHEER_MS {
                let last = self.level + 1 == self.c.levels.len();
                self.mode = match (all_fed, last) {
                    (true, true) => Mode::Finale,
                    (true, false) => Mode::Won,
                    _ => Mode::Lost,
                };
                self.sounds.push(if all_fed { "win" } else { "lose" });
                changed = true;
            }
        }
        if changed {
            self.touch();
        }
    }

    // --- the player's actions --------------------------------------------------------

    /// One named action from the page. An empty string means it was done;
    /// otherwise the reason it was refused.
    pub fn act(&mut self, name: &str, value: &str) -> Result<(), String> {
        let num = |v: &str| {
            v.parse::<usize>()
                .map_err(|_| format!("\"{v}\" is not a number."))
        };
        let pair = |v: &str| -> Result<(usize, usize), String> {
            let (a, b) = v
                .split_once(':')
                .ok_or_else(|| format!("\"{v}\" is not two numbers."))?;
            Ok((num(a)?, num(b)?))
        };
        match name {
            "screen.title" => self.mode = Mode::Title,
            "level.choose" => {
                let n = num(value)?;
                if n >= self.c.levels.len() {
                    return Err(format!("There is no level {}.", n + 1));
                }
                self.level = n;
                self.play = None;
                self.mode = Mode::Intro;
            }
            "level.start" => {
                self.need(Mode::Intro)?;
                self.start_level();
            }
            "level.next" => {
                self.need(Mode::Won)?;
                self.level += 1;
                self.play = None;
                self.mode = Mode::Intro;
            }
            "level.retry" => {
                if !matches!(self.mode, Mode::Lost | Mode::Play) {
                    return Err("There is no level to try again.".into());
                }
                self.play = None;
                self.mode = Mode::Intro;
            }
            "pizza.pick" => {
                let (layer, option) = pair(value)?;
                let picks = self.level_def().layout.picks.clone();
                let play = self.playing()?;
                if layer >= picks.len() || option >= picks[layer] as usize {
                    return Err(format!("There is no option {value}."));
                }
                play.pizza.picks[layer] = option as u8;
                self.sounds.push("click");
            }
            "pizza.top" => {
                let t = num(value)?;
                let n = self.level_def().layout.toppings as usize;
                let play = self.playing()?;
                if t >= n {
                    return Err(format!("There is no topping {value}."));
                }
                play.pizza.tops ^= 1 << t;
                self.sounds.push("click");
            }
            "pizza.clear" => {
                let play = self.playing()?;
                play.pizza.tops = 0;
                play.pizza.picks.iter_mut().for_each(|p| *p = 0);
            }
            "pizza.rebuild" => {
                let (d, k) = pair(value)?;
                let play = self.playing()?;
                let entry = play
                    .seats
                    .get(d)
                    .and_then(|s| s.log.get(k))
                    .ok_or_else(|| format!("There is no logged pizza {value}."))?;
                play.pizza = entry.pizza.clone();
            }
            "serve" => self.serve(num(value)?)?,
            "solver.suggest" => {
                // the solver's next pizza for a drummer, from his hints alone:
                // used by the browser check to play a level through
                let d = num(value)?;
                let def = self.level_def().clone();
                let play = self.playing()?;
                let seat = play
                    .seats
                    .get(d)
                    .ok_or_else(|| format!("There is no drummer {value}."))?;
                let history: Vec<(Pizza, Feedback)> =
                    seat.log.iter().map(|e| (e.pizza.clone(), e.feedback)).collect();
                let left = solver::consistent(&def.layout.orders(), &history, def.clarity);
                play.pizza = solver::choose(&left, &def.layout.pizzas(), def.clarity);
            }
            other => return Err(format!("Unknown action \"{other}\".")),
        }
        self.touch();
        Ok(())
    }

    fn need(&self, mode: Mode) -> Result<(), String> {
        if self.mode != mode {
            return Err(format!("That needs the {} screen.", mode.name()));
        }
        Ok(())
    }

    fn playing(&mut self) -> Result<&mut Play, String> {
        if self.mode != Mode::Play {
            return Err("That needs a level in play.".into());
        }
        self.play
            .as_mut()
            .ok_or_else(|| "That needs a level in play.".to_string())
    }

    fn start_level(&mut self) {
        let def = self.level_def().clone();
        let orders = def.layout.orders();
        let seats = (0..def.drummers)
            .map(|_| {
                let k = self.rng.range(0, orders.len() as i32 - 1) as usize;
                Seat {
                    order: orders[k].clone(),
                    log: Vec::new(),
                    fed: false,
                    run: None,
                    mood: Mood::Waiting,
                    mood_at: self.clock,
                }
            })
            .collect();
        self.play = Some(Play {
            seats,
            left: def.patience,
            pizza: Pizza {
                picks: vec![0; def.layout.picks.len()],
                tops: 0,
            },
            ended_at: None,
        });
        self.mode = Mode::Play;
    }

    /// Whether drummer `d` can be served now, and if not, why.
    fn can_serve(&self, d: usize) -> Result<(), String> {
        let play = self.play.as_ref().filter(|_| self.mode == Mode::Play);
        let play = play.ok_or_else(|| "That needs a level in play.".to_string())?;
        let seat = play
            .seats
            .get(d)
            .ok_or_else(|| format!("There is no drummer {}.", d + 1))?;
        let name = &self.c.band[d].name;
        if seat.fed {
            return Err(format!("{name} already has his pizza."));
        }
        if seat.run.is_some() {
            return Err(format!("{name} is still tasting."));
        }
        if play.left == 0 {
            return Err("The band has no patience left.".into());
        }
        Ok(())
    }

    fn serve(&mut self, d: usize) -> Result<(), String> {
        self.can_serve(d)?;
        let clarity = self.level_def().clarity;
        let now = self.clock;
        let (pizza, order) = {
            let play = self.play.as_ref().ok_or("no play")?;
            (play.pizza.clone(), play.seats[d].order.clone())
        };
        let feedback = judge(&order, &pizza, clarity);
        let line = self.hint_line(&feedback, clarity);
        let riffs = match feedback.verdict {
            Verdict::Perfect => &self.c.riffs.perfect,
            Verdict::Close => &self.c.riffs.close,
            Verdict::Miss => &self.c.riffs.miss,
        };
        let riff = riffs[self.rng.range(0, riffs.len() as i32 - 1) as usize].clone();
        let play = self.play.as_mut().ok_or("no play")?;
        play.left -= 1;
        play.seats[d].run = Some(Run {
            pizza,
            feedback,
            line,
            at: now,
            riff,
            played: 0,
            revealed: false,
        });
        play.seats[d].mood = Mood::Waiting;
        self.sounds.push("serve");
        Ok(())
    }

    /// The name a hint uses for an item, lower case.
    fn item_name(&self, item: Item) -> String {
        let def = self.level_def();
        let name = match item {
            Item::Pick { layer, option } => {
                let l = &self.c.menu.layers[def.layers[layer as usize]];
                l.options[option as usize].name.clone()
            }
            Item::Topping(t) => self.c.menu.toppings[t as usize].name.clone(),
        };
        name.to_lowercase()
    }

    /// The list a hint is drawn from, and the fills for its placeholders.
    pub fn hint_key(&self, f: &Feedback, clarity: Clarity) -> (&'static str, Vec<(&'static str, String)>) {
        let one_or_many = |n: u8, one: &'static str, many: &'static str| if n == 1 { one } else { many };
        match (f.verdict, f.detail) {
            (Verdict::Perfect, _) => ("perfect", vec![]),
            (Verdict::Close, Detail::Count(n)) if clarity != Clarity::Vague => (
                one_or_many(n, "close_one", "close_many"),
                vec![("count", n.to_string())],
            ),
            (Verdict::Close, _) => ("close_vague", vec![]),
            (Verdict::Miss, Detail::Named(item, n)) => (
                one_or_many(n, "miss_named_one", "miss_named_many"),
                vec![("item", self.item_name(item)), ("count", n.to_string())],
            ),
            (Verdict::Miss, Detail::Count(n)) => (
                one_or_many(n, "miss_counted_one", "miss_counted_many"),
                vec![("count", n.to_string())],
            ),
            (Verdict::Miss, Detail::None) => ("miss_vague", vec![]),
        }
    }

    fn hint_line(&mut self, f: &Feedback, clarity: Clarity) -> String {
        let (key, fills) = self.hint_key(f, clarity);
        let lines = &self.c.hints[key];
        let k = self.rng.range(0, lines.len() as i32 - 1) as usize;
        let mut s = lines[k].clone();
        for (name, v) in fills {
            s = s.replace(&format!("{{{name}}}"), &v);
        }
        s
    }

    // --- the view -----------------------------------------------------------------

    /// CSS size and device pixel ratio (in hundredths) in; drawing-buffer size out.
    pub fn resize(&mut self, css_w: i32, css_h: i32, dpr100: i32) -> [i32; 2] {
        let w = (css_w as i64 * dpr100 as i64 / 100).clamp(1, 4096) as i32;
        let h = (css_h as i64 * dpr100 as i64 / 100).clamp(1, 4096) as i32;
        self.size = (w, h);
        self.frame = None;
        self.touch();
        [w, h]
    }

    pub(crate) fn aspect(&self) -> i32 {
        fx::div(int(self.size.0.max(1)), int(self.size.1.max(1)))
    }

    /// How many drummers are on stage: the level's, or the whole band on the
    /// title screen.
    pub(crate) fn on_stage(&self) -> usize {
        match self.mode {
            Mode::Title | Mode::Finale => self.c.band.len(),
            _ => self.level_def().drummers,
        }
    }

    pub(crate) fn seat_x(&self, d: usize) -> i32 {
        let n = self.on_stage() as i32;
        (2 * d as i32 - (n - 1)) * SEAT_SPACING / 2
    }

    /// The camera backs away until the whole band fits across the screen.
    pub(crate) fn camera(&self) -> Camera {
        let fov = fx::deg(46);
        let half = fx::div(fx::sin(fov / 2), fx::cos(fov / 2));
        let across = fx::mul(half, self.aspect()).max(ONE / 8);
        let band = (self.on_stage() as i32 - 1) * SEAT_SPACING / 2 + BAND_MARGIN;
        let dist = fx::div(band, across).max(int(8) + ONE / 2);
        let target = v3(0, int(1) + ONE * 35 / 100, -int(1) - ONE * 6 / 10);
        let back = v3(0, ONE * 36 / 100, ONE).norm();
        let eye = target.add(back.scale(dist));
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov,
            far: int(200),
        }
    }

    pub(crate) fn head_of(&self, d: usize) -> V3 {
        v3(self.seat_x(d), int(2) + ONE / 4, -int(3) - ONE * 6 / 10)
    }

    /// The scene: instance values and draws, built once per frame.
    fn built(&mut self) -> &(Vec<i32>, Vec<u32>) {
        if self.frame.is_none() {
            self.frame = Some(self.scene().finish());
        }
        self.frame.as_ref().unwrap_or_else(|| unreachable!())
    }

    pub fn instances(&mut self) -> Vec<i32> {
        self.built().0.clone()
    }

    pub fn draws(&mut self) -> Vec<u32> {
        self.built().1.clone()
    }

    pub fn mats(&self) -> Vec<i32> {
        self.camera().mat_values(self.aspect()).to_vec()
    }

    pub fn mesh(&self, id: u32) -> Vec<i32> {
        Mesh::ALL
            .get(id as usize)
            .map(|&m| mesh::vertices(m))
            .unwrap_or_default()
    }

    /// The shader's uniforms, in the order `numbers()` lists them.
    pub fn uniforms(&self) -> Vec<i32> {
        let light = v3(-ONE * 4 / 10, ONE * 8 / 10, ONE * 6 / 10).norm();
        let mut out = vec![light.x, light.y, light.z];
        for name in ["light", "ambient", "ink"] {
            out.extend_from_slice(&self.c.color(name));
        }
        // the ink line: least width, growth per metre of the shape, most width
        out.extend_from_slice(&[ONE * 2 / 100, ONE * 6 / 1000, ONE * 5 / 100]);
        // light bands: the bright threshold and the dim one, and the shade of each band
        out.extend_from_slice(&[ONE * 55 / 100, ONE / 10, ONE * 80 / 100, ONE * 60 / 100]);
        // planks (boards per metre, seam share) and stripes (per metre, dark share)
        out.extend_from_slice(&[ONE * 3 / 2, ONE / 12, ONE * 3 / 2, ONE / 2]);
        // shading: the share of ambient and of direct light, the shade of a seam
        out.extend_from_slice(&[ONE * 65 / 100, ONE * 55 / 100, ONE * 72 / 100]);
        out.push(ONE / 65536);
        out.push(ONE / 10000);
        out
    }

    pub fn clear_color(&self) -> Vec<i32> {
        self.c.color("sky").to_vec()
    }

    // --- the screen, as JSON --------------------------------------------------------

    fn chips(&self, p: &Pizza) -> Vec<Value> {
        let def = self.level_def();
        let mut out = Vec::new();
        for (k, &layer) in def.layers.iter().enumerate() {
            let o = &self.c.menu.layers[layer].options[p.picks[k] as usize];
            out.push(json!({ "name": o.name, "css": o.css, "shape": o.shape.name() }));
        }
        for t in 0..def.layout.toppings {
            if p.tops & (1 << t) != 0 {
                let o = &self.c.menu.toppings[t as usize];
                out.push(json!({ "name": o.name, "css": o.css, "shape": o.shape.name() }));
            }
        }
        if p.tops == 0 {
            out.push(json!({ "name": self.c.say("plain", &[]), "css": "", "shape": "none" }));
        }
        out
    }

    fn verdict_label(&self, v: Verdict) -> (&'static str, String) {
        let key = match v {
            Verdict::Perfect => "perfect",
            Verdict::Close => "close",
            Verdict::Miss => "miss",
        };
        (key, self.c.say(&format!("verdict_{key}"), &[]))
    }

    fn rules(&self) -> Vec<String> {
        let def = self.level_def();
        let n = def.drummers.to_string();
        let mut out = vec![self.c.say(
            if def.drummers == 1 {
                "drummers_one"
            } else {
                "drummers_many"
            },
            &[("count", n)],
        )];
        if def.layers.is_empty() {
            out.push(self.c.say("layers_none", &[]));
        } else {
            let names: Vec<String> = def
                .layers
                .iter()
                .map(|&l| self.c.menu.layers[l].name.to_lowercase())
                .collect();
            out.push(self.c.say("layers_some", &[("layers", names.join(", "))]));
        }
        out.push(self.c.say(
            "toppings_rule",
            &[
                ("count", def.layout.toppings.to_string()),
                ("min", def.layout.min.to_string()),
                ("max", def.layout.max.to_string()),
            ],
        ));
        let clarity = match def.clarity {
            Clarity::Named => "clarity_named",
            Clarity::Counted => "clarity_counted",
            Clarity::Vague => "clarity_vague",
        };
        out.push(self.c.say(clarity, &[]));
        out.push(
            self.c
                .say("patience_rule", &[("count", def.patience.to_string())]),
        );
        out
    }

    /// Percent across and down, as CSS lengths, of a point on the stage.
    fn css_at(&self, p: V3) -> Value {
        let cam = self.camera();
        match cam.screen(self.aspect(), p) {
            Some((x, y)) => {
                let tenth = |q: i32| {
                    let t = (q as i64 * 10 / ONE as i64) as i32;
                    format!("{}.{}%", t / 10, (t % 10).abs())
                };
                json!([tenth(x.clamp(0, int(100))), tenth(y.clamp(0, int(100)))])
            }
            None => Value::Null,
        }
    }

    pub fn ui(&self) -> Value {
        let def = self.level_def();
        let levels: Vec<Value> = self
            .c
            .levels
            .iter()
            .enumerate()
            .map(|(k, l)| {
                json!({
                    "value": k.to_string(),
                    "label": self.c.say("level_label", &[("n", (k + 1).to_string())]),
                    "name": l.name,
                })
            })
            .collect();
        let mut ui = json!({
            "mode": self.mode.name(),
            "version": self.version,
            "levels": levels,
            "level": {
                "label": self.c.say("level_label", &[("n", (self.level + 1).to_string())]),
                "name": def.name,
                "rules": self.rules(),
                "last": self.level + 1 == self.c.levels.len(),
            },
        });
        if let Some(play) = &self.play {
            let layers: Vec<Value> = def
                .layers
                .iter()
                .enumerate()
                .map(|(k, &l)| {
                    let layer = &self.c.menu.layers[l];
                    let options: Vec<Value> = layer
                        .options
                        .iter()
                        .enumerate()
                        .map(|(o, ing)| {
                            json!({
                                "name": ing.name, "css": ing.css, "shape": ing.shape.name(),
                                "on": play.pizza.picks[k] as usize == o,
                                "value": format!("{k}:{o}"),
                            })
                        })
                        .collect();
                    json!({ "name": layer.name, "options": options })
                })
                .collect();
            let toppings: Vec<Value> = (0..def.layout.toppings)
                .map(|t| {
                    let ing = &self.c.menu.toppings[t as usize];
                    json!({
                        "name": ing.name, "css": ing.css, "shape": ing.shape.name(),
                        "on": play.pizza.tops & (1 << t) != 0,
                        "value": t.to_string(),
                    })
                })
                .collect();
            let drummers: Vec<Value> = play
                .seats
                .iter()
                .enumerate()
                .map(|(d, seat)| {
                    let name = &self.c.band[d].name;
                    let log: Vec<Value> = seat
                        .log
                        .iter()
                        .enumerate()
                        .map(|(k, e)| {
                            let (verdict, label) = self.verdict_label(e.feedback.verdict);
                            json!({
                                "chips": self.chips(&e.pizza),
                                "verdict": verdict,
                                "verdict_label": label,
                                "line": e.line,
                                "rebuild": format!("{d}:{k}"),
                            })
                        })
                        .collect();
                    let bubble = match (&seat.run, seat.log.last()) {
                        (Some(r), _) if !r.revealed => {
                            json!({ "verdict": "tasting", "label": "", "line": self.c.say("tasting", &[]) })
                        }
                        (_, Some(e)) => {
                            let (verdict, label) = self.verdict_label(e.feedback.verdict);
                            json!({ "verdict": verdict, "label": label, "line": e.line })
                        }
                        _ => Value::Null,
                    };
                    let status = if seat.fed {
                        self.c.say("fed", &[])
                    } else if seat.run.is_some() {
                        self.c.say("tasting", &[])
                    } else {
                        String::new()
                    };
                    json!({
                        "name": name,
                        "value": d.to_string(),
                        "fed": seat.fed,
                        "status": status,
                        "can_serve": self.can_serve(d).is_ok(),
                        "serve_label": self.c.say("serve_to", &[("name", name.clone())]),
                        "bubble": bubble,
                        "at": self.css_at(self.head_of(d).add(v3(0, ONE * 4 / 10, 0))),
                        "log": log,
                    })
                })
                .collect();
            let open = play.seats.iter().filter(|s| !s.fed).count();
            // a bubble is as wide as the gap between two drummers on screen
            let across = |d: usize| {
                self.camera()
                    .screen(self.aspect(), self.head_of(d))
                    .map(|(x, _)| x)
                    .unwrap_or(0)
            };
            let width = match play.seats.len() {
                1 => int(34),
                _ => ((across(1) - across(0)) * 92 / 100).clamp(int(10), int(34)),
            };
            ui["play"] = json!({
                "layers": layers,
                "toppings": toppings,
                "drummers": drummers,
                "pizza": self.chips(&play.pizza),
                "patience": self.c.say("patience_value", &[
                    ("left", play.left.to_string()),
                    ("total", def.patience.to_string()),
                ]),
                "patience_left": play.left,
                "bubble_width": format!("{}%", width / ONE),
                "won_text": self.c.say("won_text", &[("count", play.left.to_string())]),
                "lost_text": self.c.say("lost_text", &[("count", open.to_string())]),
            });
        }
        ui
    }

    pub fn ui_json(&self) -> String {
        self.ui().to_string()
    }
}

/// The layout of everything the page's renderer reads, so `gl.js` holds no
/// size, stride or offset of its own.
pub fn numbers() -> String {
    let i = |name: &str, size: usize, offset: usize| json!([name, size, offset * 4]);
    json!({
        "vertex": { "stride": mesh::VERTEX_VALUES * 4, "attributes": [i("aPos", 3, 0), i("aNormal", 3, 3)] },
        "instance": {
            "stride": render::INSTANCE_VALUES * 4,
            "attributes": [i("iA", 4, 0), i("iB", 4, 4), i("iC", 4, 8), i("iD", 4, 12)]
        },
        "passes": { "solid": render::PASS_SOLID, "ink": render::PASS_INK, "tint": render::PASS_TINT },
        "draw_values": render::DRAW_VALUES,
        "vertex_values": mesh::VERTEX_VALUES,
        "view_values": 16,
        "mat_values": render::MAT_VALUES,
        "mesh_count": render::MESH_COUNT,
        "uniforms": [
            ["uLightDir", 3], ["uLight", 3], ["uAmbient", 3], ["uInk", 3], ["uInkSize", 3],
            ["uBands", 4], ["uPattern", 4], ["uShade", 3], ["uQ", 1], ["uEps", 1]
        ],
        "sounds": Drum::ALL.iter().map(|d| d.key()).chain(SOUNDS).collect::<Vec<_>>(),
    })
    .to_string()
}

/// The sounds of one frame, as a JSON list of names.
pub fn sounds_json(sounds: &[&str]) -> String {
    serde_json::to_string(sounds).unwrap_or_default()
}

/// White noise for the page's sound, from the core's seeded generator.
pub fn noise_samples(n: u32, seed: u32) -> Vec<i32> {
    let mut rng = Rng::new(seed as u64);
    (0..n.min(1 << 20)).map(|_| rng.range(-ONE, ONE)).collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::content::tests::files;

    pub fn game() -> Game {
        let mut g = Game::new(&files(), 7).unwrap();
        g.advance(0);
        g
    }

    /// Runs the clock forward in frames of `LONGEST_FRAME_MS`, collecting sounds.
    pub fn run_for(g: &mut Game, ms: i64) -> Vec<&'static str> {
        let mut out = Vec::new();
        let mut t = g.page_clock.unwrap_or(0);
        let end = t + ms;
        while t < end {
            t += LONGEST_FRAME_MS.min(end - t);
            out.extend(g.advance(t));
        }
        out
    }

    fn started(level: usize) -> Game {
        let mut g = game();
        g.act("level.choose", &level.to_string()).unwrap();
        g.act("level.start", "").unwrap();
        g
    }

    fn serve_and_wait(g: &mut Game, d: usize) {
        g.act("serve", &d.to_string()).unwrap();
        while g.play.as_ref().unwrap().seats[d].run.is_some() {
            run_for(g, 100);
        }
    }

    #[test]
    fn choosing_and_starting_a_level_deals_each_drummer_an_order_and_full_patience() {
        let g = started(2);
        let p = g.play.as_ref().unwrap();
        assert_eq!(g.mode, Mode::Play);
        assert_eq!(p.seats.len(), 2);
        assert_eq!(p.left, 10);
        let orders = g.level_def().layout.orders();
        assert!(p.seats.iter().all(|s| orders.contains(&s.order)));
    }

    #[test]
    fn a_serve_costs_one_patience_and_its_hint_shows_only_after_the_riff() {
        let mut g = started(0);
        g.act("pizza.top", "0").unwrap();
        g.act("serve", "0").unwrap();
        assert_eq!(g.play.as_ref().unwrap().left, 4);
        let sounds = run_for(&mut g, WALK_MS + TASTE_MS - LONGEST_FRAME_MS);
        assert!(
            g.play.as_ref().unwrap().seats[0].log.is_empty(),
            "nothing said during the walk"
        );
        assert!(!sounds.contains(&"kick") && !sounds.contains(&"snare"));
        let sounds = run_for(&mut g, 6000);
        assert_eq!(g.play.as_ref().unwrap().seats[0].log.len(), 1);
        assert!(
            sounds.iter().any(|s| Drum::ALL.iter().any(|d| d.key() == *s)),
            "{sounds:?}"
        );
    }

    #[test]
    fn a_drummer_still_tasting_cannot_be_served_again() {
        let mut g = started(0);
        g.act("serve", "0").unwrap();
        let e = g.act("serve", "0").unwrap_err();
        assert!(e.contains("still tasting"), "{e}");
        assert_eq!(g.play.as_ref().unwrap().left, 4);
    }

    #[test]
    fn two_drummers_can_taste_at_once() {
        let mut g = started(2);
        g.act("serve", "0").unwrap();
        g.act("serve", "1").unwrap();
        let p = g.play.as_ref().unwrap();
        assert!(p.seats[0].run.is_some() && p.seats[1].run.is_some());
        assert_eq!(p.left, 8);
    }

    #[test]
    fn serving_every_order_clears_the_level_and_running_out_loses_it() {
        let mut g = started(2);
        for d in 0..2 {
            let order = g.play.as_ref().unwrap().seats[d].order.clone();
            g.play.as_mut().unwrap().pizza = order;
            serve_and_wait(&mut g, d);
        }
        assert!(g.play.as_ref().unwrap().seats.iter().all(|s| s.fed));
        run_for(&mut g, CHEER_MS + 100);
        assert_eq!(g.mode, Mode::Won);
        g.act("level.next", "").unwrap();
        assert_eq!((g.mode, g.level), (Mode::Intro, 3));

        let mut g = started(0);
        // an empty pizza is close for every order, so it never wins
        for _ in 0..5 {
            serve_and_wait(&mut g, 0);
        }
        assert_eq!(g.play.as_ref().unwrap().left, 0);
        assert!(g.act("serve", "0").unwrap_err().contains("no patience"));
        let sounds = run_for(&mut g, CHEER_MS + 100);
        assert_eq!(g.mode, Mode::Lost);
        assert!(sounds.contains(&"lose"));
    }

    #[test]
    fn clearing_the_last_level_reaches_the_finale() {
        let mut g = started(9);
        for d in 0..4 {
            let order = g.play.as_ref().unwrap().seats[d].order.clone();
            g.play.as_mut().unwrap().pizza = order;
            serve_and_wait(&mut g, d);
        }
        run_for(&mut g, CHEER_MS + 100);
        assert_eq!(g.mode, Mode::Finale);
    }

    #[test]
    fn the_solver_playing_through_the_page_actions_clears_every_level() {
        for level in 0..10 {
            let mut g = started(level);
            let n = g.level_def().drummers;
            while g.mode == Mode::Play {
                for d in 0..n {
                    if g.can_serve(d).is_ok() {
                        g.act("solver.suggest", &d.to_string()).unwrap();
                        g.act("serve", &d.to_string()).unwrap();
                    }
                }
                run_for(&mut g, 500);
            }
            assert!(
                matches!(g.mode, Mode::Won | Mode::Finale),
                "level {}: {:?}",
                level + 1,
                g.mode
            );
        }
    }

    #[test]
    fn a_hint_picks_its_list_by_verdict_clarity_and_count_and_fills_it() {
        let g = started(4); // sauce, counted
        let named = Feedback {
            verdict: Verdict::Miss,
            detail: Detail::Named(Item::Pick { layer: 0, option: 1 }, 2),
        };
        let (key, fills) = g.hint_key(&named, Clarity::Named);
        assert_eq!(key, "miss_named_many");
        assert_eq!(
            fills,
            vec![("item", "pesto".to_string()), ("count", "2".to_string())]
        );
        let close = Feedback {
            verdict: Verdict::Close,
            detail: Detail::Count(1),
        };
        assert_eq!(g.hint_key(&close, Clarity::Counted).0, "close_one");
        let vague = Feedback {
            verdict: Verdict::Close,
            detail: Detail::None,
        };
        assert_eq!(g.hint_key(&vague, Clarity::Vague).0, "close_vague");
    }

    #[test]
    fn every_hint_line_uses_only_the_placeholders_its_list_is_given() {
        let c = Content::load(&files()).unwrap();
        for (key, lines) in &c.hints {
            let allowed: &[&str] = match key.as_str() {
                "miss_named_one" | "miss_named_many" => &["{item}", "{count}"],
                "close_one" | "close_many" | "miss_counted_one" | "miss_counted_many" => &["{count}"],
                _ => &[],
            };
            for line in lines {
                let mut rest = line.clone();
                for a in allowed {
                    rest = rest.replace(a, "");
                }
                assert!(!rest.contains('{'), "{key}: {line}");
            }
        }
    }

    #[test]
    fn the_pizza_follows_the_players_picks_and_toppings() {
        let mut g = started(4);
        g.act("pizza.pick", "0:2").unwrap();
        g.act("pizza.top", "3").unwrap();
        g.act("pizza.top", "1").unwrap();
        g.act("pizza.top", "3").unwrap();
        let p = &g.play.as_ref().unwrap().pizza;
        assert_eq!((p.picks.clone(), p.tops), (vec![2], 0b10));
        assert!(g.act("pizza.top", "5").is_err(), "level 5 has five toppings");
        assert!(g.act("pizza.pick", "0:3").is_err());
        g.act("pizza.clear", "").unwrap();
        let p = &g.play.as_ref().unwrap().pizza;
        assert_eq!((p.picks.clone(), p.tops), (vec![0], 0));
    }

    #[test]
    fn a_logged_pizza_can_be_rebuilt() {
        let mut g = started(0);
        g.act("pizza.top", "2").unwrap();
        serve_and_wait(&mut g, 0);
        g.act("pizza.clear", "").unwrap();
        g.act("pizza.rebuild", "0:0").unwrap();
        assert_eq!(g.play.as_ref().unwrap().pizza.tops, 0b100);
    }

    #[test]
    fn the_screen_lists_the_serve_buttons_and_the_rules() {
        let mut g = started(2);
        let ui = g.ui();
        assert_eq!(ui["mode"], "play");
        assert_eq!(ui["play"]["drummers"][1]["serve_label"], "Serve Snare Drum Stan");
        assert_eq!(ui["play"]["drummers"][1]["can_serve"], true);
        assert_eq!(ui["play"]["patience"], "10 of 10");
        assert_eq!(ui["level"]["rules"][0], "2 drummers to feed at once.");
        g.act("serve", "1").unwrap();
        assert_eq!(g.ui()["play"]["drummers"][1]["can_serve"], false);
    }

    #[test]
    fn the_clock_never_jumps_more_than_one_long_frame() {
        let mut g = game();
        g.advance(1_000_000);
        assert_eq!(g.clock, LONGEST_FRAME_MS);
        g.advance(999_000);
        assert_eq!(g.clock, LONGEST_FRAME_MS, "a clock going backwards is ignored");
    }

    #[test]
    fn unknown_actions_and_values_are_refused_by_name() {
        let mut g = game();
        assert_eq!(g.act("dance", "").unwrap_err(), "Unknown action \"dance\".");
        assert_eq!(g.act("level.choose", "10").unwrap_err(), "There is no level 11.");
        assert!(g.act("serve", "0").is_err());
    }

    #[test]
    fn the_band_fits_across_a_narrow_screen() {
        let mut g = started(9);
        g.resize(400, 800, 100);
        let ui = g.ui();
        for d in 0..4 {
            let at = &ui["play"]["drummers"][d]["at"];
            let x: i32 = at[0]
                .as_str()
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert!((2..=98).contains(&x), "drummer {d} at {at}");
        }
    }
}
