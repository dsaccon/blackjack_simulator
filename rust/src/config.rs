// src/config.rs

pub const NUM_DECKS: usize = 6;
pub const RESHUFFLE_THRESHOLD_RATIO: f64 = 0.25;
pub const NUM_PLAYERS: usize = 3;

pub const STARTING_BALANCE: f64 = 1000.00;
pub const DEFAULT_BET: f64 = 25.00;
pub const MIN_BET_ALLOWED: f64 = 1.00;

pub const BLACKJACK_PAYOUT_NUMERATOR: f64 = 6.0;
pub const BLACKJACK_PAYOUT_DENOMINATOR: f64 = 5.0;
pub const BLACKJACK_PAYOUT_MULTIPLIER: f64 = BLACKJACK_PAYOUT_NUMERATOR / BLACKJACK_PAYOUT_DENOMINATOR;

pub const DEFAULT_SIM_ITERATIONS: u32 = 1000;

pub const LOGS_DIR_NAME: &str = "logs";
pub const TEXT_LOG_FILENAME: &str = "results.log";
pub const GRAPH_EXTENSION: &str = "png";
