// src/main.rs

// Module declarations
mod config;
mod card_deck;
mod hand;
mod player;
mod strategy;
mod game_logic;
mod stats;
mod graph;
mod utils;

// USE STATEMENTS to bring items into the main.rs scope
use crate::card_deck::Deck; // Use `crate::` prefix for clarity, assumes modules are direct children of src
use crate::player::{Player, Dealer};
use crate::stats::{SessionStats, setup_logger};
use crate::graph::generate_balance_graph; // Specific function for graph
use crate::game_logic::play_blackjack_round; // Specific function for playing a round

use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_start_time = Instant::now();
    let run_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    if let Err(e) = setup_logger(run_timestamp) {
        eprintln!("Failed to initialize logger: {}. Game will continue without text logging.", e);
    }

    // Use log facade after logger is set up
    log::info!("--- New Game Session Started (Payout: {}/{}, Total Players: {}) ---",
        config::BLACKJACK_PAYOUT_NUMERATOR, config::BLACKJACK_PAYOUT_DENOMINATOR, config::NUM_PLAYERS);

    println!("--- Welcome to {}-Deck Blackjack! (RUN ID: {}) ---", config::NUM_DECKS, run_timestamp);
    println!("Blackjack Payout: {}/{}", config::BLACKJACK_PAYOUT_NUMERATOR, config::BLACKJACK_PAYOUT_DENOMINATOR);
    println!("Total Players at Table (incl. You): {}", config::NUM_PLAYERS);

    // Check if plotting is available (assuming graph.rs might expose such a const or function)
    // For simplicity, we'll directly reference the const as if it were in graph module.
    // In graph.rs, you'd have: pub const MATPLOTLIB_AVAILABLE: bool = true; // or based on try_import
    if !graph::MATPLOTLIB_AVAILABLE { // Assuming MATPLOTLIB_AVAILABLE is a public const in graph module
        println!("Note: Plotting library backend issue. Balance graph will not be generated for simulations.");
    }

    let game_mode_input = utils::get_user_input(
        "Choose mode: (i)nteractive or (s)imulation (for 'Your' play)? ",
    );
    let is_simulation_for_user_player = game_mode_input == "s";

    let mut your_player_balance = config::STARTING_BALANCE;
    let mut session_stats = SessionStats::new(
        run_timestamp,
        if is_simulation_for_user_player { "Simulation (You: Book, AI: Book)".to_string() }
        else { "Interactive (You: Manual/Book, AI: Book)".to_string() },
        config::STARTING_BALANCE,
    );

    log::info!("Mode Selected (for 'Your' play): {}", session_stats.mode);
    log::info!("Starting Balance (You): ${:.2}", your_player_balance);
    log::info!("Configured Default Bet (You): ${:.2}", config::DEFAULT_BET);

    let mut deck = Deck::new(config::NUM_DECKS);
    println!("--- Initializing a new {}-deck shoe with {} cards. ---", config::NUM_DECKS, deck.initial_size);
    utils::sleep_ms(utils::get_delay_multiplied(500, false)); // Delay not dependent on sim active yet


    if is_simulation_for_user_player {
        let num_iterations = utils::get_num_iterations(config::DEFAULT_SIM_ITERATIONS);
        session_stats.target_iterations = Some(num_iterations);
        println!("\nStarting simulation for {} hands. 'You' will play by Book strategy.", num_iterations);
        log::info!("Simulation Target Iterations: {}", num_iterations);

        let mut balance_history_sim: Vec<f64> = vec![your_player_balance];

        for i in 0..num_iterations {
            println!("\n--- Sim Hand #{} / {} | Your Bal: ${:.2} ---", i + 1, num_iterations, your_player_balance);
            log::info!("Starting Sim Hand #{}", i + 1);

            if deck.needs_reshuffle(config::RESHUFFLE_THRESHOLD_RATIO) {
                println!("--- Shoe penetration low ({} cards left). Reshuffling... ---", deck.len());
                log::info!("Reshuffling shoe. Cards left: {}", deck.len());
                deck = Deck::new(config::NUM_DECKS);
                println!("--- New shoe shuffled with {} cards. ---", deck.initial_size);
                utils::sleep_ms(utils::get_delay_multiplied(500, true));
            }

            let mut all_players_at_table: Vec<Player> = Vec::new();
            all_players_at_table.push(Player::new_user(0, "Your".to_string(), 0.0));
            for p_id in 1..config::NUM_PLAYERS {
                all_players_at_table.push(Player::new_ai(p_id, format!("Player {}", p_id + 1)));
            }
            let mut dealer = Dealer::new();

            if your_player_balance < config::DEFAULT_BET {
                let msg = format!("Your Balance (${:.2}) < Default Bet (${:.2}). Sim ends.", your_player_balance, config::DEFAULT_BET);
                println!("{}", msg); log::warn!("Sim ended early at hand {}: {}", i + 1, msg);
                break;
            }

            if !play_blackjack_round( // Use directly after `use` statement
                &mut deck,
                &mut all_players_at_table,
                &mut dealer,
                &mut your_player_balance,
                &mut session_stats,
                true, // is_simulation_round = true
                run_timestamp,
                0.02, // delay_multiplier
            ) {
                let msg = "Could not place Your bet (sim). Sim ends.";
                println!("{}", msg); log::warn!("Sim ended early at hand {}: {}", i + 1, msg);
                break;
            }

            balance_history_sim.push(your_player_balance);
            session_stats.update_balance_extremes(your_player_balance);

            if your_player_balance < config::MIN_BET_ALLOWED {
                let msg = format!("Your Balance (${:.2}) < Min Bet (${:.2}). Sim ends.", your_player_balance, config::MIN_BET_ALLOWED);
                println!("{}", msg); log::warn!("Sim ended early at hand {}: {}", i + 1, msg);
                break;
            }
            log::info!("Finished Sim Hand #{}. Your Balance: ${:.2}", i + 1, your_player_balance);
        }

        if let Err(e) = generate_balance_graph(&balance_history_sim, run_timestamp, session_stats.initial_balance) {
            log::error!("Failed to generate balance graph: {}", e);
            eprintln!("Error generating balance graph: {}", e);
        }

        session_stats.final_balance = your_player_balance;
        session_stats.calculate_final_metrics();

        println!("\n\n--- Simulation Finished (Your Play: Book) ---");
        log::info!("--- Simulation Results (Your Play: Book) ---");
        for line in session_stats.to_log_lines() {
            println!("{}", line);
            log::info!("{}", line);
        }

    } else { // Interactive Mode for "Your" play
        println!("\nStarting interactive game for 'You'. Other {} player(s) will play by Book.",
            if config::NUM_PLAYERS > 0 { config::NUM_PLAYERS.saturating_sub(1) } else { 0 });

        let mut balance_history_interactive: Vec<f64> = vec![your_player_balance];

        loop {
            println!("\n--- New Interactive Hand | Your Bal: ${:.2} ---", your_player_balance);
            log::info!("Starting New Interactive Hand. Your Balance: ${:.2}", your_player_balance);

            if your_player_balance < config::MIN_BET_ALLOWED {
                let msg = format!("\nYour balance (${:.2}) is too low. Game over!", your_player_balance);
                println!("{}", msg); log::info!("{}", msg);
                break;
            }
            if your_player_balance < config::DEFAULT_BET && your_player_balance >= config::MIN_BET_ALLOWED {
                println!("\nNotice: Your balance (${:.2}) is less than the default bet (${:.2}).", your_player_balance, config::DEFAULT_BET);
            }

            if deck.needs_reshuffle(config::RESHUFFLE_THRESHOLD_RATIO) {
                println!("--- Shoe penetration low ({} cards left). Reshuffling... ---", deck.len());
                log::info!("Reshuffling shoe. Cards left: {}", deck.len());
                deck = Deck::new(config::NUM_DECKS);
                println!("--- New shoe shuffled with {} cards. ---", deck.initial_size);
                utils::sleep_ms(utils::get_delay_multiplied(500, false));
            }

            let mut all_players_at_table: Vec<Player> = Vec::new();
            all_players_at_table.push(Player::new_user(0, "Your".to_string(), 0.0));
            for p_id in 1..config::NUM_PLAYERS {
                all_players_at_table.push(Player::new_ai(p_id, format!("Player {}", p_id + 1)));
            }
            let mut dealer = Dealer::new();

            if !play_blackjack_round( // Use directly
                &mut deck,
                &mut all_players_at_table,
                &mut dealer,
                &mut your_player_balance,
                &mut session_stats,
                false, // is_simulation_round = false
                run_timestamp,
                1.0, // delay_multiplier
            ) {
                let msg = "Could not play Your hand (likely insufficient funds). Game over.";
                println!("{}", msg); log::info!("{}", msg);
                break;
            }

            balance_history_interactive.push(your_player_balance);
            session_stats.update_balance_extremes(your_player_balance);
            log::info!("Finished Interactive Hand. Your Balance: ${:.2}", your_player_balance);

            if utils::get_user_input("\nPlay another hand? (y/n): ") != "y" {
                break;
            }
        }

        // Optionally generate graph for interactive mode too
        // if let Err(e) = generate_balance_graph(&balance_history_interactive, run_timestamp, session_stats.initial_balance) {
        //     log::error!("Failed to generate balance graph for interactive mode: {}", e);
        // }

        session_stats.final_balance = your_player_balance;
        session_stats.calculate_final_metrics();

        println!("\n--- Interactive Session Ended ---");
        log::info!("--- Interactive Session Results (Your Play: Interactive) ---");
        let final_msg = "Thanks for playing!";
        println!("{}", final_msg); log::info!("{}", final_msg);

        for line in session_stats.to_log_lines() {
            // Selective printing for interactive console
            if line.starts_with("Run ID:") || line.starts_with("Mode:") ||
               line.starts_with("Starting Balance") || line.starts_with("Final Balance") ||
               line.starts_with("Highest Balance") || line.starts_with("Lowest Balance") ||
               line.starts_with("Hands Played") || line.starts_with("Your Blackjacks") ||
               line.starts_with("Your Times Split") || line.starts_with("Configured Default Bet")
            {
                 if line.contains("Final balance") && !is_simulation_for_user_player { /* Already shown */ }
                 else { println!("{}", line); }
            }
            log::info!("{}", line);
        }
    }

    let script_end_time = Instant::now(); // <<<<---- ADD: Mark script end time
    let total_elapsed_time = script_end_time.duration_since(script_start_time);

    // Calculate average time per hand (for "Your" main hands)
    let average_time_per_hand = if session_stats.hands_played_session > 0 {
        total_elapsed_time.as_secs_f64() / session_stats.hands_played_session as f64
    } else {
        0.0 // Avoid division by zero if no hands were played
    };

    // --- Update SessionStats with timing if you add fields for it, or just print/log ---
    // For simplicity, we'll just print and log here.
    // If you add fields to SessionStats, update them before calling to_log_lines().
    // Example:
    session_stats.total_script_runtime_seconds = total_elapsed_time.as_secs_f64();
    session_stats.avg_time_per_hand_seconds = average_time_per_hand;
    session_stats.calculate_final_metrics(); // Then call this if it uses the new fields

    // --- Log and Print Timing Information ---
    let timing_info_total = format!("Total script execution time: {:.3} seconds.", total_elapsed_time.as_secs_f64());
    let timing_info_avg = format!("Average time per main hand played by You: {:.4} seconds.", average_time_per_hand);

    println!("\n--- Timing Statistics ---");
    println!("{}", timing_info_total);
    println!("{}", timing_info_avg);

    log::info!("--- Timing Statistics ---");
    log::info!("{}", timing_info_total);
    log::info!("{}", timing_info_avg);


    // --- Existing Final Logging and Printing ---
    if is_simulation_for_user_player {
        // ... (existing simulation results print/log) ...
        // The results_lines from session_stats.to_log_lines() will be printed/logged
    } else { // Interactive Mode
        // ... (existing interactive results print/log) ...
    }

    log::info!("--- Session Ended (RUN ID: {}) ---", run_timestamp);
    println!("\nFull session results logged to: {}/{}", config::LOGS_DIR_NAME, config::TEXT_LOG_FILENAME);
    Ok(())
}
