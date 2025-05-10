// src/player.rs
use crate::hand::Hand; // Assuming hand.rs is in the same crate (src/)

#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize, // 0 for "You", 1+ for AI
    pub name: String,
    pub is_user: bool,
    pub hands: Vec<Hand>,
    // Flags for "Your" player to track if a split/double happened in the *current round's initial hand*
    // This helps avoid over-counting in stats['hands_involved_in_split/double']
    pub hand_involved_in_split_this_round: bool,
    pub hand_involved_in_double_this_round: bool,
}

impl Player {
    pub fn new_user(id: usize, name: String, initial_bet: f64) -> Self {
        Player {
            id,
            name,
            is_user: true,
            hands: vec![Hand::new(initial_bet)],
            hand_involved_in_split_this_round: false,
            hand_involved_in_double_this_round: false,
        }
    }

    pub fn new_ai(id: usize, name: String) -> Self {
        Player {
            id,
            name,
            is_user: false,
            hands: vec![Hand::new(0.0)], // AI bet is not financially tracked
            hand_involved_in_split_this_round: false, // Not relevant for AI stat tracking
            hand_involved_in_double_this_round: false, // Not relevant for AI stat tracking
        }
    }

    pub fn reset_round_flags(&mut self) {
        if self.is_user {
            self.hand_involved_in_split_this_round = false;
            self.hand_involved_in_double_this_round = false;
        }
    }
}

// Dealer can be a simplified Player-like struct or its own distinct struct
#[derive(Debug, Clone)]
pub struct Dealer {
    pub hand: Hand, // Dealer only has one hand
}

impl Dealer {
    pub fn new() -> Self {
        Dealer {
            hand: Hand::new(0.0), // Bet is irrelevant
        }
    }
}
