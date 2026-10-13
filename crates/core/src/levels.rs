//! The levels' promise: every order of every level can be worked out from
//! the hints alone within the level's patience. The solver (`solver.rs`)
//! sees only the hints, and the tests below play it against every order a
//! level allows.

use crate::content::LevelDef;
use crate::solver::worst_case;

/// The most pizzas the solver needs to finish every drummer of a level,
/// whatever their orders: each drummer's worst case, added up.
pub fn solver_needs(level: &LevelDef) -> u32 {
    level.drummers as u32 * worst_case(&level.layout, level.clarity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{tests::files, Content};
    use crate::solver::{every_order, play};

    fn levels() -> Vec<LevelDef> {
        Content::load(&files()).unwrap().levels
    }

    #[test]
    fn the_solver_finishes_every_order_of_every_level_within_its_patience() {
        for (k, l) in levels().iter().enumerate() {
            let worst = worst_case(&l.layout, l.clarity);
            let results = every_order(&l.layout, l.clarity);
            let orders = l.layout.orders();
            assert_eq!(
                results.len(),
                orders.len(),
                "level {}: every order is reached",
                k + 1
            );
            for o in &orders {
                let n = results.iter().find(|(r, _)| r == o).map(|(_, n)| *n);
                assert!(
                    n.map(|n| n <= worst).unwrap_or(false),
                    "level {}: order {o:?} took {n:?}",
                    k + 1
                );
            }
            assert!(
                solver_needs(l) <= l.patience,
                "level {} \"{}\": the solver may need {} pizzas, patience is {}",
                k + 1,
                l.name,
                solver_needs(l),
                l.patience
            );
        }
    }

    #[test]
    fn the_solver_playing_from_hints_matches_its_tree_on_the_first_four_levels() {
        for l in levels().iter().take(4) {
            let worst = worst_case(&l.layout, l.clarity);
            for (o, n) in every_order(&l.layout, l.clarity) {
                assert_eq!(play(&l.layout, l.clarity, &o, worst), Some(n), "{o:?}");
            }
        }
    }

    #[test]
    fn no_level_is_easier_for_the_solver_than_the_one_before() {
        let needs: Vec<u32> = levels().iter().map(solver_needs).collect();
        for w in needs.windows(2) {
            assert!(w[0] <= w[1], "{needs:?}");
        }
        assert!(needs[0] < needs[needs.len() - 1]);
    }

    #[test]
    fn the_band_grows_from_one_drummer_to_four_and_the_menu_from_toppings_to_two_layers() {
        let ls = levels();
        assert_eq!(ls[0].drummers, 1);
        assert_eq!(ls[ls.len() - 1].drummers, 4);
        assert!(ls[0].layers.is_empty());
        assert_eq!(ls[ls.len() - 1].layers.len(), 2);
        assert!(ls[0].layout.toppings < ls[ls.len() - 1].layout.toppings);
    }
}
