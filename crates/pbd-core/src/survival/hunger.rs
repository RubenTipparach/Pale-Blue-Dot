//! Hunger (`survival-mode` design 4): a food bar that play time empties and
//! meals fill. Time with the game closed is no play time, so it never
//! starves anyone. The buffs stay `cooking`'s, in both modes.

use super::config::SurvivalConfig;

/// Food a second of play burns standing about, fp/s; times
/// `food.cold_mult` while the body shivers (design 14).
pub fn idle_burn_fp_per_s(cfg: &SurvivalConfig, shivering: bool) -> f32 {
    let base = cfg.food.idle_fp_per_s();
    if shivering {
        base * cfg.food.cold_mult
    } else {
        base
    }
}

/// A tree's crop that is not eaten: the pine cone is wood (`values.ron`).
const NOT_EATEN: [&str; 1] = ["pine-cone"];

/// The code a good is eaten as in `survival.ron`'s tables: every tree fruit
/// (`flora::FRUIT_GOODS`, less the pine cone) is the one `fruit` row;
/// anything else is itself.
pub fn food_code(good: &str) -> &str {
    if crate::flora::FRUIT_GOODS.contains(&good) && !NOT_EATEN.contains(&good) {
        "fruit"
    } else {
        good
    }
}

/// Whether `good` is eaten for food or water in Survival.
pub fn is_eaten(cfg: &SurvivalConfig, good: &str) -> bool {
    let code = food_code(good);
    cfg.fills.contains_key(code) || cfg.drinks.eaten.contains_key(code)
}

/// Food a good fills when eaten at `stars` (1 to 3; a raw food is eaten at
/// one), or `None` for a good that is not eaten for food.
pub fn fills(cfg: &SurvivalConfig, good: &str, stars: u8) -> Option<f32> {
    let one = *cfg.fills.get(food_code(good))?;
    let i = usize::from(stars.clamp(1, 3) - 1);
    Some(one * cfg.star_fill_mult[i])
}

/// Water a good gives when eaten (milk, fruit, stew), or 0.
pub fn eaten_water(cfg: &SurvivalConfig, good: &str) -> f32 {
    cfg.drinks
        .eaten
        .get(food_code(good))
        .copied()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roast(stars: u8) -> f32 {
        fills(&SurvivalConfig::shipped(), "roast-meat", stars).unwrap()
    }

    #[test]
    fn a_one_star_meal_fills_its_own_amount() {
        assert!((roast(1) - 35.0).abs() < 1e-4);
        let cfg = SurvivalConfig::shipped();
        assert!((fills(&cfg, "raw-meat", 1).unwrap() - 8.0).abs() < 1e-6);
        // Raw fish, and anything not a food, fills nothing.
        assert_eq!(fills(&cfg, "stone", 1), None);
    }

    #[test]
    fn every_tree_fruit_is_eaten_as_fruit() {
        let cfg = SurvivalConfig::shipped();
        for fruit in crate::flora::FRUIT_GOODS
            .iter()
            .filter(|f| **f != "pine-cone")
        {
            assert_eq!(food_code(fruit), "fruit");
            assert_eq!(fills(&cfg, fruit, 1), Some(6.0), "{fruit}");
            assert_eq!(eaten_water(&cfg, fruit), 4.0, "{fruit}");
            assert!(is_eaten(&cfg, fruit));
        }
        assert!(!is_eaten(&cfg, "pine-cone"));
        assert_eq!(food_code("raw-meat"), "raw-meat");
        assert!(is_eaten(&cfg, "milk"));
        assert!(is_eaten(&cfg, "raw-meat"));
        assert!(!is_eaten(&cfg, "stone"));
    }

    #[test]
    fn a_two_star_meal_fills_115_percent() {
        assert!((roast(2) - 40.25).abs() < 1e-4);
        assert!(roast(1) < roast(2));
    }

    #[test]
    fn a_three_star_meal_fills_130_percent() {
        assert!((roast(3) - 45.5).abs() < 1e-4);
        assert!(roast(2) < roast(3));
    }

    #[test]
    fn standing_about_a_full_bar_lasts_75_minutes_and_shivering_burns_faster() {
        let cfg = SurvivalConfig::shipped();
        let rate = idle_burn_fp_per_s(&cfg, false);
        assert!((cfg.food.max_fp / rate / 60.0 - 75.0).abs() < 1e-3);
        assert!((idle_burn_fp_per_s(&cfg, true) / rate - 1.5).abs() < 1e-5);
    }

    #[test]
    fn stew_and_milk_give_water() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(eaten_water(&cfg, "meat-stew"), 20.0);
        assert_eq!(eaten_water(&cfg, "milk"), 15.0);
        assert_eq!(eaten_water(&cfg, "flatbread"), 0.0);
    }
}
