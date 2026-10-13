//! The deduction rule: what a drummer wants, what a pizza is, and what he
//! says back when he tastes one.
//!
//! A pizza has one choice in each pick-one layer (crust, sauce) and any set
//! of toppings. An order is a pizza with between `min` and `max` toppings.
//! A served pizza is judged against the order:
//!
//! - anything on it the drummer does not want (a wrong crust, a wrong sauce,
//!   a topping not in the order) makes it a **miss**;
//! - otherwise, a topping the order has and the pizza lacks makes it **close**;
//! - otherwise it is **perfect**.
//!
//! How much more the hint says depends on the level's clarity: a named hint
//! counts the wrong things and names the first of them (layers first, then
//! toppings, in menu order), or counts what is missing; a counted hint
//! counts the wrong things or the missing toppings; a vague hint says only
//! miss, close or perfect. Each clarity says everything the next one says.

use serde::Deserialize;

/// How much a hint says beyond its verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Clarity {
    Named,
    Counted,
    Vague,
}

/// The shape of a level's menu: how many options each pick-one layer has,
/// how many toppings there are, and how many toppings an order holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub picks: Vec<u8>,
    pub toppings: u8,
    pub min: u8,
    pub max: u8,
}

/// A pizza: one option per pick-one layer, and toppings as a bit set.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pizza {
    pub picks: Vec<u8>,
    pub tops: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Perfect,
    Close,
    Miss,
}

/// One thing on a pizza: an option in a pick-one layer, or a topping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    Pick { layer: u8, option: u8 },
    Topping(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Detail {
    None,
    Count(u8),
    /// the first wrong thing, and how many wrong things there are
    Named(Item, u8),
}

/// What a drummer says about one pizza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Feedback {
    pub verdict: Verdict,
    pub detail: Detail,
}

impl Feedback {
    /// A small whole number that differs for different feedback, for
    /// counting how a set of orders splits. Below `CODES`.
    pub fn code(&self) -> usize {
        let v = self.verdict as usize;
        let item = |i: Item| match i {
            Item::Pick { layer, option } => layer as usize * 8 + option as usize,
            Item::Topping(t) => 32 + t as usize,
        };
        let d = match self.detail {
            Detail::None => 0,
            Detail::Count(n) => 1 + n as usize,
            Detail::Named(i, n) => 64 + item(i) * 32 + n as usize,
        };
        v * PER_VERDICT + d
    }
}

const PER_VERDICT: usize = 64 + 48 * 32;
/// Every `Feedback::code` is below this.
pub const CODES: usize = 3 * PER_VERDICT;

/// The most toppings a layout may have (the bit set is a `u32`, and named
/// hints index toppings below 48).
pub const MOST_TOPPINGS: u8 = 16;

impl Layout {
    pub fn check(&self) -> Result<(), String> {
        if self.toppings == 0 || self.toppings > MOST_TOPPINGS {
            return Err(format!(
                "A level has {} toppings; it needs between 1 and {MOST_TOPPINGS}.",
                self.toppings
            ));
        }
        if self.min == 0 || self.min > self.max || self.max > self.toppings {
            return Err(format!(
                "A level's orders hold {} to {} toppings, which does not fit {} toppings.",
                self.min, self.max, self.toppings
            ));
        }
        if self.picks.iter().any(|n| !(2..=8).contains(n)) || self.picks.len() > 4 {
            return Err(
                "A pick-one layer needs between 2 and 8 options, and a level at most 4 such layers.".into(),
            );
        }
        Ok(())
    }

    fn pick_combos(&self) -> Vec<Vec<u8>> {
        let mut combos = vec![Vec::new()];
        for &n in &self.picks {
            combos = combos
                .into_iter()
                .flat_map(|c| {
                    (0..n).map(move |o| {
                        let mut c = c.clone();
                        c.push(o);
                        c
                    })
                })
                .collect();
        }
        combos
    }

    /// Every order a drummer may hold, in a fixed order.
    pub fn orders(&self) -> Vec<Pizza> {
        let mut out = Vec::new();
        for picks in self.pick_combos() {
            for tops in 0..(1u32 << self.toppings) {
                let n = tops.count_ones() as u8;
                if n >= self.min && n <= self.max {
                    out.push(Pizza {
                        picks: picks.clone(),
                        tops,
                    });
                }
            }
        }
        out
    }

    /// Every pizza the player can serve, in a fixed order.
    pub fn pizzas(&self) -> Vec<Pizza> {
        let mut out = Vec::new();
        for picks in self.pick_combos() {
            for tops in 0..(1u32 << self.toppings) {
                out.push(Pizza {
                    picks: picks.clone(),
                    tops,
                });
            }
        }
        out
    }
}

/// The drummer holding `order` tastes `pizza`.
pub fn judge(order: &Pizza, pizza: &Pizza, clarity: Clarity) -> Feedback {
    let wrong_picks = order
        .picks
        .iter()
        .zip(&pizza.picks)
        .filter(|(a, b)| a != b)
        .count();
    let extra = pizza.tops & !order.tops;
    let wrong = wrong_picks + extra.count_ones() as usize;
    let missing = (order.tops & !pizza.tops).count_ones() as u8;
    if wrong > 0 {
        let detail = match clarity {
            Clarity::Named => {
                let first_pick = order
                    .picks
                    .iter()
                    .zip(&pizza.picks)
                    .position(|(a, b)| a != b)
                    .map(|l| Item::Pick {
                        layer: l as u8,
                        option: pizza.picks[l],
                    });
                Detail::Named(
                    first_pick.unwrap_or(Item::Topping(extra.trailing_zeros() as u8)),
                    wrong as u8,
                )
            }
            Clarity::Counted => Detail::Count(wrong as u8),
            Clarity::Vague => Detail::None,
        };
        return Feedback {
            verdict: Verdict::Miss,
            detail,
        };
    }
    if missing > 0 {
        let detail = match clarity {
            Clarity::Vague => Detail::None,
            _ => Detail::Count(missing),
        };
        return Feedback {
            verdict: Verdict::Close,
            detail,
        };
    }
    Feedback {
        verdict: Verdict::Perfect,
        detail: Detail::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pz(picks: &[u8], tops: u32) -> Pizza {
        Pizza {
            picks: picks.to_vec(),
            tops,
        }
    }

    #[test]
    fn a_wrong_thing_outranks_a_missing_one() {
        // order: sauce 1, toppings 0 and 2; pizza: sauce 1, toppings 1 and 2
        let f = judge(&pz(&[1], 0b101), &pz(&[1], 0b110), Clarity::Counted);
        assert_eq!(f.verdict, Verdict::Miss);
        assert_eq!(f.detail, Detail::Count(1));
    }

    #[test]
    fn a_subset_is_close_and_counts_what_is_missing() {
        let f = judge(&pz(&[], 0b1011), &pz(&[], 0b0001), Clarity::Named);
        assert_eq!(
            f,
            Feedback {
                verdict: Verdict::Close,
                detail: Detail::Count(2)
            }
        );
        let v = judge(&pz(&[], 0b1011), &pz(&[], 0b0001), Clarity::Vague);
        assert_eq!(v.detail, Detail::None);
    }

    #[test]
    fn the_exact_pizza_is_perfect() {
        let f = judge(&pz(&[2, 0], 0b11), &pz(&[2, 0], 0b11), Clarity::Vague);
        assert_eq!(f.verdict, Verdict::Perfect);
    }

    #[test]
    fn a_named_miss_names_the_first_wrong_layer_before_any_topping() {
        // crust right, sauce wrong (pizza has option 2), topping 3 extra
        let f = judge(&pz(&[0, 1], 0b0001), &pz(&[0, 2], 0b1001), Clarity::Named);
        assert_eq!(f.detail, Detail::Named(Item::Pick { layer: 1, option: 2 }, 2));
        // layers right: the lowest extra topping is named
        let g = judge(&pz(&[0, 1], 0b0001), &pz(&[0, 1], 0b1101), Clarity::Named);
        assert_eq!(g.detail, Detail::Named(Item::Topping(2), 2));
    }

    #[test]
    fn a_counted_miss_counts_wrong_layers_and_extra_toppings_together() {
        let f = judge(&pz(&[0, 1], 0b0001), &pz(&[1, 2], 0b1100), Clarity::Counted);
        assert_eq!(f.detail, Detail::Count(4));
    }

    #[test]
    fn orders_hold_between_min_and_max_toppings_with_every_layer_combination() {
        let l = Layout {
            picks: vec![3],
            toppings: 4,
            min: 1,
            max: 2,
        };
        // 3 sauces × (4 singles + 6 pairs)
        assert_eq!(l.orders().len(), 30);
        // 3 sauces × 16 topping sets, the empty one included
        assert_eq!(l.pizzas().len(), 48);
    }

    #[test]
    fn different_feedback_has_different_codes_below_the_limit() {
        let mut all = Vec::new();
        for verdict in [Verdict::Perfect, Verdict::Close, Verdict::Miss] {
            let mut details = vec![Detail::None];
            details.extend((0..32).map(Detail::Count));
            for n in 0..32 {
                for layer in 0..4 {
                    for option in 0..8 {
                        details.push(Detail::Named(Item::Pick { layer, option }, n));
                    }
                }
                details.extend((0..MOST_TOPPINGS).map(|t| Detail::Named(Item::Topping(t), n)));
            }
            for detail in details {
                all.push(Feedback { verdict, detail }.code());
            }
        }
        assert!(all.iter().all(|&c| c < CODES));
        let mut sorted = all.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), all.len());
    }

    #[test]
    fn a_layout_out_of_bounds_is_refused() {
        let ok = Layout {
            picks: vec![3],
            toppings: 5,
            min: 1,
            max: 3,
        };
        assert!(ok.check().is_ok());
        assert!(Layout { min: 4, ..ok.clone() }.check().is_err());
        assert!(Layout { max: 6, ..ok.clone() }.check().is_err());
        assert!(Layout {
            picks: vec![1],
            ..ok.clone()
        }
        .check()
        .is_err());
        assert!(Layout { toppings: 17, ..ok }.check().is_err());
    }
}
