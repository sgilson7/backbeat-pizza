//! The stage, built fresh each frame from the game's state: the room, the
//! drummers at their kits (their sticks follow the riff being played), the
//! Masklings carrying pizzas, and the pizza on the counter.
//!
//! Lengths are in centimetres through `cm`, angles in degrees through
//! `fx::deg`, and the page's clock drives every motion.

use crate::content::{Drum, Rgb, Shape};
use crate::fx::{self, deg, ONE};
use crate::game::{Game, Mode, Mood, Seat, REACT_MS, TASTE_MS, WALK_MS};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::puzzle::Pizza;
use crate::render::{boxed, segment, standing, Batch, STYLE_GLOW, STYLE_LIT, STYLE_PLANKS, STYLE_STRIPES};

const fn cm(n: i32) -> i32 {
    n * ONE / 100
}

fn p(x: i32, y: i32, z: i32) -> V3 {
    v3(cm(x), cm(y), cm(z))
}

/// An angle that turns once every `period` milliseconds.
fn phase(clock: i64, period: i64) -> i32 {
    ((clock.rem_euclid(period)) * fx::TURN as i64 / period) as i32
}

/// 0 to ONE, easing in and out.
fn smooth(t: i32) -> i32 {
    let t = t.clamp(0, ONE);
    fx::mul(fx::mul(t, t), 3 * ONE - 2 * t)
}

fn shade(c: Rgb, pct: i32) -> Rgb {
    [c[0] * pct / 100, c[1] * pct / 100, c[2] * pct / 100]
}

/// The riser's top: drummers and kits stand on it.
const RISER: i32 = cm(40);
/// How far back the drummers sit.
const SEAT_Z: i32 = -cm(370);
/// Where a Maskling hands a pizza up.
const HANDOFF_Z: i32 = -cm(175);
const COUNTER_Z: i32 = cm(165);
const COUNTER_TOP: i32 = cm(74);

/// The hand that plays each hit of a riff: the left takes the hi-hat, the
/// snare and the high tom; the right the crash and the floor tom, and the
/// snare when the left is on the hi-hat at the same step.
pub fn hands(hits: &[(u32, Drum)]) -> [Vec<(u32, Drum)>; 2] {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for &(s, d) in hits {
        match d {
            Drum::Hat | Drum::TomHi => left.push((s, d)),
            Drum::Crash | Drum::TomLo => right.push((s, d)),
            Drum::Snare => {
                if hits.contains(&(s, Drum::Hat)) {
                    right.push((s, d))
                } else {
                    left.push((s, d))
                }
            }
            Drum::Kick => {}
        }
    }
    [left, right]
}

/// Where a stick's tip lands on each drum of the kit at seat `x`.
fn kit_top(x: i32, d: Drum) -> V3 {
    let at = |dx: i32, y: i32, z: i32| v3(x + cm(dx), RISER + cm(y), cm(z));
    match d {
        Drum::Hat => at(-85, 112, -312),
        Drum::Snare => at(-40, 94, -298),
        Drum::TomHi => at(-6, 130, -292),
        Drum::TomLo => at(50, 84, -298),
        Drum::Crash => at(80, 158, -315),
        Drum::Kick => at(0, 84, -275),
    }
}

/// From the hand to the stick's tip.
fn stick() -> V3 {
    p(0, -16, 40)
}

impl Game {
    pub(crate) fn scene(&self) -> Batch {
        let mut b = Batch::new();
        self.room(&mut b);
        let n = self.on_stage();
        for d in 0..n {
            let seat = self
                .play
                .as_ref()
                .filter(|_| self.mode != Mode::Title)
                .and_then(|p| p.seats.get(d));
            self.drummer(&mut b, d, seat);
        }
        self.counter(&mut b);
        self.masklings(&mut b);
        b
    }

    fn room(&self, b: &mut Batch) {
        let c = |k: &str| self.c.color(k);
        let half = (self.on_stage() as i32 - 1) * crate::game::SEAT_SPACING / 2;
        b.plain(
            Mesh::Cube,
            &boxed(p(0, -10, -100), p(4000, 20, 2400)),
            c("floor"),
            STYLE_PLANKS,
        );
        b.plain(
            Mesh::Cube,
            &boxed(p(0, 450, -640), p(4000, 900, 40)),
            c("wall"),
            STYLE_LIT,
        );
        b.plain(
            Mesh::Cube,
            &boxed(p(0, 380, -600), p(3000, 760, 20)),
            c("curtain"),
            STYLE_STRIPES,
        );
        // string lights across the curtain, in the band's colours
        let bulbs = 33;
        for k in 0..bulbs {
            let x = cm(-1600) + k * cm(3200) / (bulbs - 1);
            let sag = fx::mul(cm(30), fx::sin(fx::HALF * (k % 8) / 8).abs());
            let color = if k % 3 == 0 {
                self.c.band[(k as usize / 3) % self.c.band.len()].band
            } else {
                c("bulb")
            };
            b.plain(
                Mesh::Sphere,
                &boxed(v3(x, cm(640) - sag, -cm(570)), p(18, 18, 18)),
                color,
                STYLE_GLOW,
            );
        }
        // speakers either side of the band, their cones thumping with the kicks
        let thump = self.kick_pulse();
        for side in [-1, 1] {
            let x = side * (half + cm(260));
            b.ink(
                Mesh::Cube,
                &standing(v3(x, 0, -cm(300)), p(130, 230, 100)),
                c("speaker"),
                STYLE_LIT,
            );
            for (y, r) in [(150, 80), (55, 50)] {
                let r = cm(r) + fx::mul(cm(6), thump);
                let face = Affine::translate(v3(x, cm(y), -cm(248)))
                    .then(&Affine::rot_x(fx::QUARTER))
                    .then(&Affine::scale(v3(r, cm(4), r)));
                b.ink(Mesh::Cyl, &face, shade(c("chrome"), 45), STYLE_LIT);
            }
        }
        for d in 0..self.on_stage() {
            let x = self.seat_x(d);
            b.ink(
                Mesh::Cube,
                &standing(v3(x, 0, -cm(320)), p(270, 40, 230)),
                c("riser"),
                STYLE_LIT,
            );
        }
    }

    /// ONE just after any drummer's kick, falling to 0.
    fn kick_pulse(&self) -> i32 {
        let Some(play) = &self.play else {
            return 0;
        };
        let mut best = 0;
        for (d, seat) in play.seats.iter().enumerate() {
            if let Some(run) = &seat.run {
                let t = self.clock - run.at - WALK_MS - TASTE_MS;
                let step = self.c.band[d].step_ms;
                for &(s, drum) in &run.riff.hits {
                    let since = t - s as i64 * step;
                    if drum == Drum::Kick && (0..120).contains(&since) {
                        best = best.max(ONE - (since as i32 * ONE / 120));
                    }
                }
            }
        }
        best
    }

    /// How far into his riff a drummer is, in ms, if he is playing.
    fn riff_clock(&self, seat: Option<&Seat>) -> Option<i64> {
        let run = seat?.run.as_ref()?;
        let t = self.clock - run.at - WALK_MS - TASTE_MS;
        (t >= 0).then_some(t)
    }

    fn drummer(&self, b: &mut Batch, d: usize, seat: Option<&Seat>) {
        let who = &self.c.band[d];
        let c = |k: &str| self.c.color(k);
        let x = self.seat_x(d);
        let step = who.step_ms;
        let riff_t = self.riff_clock(seat);
        let run = seat.and_then(|s| s.run.as_ref());
        // how long ago each drum was hit, for the squash
        let since_hit = |drum: Drum| -> i64 {
            let (Some(t), Some(run)) = (riff_t, run) else {
                return i64::MAX;
            };
            run.riff
                .hits
                .iter()
                .filter(|(_, d)| *d == drum)
                .map(|(s, _)| t - *s as i64 * step)
                .filter(|s| *s >= 0)
                .min()
                .unwrap_or(i64::MAX)
        };
        let squash = |drum: Drum| -> i32 {
            let s = since_hit(drum);
            if s < 140 {
                cm(6) - (s as i32) * cm(6) / 140
            } else {
                0
            }
        };

        // --- the kit ---------------------------------------------------------
        let shell = who.shell;
        let head = c("drumhead");
        let chrome = c("chrome");
        let kick = squash(Drum::Kick);
        let bass = Affine::translate(v3(x, RISER + cm(42), -cm(272)))
            .then(&Affine::rot_x(fx::QUARTER))
            .then(&Affine::scale(v3(cm(84) + kick, cm(42), cm(84) + kick)));
        b.ink(Mesh::Cyl, &bass, shell, STYLE_LIT);
        let face = Affine::translate(v3(x, RISER + cm(42), -cm(250)))
            .then(&Affine::rot_x(fx::QUARTER))
            .then(&Affine::scale(v3(cm(74), cm(2), cm(74))));
        b.plain(Mesh::Cyl, &face, head, STYLE_LIT);
        let logo = Affine::translate(v3(x, RISER + cm(42), -cm(248)))
            .then(&Affine::rot_x(fx::QUARTER))
            .then(&Affine::scale(v3(cm(40), cm(3), cm(40))));
        b.plain(Mesh::Ring, &logo, who.band, STYLE_LIT);
        let drum = |b: &mut Batch, d: Drum, w: i32, h: i32, tilt: i32| {
            let top = kit_top(x, d);
            let sq = squash(d) / 2;
            let a = Affine::translate(top)
                .then(&Affine::rot_x(deg(tilt)))
                .then(&Affine::translate(v3(0, -cm(h) / 2, 0)))
                .then(&Affine::scale(v3(cm(w), cm(h) - sq, cm(w))));
            b.ink(Mesh::Cyl, &a, shell, STYLE_LIT);
            let skin = Affine::translate(top)
                .then(&Affine::rot_x(deg(tilt)))
                .then(&Affine::scale(v3(cm(w) - cm(4), cm(2), cm(w) - cm(4))));
            b.plain(Mesh::Cyl, &skin, head, STYLE_LIT);
            b.plain(
                Mesh::Cyl,
                &segment(v3(top.x, RISER, top.z), v3(top.x, top.y - cm(h), top.z), cm(3)),
                chrome,
                STYLE_LIT,
            );
        };
        drum(b, Drum::Snare, 46, 16, 8);
        drum(b, Drum::TomHi, 36, 24, 18);
        drum(b, Drum::TomLo, 50, 44, 0);
        let cymbal = |b: &mut Batch, d: Drum, w: i32, tilt: i32, gap: i32| {
            let top = kit_top(x, d);
            let s = since_hit(d);
            let wobble = if s < 400 {
                fx::mul(deg(10), fx::sin(phase(s, 130))) * (400 - s as i32) / 400
            } else {
                0
            };
            for k in 0..(1 + (gap > 0) as i32) {
                let a = Affine::translate(top.add(v3(0, -cm(2) - k * cm(gap), 0)))
                    .then(&Affine::rot_x(deg(tilt) + wobble))
                    .then(&Affine::scale(v3(cm(w), cm(2), cm(w))));
                b.ink(Mesh::Cyl, &a, c("cymbal"), STYLE_LIT);
            }
            b.plain(
                Mesh::Cyl,
                &segment(
                    v3(top.x, RISER, top.z - cm(6)),
                    v3(top.x, top.y - cm(3), top.z),
                    cm(3),
                ),
                chrome,
                STYLE_LIT,
            );
        };
        cymbal(b, Drum::Hat, 42, 0, 4);
        cymbal(b, Drum::Crash, 66, 18, 0);
        b.ink(
            Mesh::Cyl,
            &standing(v3(x, RISER, SEAT_Z - cm(10)), p(44, 50, 44)),
            shade(chrome, 40),
            STYLE_LIT,
        );

        // --- the drummer -----------------------------------------------------
        let mood = seat
            .map(|s| (s.mood, self.clock - s.mood_at))
            .unwrap_or((Mood::Waiting, i64::MAX));
        let fed = seat.map(|s| s.fed).unwrap_or(false) || self.mode == Mode::Finale;
        let groove = if riff_t.is_some() {
            fx::mul(cm(4), fx::sin(phase(riff_t.unwrap_or(0), step * 4)).abs())
        } else if fed {
            fx::mul(cm(5), fx::sin(phase(self.clock, 520)).abs())
        } else {
            fx::mul(cm(2), fx::sin(phase(self.clock + d as i64 * 300, 1600)))
        };
        let (shake, nod) = match mood {
            (Mood::Grumpy, t) if t < REACT_MS => {
                let fade = (REACT_MS - t) as i32 * ONE / REACT_MS as i32;
                (fx::mul(fx::mul(cm(7), fx::sin(phase(t, 260))), fade), 0)
            }
            (Mood::Hopeful, t) if t < REACT_MS => {
                let fade = (REACT_MS - t) as i32 * ONE / REACT_MS as i32;
                (0, fx::mul(fx::mul(cm(5), fx::sin(phase(t, 450))), fade))
            }
            _ => (0, 0),
        };
        let pants = shade(c("speaker"), 140);
        let hips = v3(x, RISER + cm(58), SEAT_Z);
        for side in [-1, 1] {
            let hip = hips.add(v3(side * cm(18), 0, cm(5)));
            let knee = v3(x + side * cm(26), RISER + cm(58), SEAT_Z + cm(48));
            let foot = v3(x + side * cm(28), RISER + cm(6), SEAT_Z + cm(58));
            b.ink(Mesh::Cube, &segment(hip, knee, cm(20)), pants, STYLE_LIT);
            let tap = if side == 1 { kick / 2 } else { 0 };
            b.ink(
                Mesh::Cube,
                &segment(knee, foot.add(v3(0, tap, 0)), cm(17)),
                pants,
                STYLE_LIT,
            );
            b.ink(
                Mesh::Cube,
                &standing(foot.add(v3(0, 0, cm(6))), p(16, 10, 28)),
                c("ink"),
                STYLE_LIT,
            );
        }
        let chest = v3(x, RISER + cm(108) + groove / 2, SEAT_Z);
        b.ink(Mesh::Sphere, &boxed(chest, p(76, 96, 56)), who.shirt, STYLE_LIT);
        let head = v3(x + shake, RISER + cm(176) + groove + nod, SEAT_Z + cm(4));
        b.ink(Mesh::Sphere, &boxed(head, p(60, 62, 58)), who.skin, STYLE_LIT);
        b.ink(
            Mesh::Cyl,
            &boxed(head.add(p(0, 12, 0)), p(63, 10, 61)),
            who.band,
            STYLE_LIT,
        );
        b.plain(
            Mesh::Cube,
            &boxed(head.add(p(0, 3, 27)), p(48, 11, 6)),
            c("ink"),
            STYLE_LIT,
        );
        b.plain(
            Mesh::Cube,
            &boxed(head.add(p(-11, 5, 30)), p(10, 3, 2)),
            c("chrome"),
            STYLE_GLOW,
        );
        let (mouth_w, mouth_h) = match mood.0 {
            Mood::Pleased => (22, 9),
            Mood::Grumpy => (14, 3),
            _ if riff_t.is_some() => (12, 7),
            _ => (12, 3),
        };
        b.plain(
            Mesh::Cube,
            &boxed(head.add(p(0, -16, 28)), p(mouth_w, mouth_h, 3)),
            c("ink"),
            STYLE_LIT,
        );

        // arms: each hand follows its own hits; between riffs the sticks rest
        // over the snare and hi-hat, or go up in a V when he is fed
        let hand_hits = run.map(|r| hands(&r.riff.hits)).unwrap_or_default();
        for (k, side) in [-1i32, 1].into_iter().enumerate() {
            let rest_drum = if side < 0 { Drum::Hat } else { Drum::Snare };
            let rest = if fed && riff_t.is_none() {
                head.add(v3(side * cm(55), cm(55) + groove, cm(10))).sub(stick())
            } else {
                kit_top(x, rest_drum).add(p(side * 8, 22, -6)).sub(stick())
            };
            let hand = match riff_t {
                Some(t) => self.hand_at(x, &hand_hits[k], step, t, rest),
                None => rest,
            };
            let shoulder = chest.add(v3(side * cm(36), cm(26), 0));
            let elbow = v3(
                (shoulder.x + hand.x) / 2 + side * cm(16),
                (shoulder.y + hand.y) / 2 - cm(14),
                (shoulder.z + hand.z) / 2 - cm(4),
            );
            b.ink(
                Mesh::Cube,
                &segment(shoulder, elbow, cm(17)),
                who.shirt,
                STYLE_LIT,
            );
            b.ink(Mesh::Cube, &segment(elbow, hand, cm(13)), who.skin, STYLE_LIT);
            b.ink(Mesh::Sphere, &boxed(hand, p(13, 13, 13)), who.skin, STYLE_LIT);
            let tip = if fed && riff_t.is_none() {
                hand.add(p(side * 10, 40, 0))
            } else {
                hand.add(stick())
            };
            b.ink(
                Mesh::Cyl,
                &segment(hand.sub(p(0, -4, 8)), tip, cm(4)),
                c("stick"),
                STYLE_LIT,
            );
        }

        // the pizza being tasted, at his mouth and shrinking
        if let Some(run) = run {
            let t = self.clock - run.at;
            if (WALK_MS..WALK_MS + TASTE_MS).contains(&t) {
                let left = ONE - ((t - WALK_MS) as i32 * ONE / TASTE_MS as i32);
                let at = head.add(p(0, -30, 42));
                let tilt = Affine::translate(at).then(&Affine::rot_x(deg(60)));
                self.pizza(b, &tilt, fx::mul(cm(30), left.max(ONE / 10)), &run.pizza);
            }
        }

        // a spotlight on a fed drummer
        if fed {
            let beam =
                Affine::translate(v3(x, cm(320), SEAT_Z + cm(40))).then(&Affine::scale(p(300, 640, 300)));
            b.tint(Mesh::Cone, &beam, c("spot"), ONE * 14 / 100);
        }
    }

    /// A hand's position `t` ms into a riff: on each of its hits the stick's
    /// tip touches the drum; between hits it lifts in an arc to the next.
    fn hand_at(&self, x: i32, hits: &[(u32, Drum)], step: i64, t: i64, rest: V3) -> V3 {
        let at = |i: usize| (hits[i].0 as i64 * step, kit_top(x, hits[i].1).sub(stick()));
        if hits.is_empty() {
            return rest;
        }
        let next = hits.iter().position(|(s, _)| *s as i64 * step > t);
        let (from_t, from) = match next {
            Some(0) => (-step * 2, rest),
            Some(i) => at(i - 1),
            None => at(hits.len() - 1),
        };
        let (to_t, to) = match next {
            Some(i) => at(i),
            None => (from_t + step * 4, rest),
        };
        let span = (to_t - from_t).max(1);
        let u = ((t - from_t).clamp(0, span) * ONE as i64 / span) as i32;
        let lift = (span as i32 * cm(30) / (step as i32 * 4)).min(cm(32));
        from.lerp(to, smooth(u))
            .add(v3(0, fx::mul(lift, fx::sin(u / 2)), 0))
    }

    fn counter(&self, b: &mut Batch) {
        let c = |k: &str| self.c.color(k);
        b.ink(
            Mesh::Cube,
            &standing(v3(0, 0, COUNTER_Z), p(380, 70, 100)),
            c("counter"),
            STYLE_LIT,
        );
        b.ink(
            Mesh::Cube,
            &boxed(v3(0, COUNTER_TOP - cm(2), COUNTER_Z), p(400, 8, 116)),
            c("counter_top"),
            STYLE_LIT,
        );
        if let Some(play) = self.play.as_ref().filter(|_| self.mode == Mode::Play) {
            let peel =
                Affine::translate(v3(0, COUNTER_TOP + cm(12), COUNTER_Z)).then(&Affine::rot_x(deg(28)));
            b.ink(
                Mesh::Cyl,
                &peel
                    .then(&Affine::translate(p(0, -2, 0)))
                    .then(&Affine::scale(p(120, 3, 120))),
                c("chrome"),
                STYLE_LIT,
            );
            self.pizza(b, &peel, cm(54), &play.pizza);
        }
    }

    /// A pizza on a base `at` (its centre, the crust's underside; y is up
    /// through the pizza), `r` across from the middle to the rim.
    fn pizza(&self, b: &mut Batch, at: &Affine, r: i32, pizza: &Pizza) {
        let def = self.level_def();
        let menu = &self.c.menu;
        let chosen = |key: &str| {
            let layer = menu.layers.iter().position(|l| l.key == key)?;
            let k = def.layers.iter().position(|&l| l == layer);
            let option = k
                .map(|k| pizza.picks.get(k).copied().unwrap_or(0) as usize)
                .unwrap_or(0);
            menu.layers[layer].options.get(option)
        };
        let s = |n: i32| fx::mul(r, n * ONE / 100);
        let crust = chosen("crust");
        let (crust_h, crust_shape) = match crust.map(|c| c.shape) {
            Some(Shape::Thick) => (s(22), Shape::Thick),
            Some(Shape::Stuffed) => (s(14), Shape::Stuffed),
            _ => (s(9), Shape::Thin),
        };
        let crust_color = crust.map(|c| c.color).unwrap_or([ONE, ONE * 3 / 4, ONE / 2]);
        b.ink(
            Mesh::Cyl,
            &at.then(&Affine::translate(v3(0, crust_h / 2, 0)))
                .then(&Affine::scale(v3(2 * r, crust_h, 2 * r))),
            crust_color,
            STYLE_LIT,
        );
        if crust_shape == Shape::Stuffed {
            b.ink(
                Mesh::Ring,
                &at.then(&Affine::translate(v3(0, crust_h, 0)))
                    .then(&Affine::scale(v3(2 * r, s(16), 2 * r))),
                shade(crust_color, 108),
                STYLE_LIT,
            );
        }
        let sauce = chosen("sauce")
            .map(|c| c.color)
            .unwrap_or([ONE * 85 / 100, ONE / 4, ONE / 5]);
        let top = crust_h + s(2);
        b.plain(
            Mesh::Cyl,
            &at.then(&Affine::translate(v3(0, top - s(2), 0)))
                .then(&Affine::scale(v3(2 * r - s(22), s(5), 2 * r - s(22)))),
            sauce,
            STYLE_LIT,
        );
        for t in 0..def.layout.toppings as i32 {
            if pizza.tops & (1 << t) == 0 {
                continue;
            }
            let ing = &menu.toppings[t as usize];
            for k in 0..5 {
                let ang = deg(t * 41 + k * 72);
                let rad = s(22 + ((t + k) % 3) * 22);
                let spot = v3(fx::mul(rad, fx::cos(ang)), top + s(2), fx::mul(rad, fx::sin(ang)));
                let place = at.then(&Affine::translate(spot)).then(&Affine::rot_y(ang));
                let (mesh, size) = match ing.shape {
                    Shape::Disc => (Mesh::Cyl, v3(s(32), s(5), s(32))),
                    Shape::Dome => (Mesh::Sphere, v3(s(26), s(16), s(26))),
                    Shape::Ring => (Mesh::Ring, v3(s(18), s(8), s(18))),
                    Shape::Arc => (Mesh::Ring, v3(s(30), s(7), s(14))),
                    Shape::Cube => (Mesh::Cube, v3(s(15), s(12), s(15))),
                    Shape::Hoop => (Mesh::Ring, v3(s(28), s(5), s(28))),
                    Shape::Blob => (Mesh::Blob, v3(s(22), s(14), s(20))),
                    _ => (Mesh::Sphere, v3(s(42), s(7), s(11))),
                };
                b.plain(mesh, &place.then(&Affine::scale(size)), ing.color, STYLE_LIT);
            }
        }
    }

    fn masklings(&self, b: &mut Batch) {
        let robes = &self.c.masklings.robes;
        // two cooks at the counter
        for (k, side) in [-1i32, 1].into_iter().enumerate() {
            let sway = fx::mul(cm(3), fx::sin(phase(self.clock + k as i64 * 700, 1300)));
            let at = v3(side * cm(235), 0, COUNTER_Z + cm(10));
            self.maskling(b, at, -side * deg(20), robes[k % robes.len()], sway.abs(), None);
        }
        // one runner per drummer
        let n = self.on_stage();
        for d in 0..n {
            let home = match n {
                1 => v3(0, 0, cm(75)),
                _ => v3(cm(-100) + d as i32 * cm(200) / (n as i32 - 1), 0, cm(75)),
            };
            let robe = robes[(d + 2) % robes.len()];
            let run = self
                .play
                .as_ref()
                .filter(|_| self.mode == Mode::Play)
                .and_then(|p| p.seats.get(d)?.run.as_ref());
            let dest = v3(self.seat_x(d), 0, HANDOFF_Z);
            let idle = fx::mul(cm(2), fx::sin(phase(self.clock + d as i64 * 450, 1100)).abs());
            match run {
                Some(r) => {
                    let t = self.clock - r.at;
                    let (from, to, u, carrying) = if t < WALK_MS {
                        (home, dest, (t * ONE as i64 / WALK_MS) as i32, true)
                    } else if t < WALK_MS + TASTE_MS {
                        (dest, dest, ONE, false)
                    } else {
                        let back = t - WALK_MS - TASTE_MS;
                        (
                            dest,
                            home,
                            (back.min(WALK_MS) * ONE as i64 / WALK_MS) as i32,
                            false,
                        )
                    };
                    let pos = from.lerp(to, smooth(u));
                    let moving = u > 0 && u < ONE && from != to;
                    let hop = if moving {
                        fx::mul(cm(7), fx::sin(phase(t, 300)).abs())
                    } else {
                        idle
                    };
                    let dir = to.sub(from);
                    let yaw = if moving {
                        fx::atan2(dir.x, dir.z)
                    } else if from == to {
                        fx::HALF
                    } else {
                        0
                    };
                    let held = carrying.then_some(&r.pizza);
                    self.maskling(b, pos, yaw, robe, hop, held);
                }
                None => self.maskling(b, home, 0, robe, idle, None),
            }
        }
    }

    /// A Maskling standing at `at`, facing `yaw` (0 faces the audience),
    /// lifted by `hop`, holding a pizza overhead if given.
    fn maskling(&self, b: &mut Batch, at: V3, yaw: i32, robe: Rgb, hop: i32, held: Option<&Pizza>) {
        let m = &self.c.masklings;
        let body = Affine::translate(at.add(v3(0, hop, 0))).then(&Affine::rot_y(yaw));
        let part = |pos: V3, size: V3| body.then(&Affine::translate(pos)).then(&Affine::scale(size));
        b.ink(Mesh::Cone, &part(p(0, 34, 0), p(64, 68, 60)), robe, STYLE_LIT);
        b.ink(Mesh::Sphere, &part(p(0, 78, 0), p(50, 50, 48)), robe, STYLE_LIT);
        b.ink(
            Mesh::Sphere,
            &part(p(0, 78, 15), p(40, 44, 22)),
            m.mask,
            STYLE_LIT,
        );
        for ex in [-9, 9] {
            b.plain(Mesh::Sphere, &part(p(ex, 85, 26), p(8, 11, 4)), m.hole, STYLE_LIT);
        }
        b.plain(Mesh::Sphere, &part(p(0, 68, 26), p(9, 9, 4)), m.hole, STYLE_LIT);
        for fx_ in [-14, 14] {
            b.ink(Mesh::Blob, &part(p(fx_, 3, 6), p(16, 10, 22)), m.hole, STYLE_LIT);
        }
        let hand_y = if held.is_some() { 118 } else { 40 };
        for hx in [-30, 30] {
            b.ink(
                Mesh::Sphere,
                &part(p(hx, hand_y, 4), p(13, 13, 13)),
                robe,
                STYLE_LIT,
            );
        }
        if let Some(pz) = held {
            let base = body.then(&Affine::translate(p(0, 124, 0)));
            self.pizza(b, &base, cm(34), pz);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tests::{game, run_for};

    #[test]
    fn the_left_hand_takes_the_snare_unless_it_is_on_the_hi_hat() {
        let hits = vec![
            (0, Drum::Hat),
            (0, Drum::Snare),
            (2, Drum::Snare),
            (3, Drum::Crash),
            (3, Drum::Kick),
        ];
        let [l, r] = hands(&hits);
        assert_eq!(l, vec![(0, Drum::Hat), (2, Drum::Snare)]);
        assert_eq!(r, vec![(0, Drum::Snare), (3, Drum::Crash)]);
    }

    #[test]
    fn on_a_hit_the_sticks_tip_is_on_the_drum() {
        let g = game();
        let hits = vec![(4, Drum::Snare)];
        let rest = v3(0, 0, 0);
        let step = 100;
        let hand = g.hand_at(0, &hits, step, 400, rest);
        assert_eq!(hand.add(stick()), kit_top(0, Drum::Snare));
        let before = g.hand_at(0, &hits, step, 300, rest);
        assert_ne!(before.add(stick()), kit_top(0, Drum::Snare));
    }

    #[test]
    fn the_scene_draws_more_while_a_pizza_is_carried_and_tasted() {
        let mut g = game();
        g.act("level.choose", "0").unwrap();
        g.act("level.start", "").unwrap();
        g.act("pizza.top", "0").unwrap();
        let resting = g.scene().count();
        g.act("serve", "0").unwrap();
        run_for(&mut g, WALK_MS / 2);
        let carrying = g.scene().count();
        assert!(
            carrying > resting,
            "{carrying} > {resting}: the carried pizza is drawn"
        );
    }

    #[test]
    fn a_fed_drummer_stands_in_a_spotlight() {
        let mut g = game();
        g.act("level.choose", "0").unwrap();
        g.act("level.start", "").unwrap();
        let tints = |g: &mut Game| {
            let (_, draws) = g.scene().finish();
            draws
                .chunks(4)
                .filter(|d| d[3] == crate::render::PASS_TINT)
                .count()
        };
        assert_eq!(tints(&mut g), 0);
        g.play.as_mut().unwrap().seats[0].fed = true;
        assert_eq!(tints(&mut g), 1);
    }
}
