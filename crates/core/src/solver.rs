//! A player that sees only the hints. It keeps every order that agrees with
//! every hint so far, and serves the pizza whose worst hint leaves the fewest
//! orders standing. Serving any order that is still possible always rules
//! out at least that one when it is wrong, so the solver always finishes.
//!
//! `worst_case` builds the whole tree of the solver's play over every order a
//! layout allows and returns its depth: the most pizzas it ever needs. The
//! levels' patience is checked against it (`levels.rs`).

use crate::puzzle::{judge, Clarity, Feedback, Layout, Pizza, Verdict, CODES};

/// The orders still possible after the hints so far.
pub fn consistent(orders: &[Pizza], history: &[(Pizza, Feedback)], clarity: Clarity) -> Vec<Pizza> {
    orders
        .iter()
        .filter(|o| history.iter().all(|(p, f)| judge(o, p, clarity) == *f))
        .cloned()
        .collect()
}

/// The size of the largest group of `orders` that would give the same hint.
fn worst_split(
    orders: &[Pizza],
    pizza: &Pizza,
    clarity: Clarity,
    counts: &mut [u32],
    seen: &mut Vec<usize>,
) -> u32 {
    let mut worst = 0;
    for o in orders {
        let code = judge(o, pizza, clarity).code();
        if counts[code] == 0 {
            seen.push(code);
        }
        counts[code] += 1;
        worst = worst.max(counts[code]);
    }
    for code in seen.drain(..) {
        counts[code] = 0;
    }
    worst
}

/// The pizza to serve next. With one order left, that order. Otherwise the
/// pizza whose worst hint leaves the fewest orders; among equals, one that
/// could itself be the order, then the first in the menu's order.
pub fn choose(orders: &[Pizza], pizzas: &[Pizza], clarity: Clarity) -> Pizza {
    if orders.len() == 1 {
        return orders[0].clone();
    }
    let mut counts = vec![0u32; CODES];
    let mut seen = Vec::new();
    let mut best: Option<(u32, bool, &Pizza)> = None;
    for p in pizzas {
        let w = worst_split(orders, p, clarity, &mut counts, &mut seen);
        let possible = orders.contains(p);
        let better = match best {
            None => true,
            Some((bw, bp, _)) => w < bw || (w == bw && possible && !bp),
        };
        if better {
            best = Some((w, possible, p));
        }
    }
    best.map(|(_, _, p)| p.clone())
        .unwrap_or_else(|| orders[0].clone())
}

/// The most pizzas the solver serves to finish any one order of `layout`,
/// the last, perfect one included.
pub fn worst_case(layout: &Layout, clarity: Clarity) -> u32 {
    let pizzas = layout.pizzas();
    depth(&layout.orders(), &pizzas, clarity)
}

fn depth(orders: &[Pizza], pizzas: &[Pizza], clarity: Clarity) -> u32 {
    if orders.len() <= 1 {
        return 1;
    }
    let p = choose(orders, pizzas, clarity);
    let mut groups: Vec<(Feedback, Vec<Pizza>)> = Vec::new();
    for o in orders {
        let f = judge(o, &p, clarity);
        match groups.iter_mut().find(|(g, _)| *g == f) {
            Some((_, v)) => v.push(o.clone()),
            None => groups.push((f, vec![o.clone()])),
        }
    }
    groups
        .iter()
        .map(|(f, g)| match f.verdict {
            Verdict::Perfect => 1,
            _ => 1 + depth(g, pizzas, clarity),
        })
        .max()
        .unwrap_or(1)
}

/// Every order of `layout` with the number of pizzas the solver serves to
/// finish it, from one walk of the solver's tree.
pub fn every_order(layout: &Layout, clarity: Clarity) -> Vec<(Pizza, u32)> {
    let pizzas = layout.pizzas();
    let mut out = Vec::new();
    walk(&layout.orders(), &pizzas, clarity, 1, &mut out);
    out
}

fn walk(orders: &[Pizza], pizzas: &[Pizza], clarity: Clarity, served: u32, out: &mut Vec<(Pizza, u32)>) {
    if orders.len() == 1 {
        out.push((orders[0].clone(), served));
        return;
    }
    let p = choose(orders, pizzas, clarity);
    let mut groups: Vec<(Feedback, Vec<Pizza>)> = Vec::new();
    for o in orders {
        let f = judge(o, &p, clarity);
        match groups.iter_mut().find(|(g, _)| *g == f) {
            Some((_, v)) => v.push(o.clone()),
            None => groups.push((f, vec![o.clone()])),
        }
    }
    for (f, g) in groups {
        if f.verdict == Verdict::Perfect {
            out.push((g[0].clone(), served));
        } else {
            walk(&g, pizzas, clarity, served + 1, out);
        }
    }
}

/// Plays one order to the end from the hints alone, as the game would see
/// it, and returns how many pizzas it took.
pub fn play(layout: &Layout, clarity: Clarity, order: &Pizza, limit: u32) -> Option<u32> {
    let all = layout.orders();
    let pizzas = layout.pizzas();
    let mut history: Vec<(Pizza, Feedback)> = Vec::new();
    for n in 1..=limit {
        let left = consistent(&all, &history, clarity);
        let p = choose(&left, &pizzas, clarity);
        let f = judge(order, &p, clarity);
        if f.verdict == Verdict::Perfect {
            return Some(n);
        }
        history.push((p, f));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> Layout {
        Layout {
            picks: vec![2],
            toppings: 3,
            min: 1,
            max: 2,
        }
    }

    #[test]
    fn hints_rule_out_every_order_that_disagrees() {
        let l = small();
        let order = Pizza {
            picks: vec![1],
            tops: 0b011,
        };
        let served = Pizza {
            picks: vec![1],
            tops: 0b001,
        };
        let f = judge(&order, &served, Clarity::Vague);
        let left = consistent(&l.orders(), &[(served.clone(), f)], Clarity::Vague);
        // close on sauce 1 with topping 0: sauce 1, topping 0 and one more
        assert_eq!(left.len(), 2);
        assert!(left.contains(&order));
        // then a miss on sauce 1 with toppings 0 and 2 rules out {0, 2}
        let second = Pizza {
            picks: vec![1],
            tops: 0b101,
        };
        let g = judge(&order, &second, Clarity::Vague);
        let left = consistent(&l.orders(), &[(served, f), (second, g)], Clarity::Vague);
        assert_eq!(left, vec![order]);
    }

    #[test]
    fn with_one_order_left_the_solver_serves_it() {
        let only = Pizza {
            picks: vec![0],
            tops: 0b100,
        };
        assert_eq!(
            choose(std::slice::from_ref(&only), &small().pizzas(), Clarity::Vague),
            only
        );
    }

    #[test]
    fn the_solver_wins_every_order_of_a_small_layout_within_its_worst_case() {
        for clarity in [Clarity::Named, Clarity::Counted, Clarity::Vague] {
            let l = small();
            let worst = worst_case(&l, clarity);
            let mut most = 0;
            for o in l.orders() {
                let n = play(&l, clarity, &o, worst).expect("the solver finishes within its worst case");
                most = most.max(n);
            }
            assert_eq!(
                most, worst,
                "{clarity:?}: the tree's depth is reached by some order"
            );
        }
    }

    #[test]
    fn named_hints_need_no_more_pizzas_than_vague_ones() {
        let l = Layout {
            picks: vec![3],
            toppings: 4,
            min: 1,
            max: 2,
        };
        let named = worst_case(&l, Clarity::Named);
        let vague = worst_case(&l, Clarity::Vague);
        assert!(named <= vague, "{named} {vague}");
    }

    #[test]
    fn one_topping_and_nothing_else_takes_at_most_two_pizzas_with_named_hints() {
        // Worked by hand: orders {a}, {b}. Serve {a}: perfect, or a miss
        // naming a (so the order is {b}), and {b} is served next.
        let l = Layout {
            picks: vec![],
            toppings: 2,
            min: 1,
            max: 1,
        };
        assert_eq!(worst_case(&l, Clarity::Named), 2);
    }
}
