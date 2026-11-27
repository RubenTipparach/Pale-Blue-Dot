//! Survival mode's rules (`openspec/changes/survival-mode`): health, falls,
//! hunger, water and body temperature, stamina and its food cost, breath and
//! suit air, heat stroke, tool wear, weapons and hunting. Pure functions of
//! state, seconds of play and events, so the in-process server ticks them
//! for each player and a client predicts with the same code (design 10).
//! Every number comes from `assets/config/survival.ron` ([`SurvivalConfig`]).
//! None of it applies in an Explorer world ([`WorldMode`]).

pub mod body;
pub mod breath;
pub mod config;
pub mod fall;
pub mod health;
pub mod hunger;
pub mod hunting;
pub mod mode;
pub mod passout;
pub mod stamina;
pub mod temperature;
pub mod water;
pub mod weapons;
pub mod wear;

pub use body::{Conditions, Event, Shown, Tick, Vitals};
pub use config::SurvivalConfig;
pub use mode::WorldMode;
pub use water::{Note, WaterSource};
