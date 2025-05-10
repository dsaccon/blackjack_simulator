// src/utils.rs
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

// Add these use statements to bring Player and Dealer into scope for this module
use crate::player::{Player, Dealer}; // Assuming player.rs defines Player and Dealer
use crate::hand::HandStatus;         // Assuming hand.rs defines HandStatus
use crate::card_deck::Rank;          // Assuming card_deck.rs defines Rank

pub fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_lowercase()
}

pub fn get_delay_multiplied(base_millis: u64, simulation_active: bool) -> u64 {
    if simulation_active {
        (base_millis as f64 * 0.02) as u64
    } else {
        base_millis
    }
}

pub fn sleep_ms(millis: u64) {
    if millis > 0 {
        thread::sleep(Duration::from_millis(millis));
    }
}

pub fn get_num_iterations(default_iterations: u32) -> u32 {
    loop {
        let input_str = get_user_input(&format!(
            "Enter number of simulation iterations (or press Enter for default {}): ",
            default_iterations
        ));
        if input_str.is_empty() {
            return default_iterations;
        }
        match input_str.parse::<u32>() {
            Ok(num) if num > 0 => return num,
            Ok(_) => println!("Number of iterations must be positive."),
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    }
}

pub fn get_your_bet_from_input(current_balance: f64, default_bet: f64, min_bet: f64) -> Option<f64> {
    if current_balance < min_bet {
        println!(
            "Your balance (${:.2}) is too low to place any bet (min: ${:.2}).",
            current_balance, min_bet
        );
        return None;
    }
    loop {
        let input_str = get_user_input(&format!(
            "Your balance: ${:.2}. Enter bet (or press Enter for default ${:.2}): ",
            current_balance, default_bet
        ));
        let bet_amount = if input_str.is_empty() {
            default_bet
        } else {
            match input_str.parse::<f64>() {
                Ok(b) => b,
                Err(_) => {
                    println!("Invalid input. Please enter a number or press Enter.");
                    continue;
                }
            }
        };

        if bet_amount < min_bet {
            println!("Bet must be at least ${:.2}.", min_bet);
        } else if bet_amount > current_balance {
            println!(
                "You cannot bet more than your current balance (${:.2}).",
                current_balance
            );
        } else {
            return Some(bet_amount);
        }
        if input_str.is_empty() && default_bet > current_balance {
             println!(
                "Default bet (${:.2}) is higher than your current balance (${:.2}). Please enter a valid amount.",
                default_bet, current_balance
            );
        }
    }
}

// Display helper functions
pub fn display_your_hands_and_dealer(
    your_player_obj: &Player, // Now Player is in scope
    dealer_obj: &Dealer,       // Now Dealer is in scope
    hide_dealer_hole_card: bool,
) {
    println!("--------------------------------------------------");
    print!("Dealer's hand: ");
    if dealer_obj.hand.cards.is_empty() {
        println!("[No cards]");
    } else {
        for (i, card) in dealer_obj.hand.cards.iter().enumerate() {
            if i == 1 && hide_dealer_hole_card { // Assuming hole card is at index 1
                print!("[Hidden Card] ");
            } else {
                print!("{} ", card);
            }
        }
        if !hide_dealer_hole_card || dealer_obj.hand.cards.len() == 1 {
            println!("(Value: {})", dealer_obj.hand.value());
        } else if !dealer_obj.hand.cards.is_empty() {
            // Correctly access rank from card_deck module
             let upcard_val_str = if dealer_obj.hand.cards[0].rank == Rank::Ace {"1/11".to_string()} else {dealer_obj.hand.cards[0].rank.value().0.to_string()};
             println!("(Showing: {})", upcard_val_str);
        } else {
             println!();
        }
    }

    for (i, p_hand) in your_player_obj.hands.iter().enumerate() {
        let cards_str: String = p_hand.cards.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(", ");
        let active_marker = if p_hand.status == HandStatus::Active && your_player_obj.is_user { "*" } else { " " }; // HandStatus in scope
        println!(
            "{}{} Hand {}: {} (Value: {}) Bet: ${:.2} [{}]",
            active_marker,
            your_player_obj.name,
            i + 1,
            cards_str,
            p_hand.value(),
            p_hand.bet,
            p_hand.status // HandStatus used for Display trait
        );
    }
    println!("--------------------------------------------------");
}

pub fn display_dealer_final_hand(dealer_obj: &Dealer) { // Dealer in scope
    print!("Dealer's final hand: ");
    if dealer_obj.hand.cards.is_empty() {
        println!("[No cards]");
    } else {
        for card in &dealer_obj.hand.cards {
            print!("{} ", card);
        }
        println!("(Value: {})", dealer_obj.hand.value());
    }
}
