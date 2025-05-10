// src/stats.rs
use crate::config; // For DEFAULT_BET
use crate::config::{LOGS_DIR_NAME, TEXT_LOG_FILENAME};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub run_id: u64,
    pub mode: String,
    pub target_iterations: Option<u32>, // Only for simulation
    pub hands_played_session: u32,      // "Your" main hands
    pub blackjacks_dealt_player: u32,   // "Your" blackjacks

    pub times_split_chosen: u32,        // "Your" splits
    pub hands_involved_in_split: u32,   // "Your" original hands that were split
    pub total_hands_after_splits: u32,  // "Your" total resulting hands from splits

    pub times_doubled_chosen: u32,      // "Your" doubles
    pub hands_involved_in_double: u32,  // "Your" hands (original or split) that were doubled

    pub total_wins: u32,                // "Your" wins
    pub total_losses: u32,              // "Your" losses
    pub total_pushes: u32,              // "Your" pushes

    pub earnings_from_split_hands: f64, // "Your" P/L from hands that were part of a split
    pub num_resolved_split_hands: u32,  // "Your" count of individual hands resolved in a split

    pub earnings_from_doubled_hands: f64,// "Your" P/L from hands that were doubled
    pub num_resolved_doubled_hands: u32, // "Your" count of individual hands doubled & resolved

    pub initial_default_bet: f64,
    pub initial_balance: f64,
    pub final_balance: f64,
    pub highest_balance_session: f64,
    pub lowest_balance_session: f64,

    // Calculated at the end
    pub net_profit_loss: f64,
    pub avg_earn_loss_per_main_hand: f64,
    pub avg_earn_loss_per_split_hand_part: f64,
    pub avg_earn_loss_per_doubled_hand: f64,

    pub total_script_runtime_seconds: f64,
    pub avg_time_per_hand_seconds: f64,
}

impl SessionStats {
    pub fn new(run_id: u64, mode_str: String, start_bal: f64) -> Self {
        SessionStats {
            run_id,
            mode: mode_str,
            target_iterations: None,
            hands_played_session: 0,
            blackjacks_dealt_player: 0,
            times_split_chosen: 0,
            hands_involved_in_split: 0,
            total_hands_after_splits: 0,
            times_doubled_chosen: 0,
            hands_involved_in_double: 0,
            total_wins: 0,
            total_losses: 0,
            total_pushes: 0,
            earnings_from_split_hands: 0.0,
            num_resolved_split_hands: 0,
            earnings_from_doubled_hands: 0.0,
            num_resolved_doubled_hands: 0,
            initial_default_bet: config::DEFAULT_BET,
            initial_balance: start_bal,
            final_balance: start_bal, // Will be updated
            highest_balance_session: start_bal,
            lowest_balance_session: start_bal,
            net_profit_loss: 0.0,
            avg_earn_loss_per_main_hand: 0.0,
            avg_earn_loss_per_split_hand_part: 0.0,
            avg_earn_loss_per_doubled_hand: 0.0,
            total_script_runtime_seconds: 0.0,
            avg_time_per_hand_seconds: 0.0,
        }
    }

    pub fn calculate_final_metrics(&mut self) {
        self.net_profit_loss = self.final_balance - self.initial_balance;
        if self.hands_played_session > 0 {
            self.avg_earn_loss_per_main_hand = self.net_profit_loss / self.hands_played_session as f64;
        } else {
            self.avg_earn_loss_per_main_hand = 0.0;
        }
        if self.num_resolved_split_hands > 0 {
            self.avg_earn_loss_per_split_hand_part = self.earnings_from_split_hands / self.num_resolved_split_hands as f64;
        } else {
            self.avg_earn_loss_per_split_hand_part = 0.0;
        }
        if self.num_resolved_doubled_hands > 0 {
            self.avg_earn_loss_per_doubled_hand = self.earnings_from_doubled_hands / self.num_resolved_doubled_hands as f64;
        } else {
            self.avg_earn_loss_per_doubled_hand = 0.0;
        }
    }

    pub fn update_balance_extremes(&mut self, current_balance: f64) {
        if current_balance > self.highest_balance_session {
            self.highest_balance_session = current_balance;
        }
        if current_balance < self.lowest_balance_session {
            self.lowest_balance_session = current_balance;
        }
    }

    pub fn to_log_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Run ID: {}", self.run_id),
            format!("Mode: {}", self.mode),
        ];

        lines.push(format!("Total Script Runtime: {:.3} seconds", self.total_script_runtime_seconds));
        lines.push(format!("Average Time per Main Hand (You): {:.4} seconds", self.avg_time_per_hand_seconds));

        if let Some(iters) = self.target_iterations {
            lines.push(format!("Target Iterations: {}", iters));
        }
        lines.extend(vec![
            format!("Hands Played by You (Main): {}", self.hands_played_session),
            format!("Starting Balance (You): ${:.2}", self.initial_balance),
            format!("Final Balance (You):    ${:.2}", self.final_balance),
            format!("Highest Balance (You): ${:.2}", self.highest_balance_session),
            format!("Lowest Balance (You):  ${:.2}", self.lowest_balance_session),
            format!("Default Bet Used (You): ${:.2}", self.initial_default_bet),
            format!("Net Profit/Loss (You):  ${:+.2}", self.net_profit_loss),
            format!("Avg. P/L per Main Hand (You): ${:+.2}", self.avg_earn_loss_per_main_hand),
            format!("Your Blackjacks: {}", self.blackjacks_dealt_player),
            format!("Your Times 'Split' Chosen: {}", self.times_split_chosen),
            format!("Your Original Hands Involving a Split: {}", self.hands_involved_in_split),
            format!("Your Total Individual Hands from Splits: {}", self.total_hands_after_splits),
            format!("Your Net P/L from All Split Hand Parts: ${:+.2}", self.earnings_from_split_hands),
            format!("Your Avg. P/L per Individual Split Hand Part: ${:+.2} (from {} parts)",
                    self.avg_earn_loss_per_split_hand_part, self.num_resolved_split_hands),
            format!("Your Times 'Double Down' Chosen: {}", self.times_doubled_chosen),
            format!("Your Hands Involving a Double Down: {}", self.hands_involved_in_double),
            format!("Your Net P/L from Doubled Hands: ${:+.2}", self.earnings_from_doubled_hands),
            format!("Your Avg. P/L per Doubled Hand: ${:+.2} (from {} hands)",
                    self.avg_earn_loss_per_doubled_hand, self.num_resolved_doubled_hands),
            format!("Your Total Wins: {}, Losses: {}, Pushes: {}",
                    self.total_wins, self.total_losses, self.total_pushes),
        ]);
        if self.mode.contains("Simulation") {
             lines.push(format!("Strategy for All AI Players: 'Book' (H17, {}D, DAS based)", config::NUM_DECKS));
        }

        // Add timing information if they have been set (i.e., not 0.0)
        if self.total_script_runtime_seconds > 0.0 {
             lines.push(format!("Total Script Runtime: {:.3} seconds", self.total_script_runtime_seconds));
        }
        if self.avg_time_per_hand_seconds > 0.0 && self.hands_played_session > 0 {
             lines.push(format!("Average Time per Main Hand (You): {:.4} seconds", self.avg_time_per_hand_seconds));
        }
        lines
    }
}

//// Logging setup (can be moved to its own module or main.rs)
//pub fn setup_logger(run_id: u64) -> Result<(), Box<dyn std::error::Error>> {
//    use crate::config::{LOGS_DIR_NAME, TEXT_LOG_FILENAME};
//    use std::fs;
//    use std::path::PathBuf;
//
//    let logs_dir = PathBuf::from(LOGS_DIR_NAME);
//    fs::create_dir_all(&logs_dir)?;
//
//    let log_file_path = logs_dir.join(TEXT_LOG_FILENAME);
//
//    // Using simple_logger here. You can swap for fern or env_logger for more control.
//    // Note: simple_logger writes to stderr by default AND a file if configured.
//    // To only write to file, you might need a more configurable logger or custom setup.
//    // For this example, we'll let it write to stderr too for immediate feedback during dev.
//    simple_logger::SimpleLogger::new()
//        .with_level(log::LevelFilter::Info)
//        .with_timestamp_format(time::macros::format_description!("%Y-%m-%d %H:%M:%S")) // Requires time crate features
//        .with_custom_ext_formatter(move |f, record, default_color_config| { // Custom formatter for run_id
//            let time_str = time::OffsetDateTime::now_utc()
//                .format(&time::format_description::well_known::Rfc3339)
//                .unwrap_or_else(|_| "NO_TIME".to_string());
//             write!(f, "{} - RUN_ID:{} - {} - {}\n",
//                time_str,
//                run_id,
//                record.level(),
//                record.args()
//            )
//        })
//        .init()?; // Initialize globally
//    
//    // If you want a dedicated file logger without also printing to stderr with simple_logger,
//    // you'd use something like 'fern' or build a custom dispatcher with 'log' crate.
//    // For now, this is a simple approach. To log to file *only* with 'log' crate,
//    // you'd typically implement the `log::Log` trait.
//
//    // Alternative simple file logging if simple_logger is too verbose on stderr:
//    // You could create a global `Mutex<File>` and write to it.
//    // However, using the `log` facade with a backend like `fern` is generally better.
//
//    log::info!("Logger initialized. Logging to: {:?}", log_file_path); // This will use the global logger
//    Ok(())
//}

pub fn setup_logger(run_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let logs_dir = PathBuf::from(LOGS_DIR_NAME);
    fs::create_dir_all(&logs_dir)?;
    let log_file_path = logs_dir.join(TEXT_LOG_FILENAME);

    fern::Dispatch::new()
        .format(move |out, message, record| { // move run_id into the closure
            out.finish(format_args!(
                "{} [{}] RUN_ID:{} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                run_id, // Use the captured run_id
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain( // Log to stdout (optional)
            std::io::stdout()
        )
        .chain(fern::log_file(log_file_path)?) // Log to file (appends by default)
        .apply()?; // Apply the logger globally

    log::info!("Logger initialized."); // This will now use fern
    Ok(())
}
