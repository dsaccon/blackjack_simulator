// src/strategy.rs
use crate::card_deck::Rank;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Hit,
    Stand,
    Double,
    Split,
    // Surrender, // Not implemented in this version
}

// Basic Strategy implementation (Simplified for H17, 6D, DAS)
// p_hand_cards: The cards of the specific hand being evaluated
// dealer_upcard_value: The numerical value of the dealer's upcard (Ace=11 for this)
// num_total_player_hands_for_this_player: How many hands this player currently has (for split limit)
pub fn get_basic_strategy_action(
    p_hand_cards: &[crate::card_deck::Card], // Pass slice of cards
    dealer_upcard_value: u8,
    num_total_player_hands_for_this_player: usize,
    _simulation_active: bool, // Might be used for strategy variations in future
) -> PlayerAction {
    let player_value = calculate_value_for_strategy(p_hand_cards);
    let is_pair = p_hand_cards.len() == 2 &&
                  p_hand_cards[0].rank.blackjack_value() == p_hand_cards[1].rank.blackjack_value();
    let is_soft = is_soft_for_strategy(p_hand_cards);
    let can_double_check = p_hand_cards.len() == 2;
    let can_split_check = is_pair && num_total_player_hands_for_this_player < 4;

    // SPLITTING PAIRS
    if can_split_check {
        let card_rank = p_hand_cards[0].rank; // Both cards have same rank if is_pair
        match card_rank {
            Rank::Ace | Rank::Eight => return PlayerAction::Split,
            Rank::Nine => {
                if ![7, 10, 11].contains(&dealer_upcard_value) { return PlayerAction::Split; }
            }
            Rank::Seven => {
                if dealer_upcard_value <= 7 { return PlayerAction::Split; }
            }
            Rank::Six => {
                if dealer_upcard_value <= 6 { return PlayerAction::Split; } // DAS assumed
            }
            Rank::Four => { // Only split 4s if DAS, and vs 5,6
                if [5, 6].contains(&dealer_upcard_value) { return PlayerAction::Split; } // DAS assumed
            }
            Rank::Three | Rank::Two => {
                if dealer_upcard_value <= 7 { return PlayerAction::Split; } // DAS assumed
            }
            _ => {} // No split for 5s or 10-value cards by default strategy (5,5 is hard 10)
        }
    }

    // SOFT HANDS (Ace counted as 11)
    if is_soft {
        match player_value {
            19..=21 => return PlayerAction::Stand, // Soft 19-21 Stand
            18 => { // Soft 18 (A,7)
                if dealer_upcard_value <= 6 && can_double_check { return PlayerAction::Double; }
                else if [2, 7, 8].contains(&dealer_upcard_value) { return PlayerAction::Stand; }
                else { return PlayerAction::Hit; }
            }
            17 => { // Soft 17 (A,6)
                if (3..=6).contains(&dealer_upcard_value) && can_double_check { return PlayerAction::Double; }
                else { return PlayerAction::Hit; }
            }
            16 => { // Soft 16 (A,5)
                if (4..=6).contains(&dealer_upcard_value) && can_double_check { return PlayerAction::Double; }
                else { return PlayerAction::Hit; }
            }
            15 => { // Soft 15 (A,4)
                if (4..=6).contains(&dealer_upcard_value) && can_double_check { return PlayerAction::Double; }
                else { return PlayerAction::Hit; }
            }
            13 | 14 => { // Soft 13/14 (A,2/A,3)
                if (5..=6).contains(&dealer_upcard_value) && can_double_check { return PlayerAction::Double; }
                else { return PlayerAction::Hit; }
            }
            _ => return PlayerAction::Hit, // Soft 12 or less (should not happen if Ace is 11) or other unhandled
        }
    }

    // HARD HANDS
    match player_value {
        17..=21 => return PlayerAction::Stand,
        13..=16 => {
            if dealer_upcard_value <= 6 { return PlayerAction::Stand; }
            else { return PlayerAction::Hit; }
        }
        12 => {
            if (4..=6).contains(&dealer_upcard_value) { return PlayerAction::Stand; }
            else { return PlayerAction::Hit; }
        }
        11 => {
            if can_double_check { return PlayerAction::Double; } else { return PlayerAction::Hit; }
        }
        10 => {
            if dealer_upcard_value <= 9 && can_double_check { return PlayerAction::Double; }
            else { return PlayerAction::Hit; }
        }
        9 => {
            if (2..=6).contains(&dealer_upcard_value) && can_double_check { return PlayerAction::Double; }
            else { return PlayerAction::Hit; }
        }
        _ => return PlayerAction::Hit, // Hard 8 or less
    }
}

// Helper: Calculate value specifically for strategy (like Hand::value)
fn calculate_value_for_strategy(cards: &[crate::card_deck::Card]) -> u8 {
    let mut total_value = 0u8;
    let mut num_aces = 0u8;
    for card in cards {
        let (val, is_ace) = card.rank.value();
        total_value = total_value.saturating_add(val);
        if is_ace { num_aces += 1; }
    }
    while total_value > 21 && num_aces > 0 {
        total_value -= 10;
        num_aces -= 1;
    }
    total_value
}

// Helper: Check if soft specifically for strategy (like Hand::is_soft)
fn is_soft_for_strategy(cards: &[crate::card_deck::Card]) -> bool {
    let mut non_ace_value = 0u8;
    let mut num_aces = 0u8;
    for card in cards {
        if card.rank == Rank::Ace {
            num_aces += 1;
        } else {
            non_ace_value = non_ace_value.saturating_add(card.rank.value().0);
        }
    }
    if num_aces == 0 { return false; }
    (non_ace_value as u16 + (num_aces as u16 - 1) * 1 + 11) <= 21
}
