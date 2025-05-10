// src/hand.rs
//use crate::card_deck::{Card, Rank};
use crate::card_deck::{Card};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)] // Added Eq for easier comparison in some cases
pub enum HandStatus {
    Active,    // Still playing
    Stood,     // Player chose to stand
    Busted,    // Hand value > 21
    Doubled,   // Player doubled down, turn ends
    Blackjack, // Natural 21 on first two cards
}

impl fmt::Display for HandStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandStatus::Active => write!(f, "Active"),
            HandStatus::Stood => write!(f, "Stood"),
            HandStatus::Busted => write!(f, "Busted"),
            HandStatus::Doubled => write!(f, "Doubled"),
            HandStatus::Blackjack => write!(f, "Blackjack"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hand {
    pub cards: Vec<Card>,
    pub bet: f64, // Only "Your" player's bet is financially tracked
    pub status: HandStatus,
    pub is_split_ace: bool,
    pub actions_taken: Vec<String>, // For potential detailed logging or complex strategy
}

impl Hand {
    pub fn new(initial_bet: f64) -> Self {
        Hand {
            cards: Vec::new(),
            bet: initial_bet,
            status: HandStatus::Active,
            is_split_ace: false,
            actions_taken: Vec::new(),
        }
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn value(&self) -> u8 {
        let mut total_value = 0u8;
        let mut num_aces = 0u8;
        for card in &self.cards {
            let (val, is_ace) = card.rank.value();
            total_value = total_value.saturating_add(val); // Prevent overflow, though unlikely with u8
            if is_ace {
                num_aces += 1;
            }
        }

        while total_value > 21 && num_aces > 0 {
            total_value -= 10;
            num_aces -= 1;
        }
        total_value
    }

//    pub fn is_soft(&self) -> bool {
//        let mut non_ace_value = 0;
//        let mut num_aces = 0;
//        for card in &self.cards {
//            if card.rank == Rank::Ace {
//                num_aces += 1;
//            } else {
//                non_ace_value += card.rank.value().0;
//            }
//        }
//        if num_aces == 0 {
//            return false;
//        }
//        // An ace can be 11 if (value_of_other_cards + (num_aces - 1)*1 + 11) <= 21
//        non_ace_value as u16 + (num_aces as u16 - 1) * 1 + 11 <= 21
//    }

    pub fn is_pair(&self) -> bool {
        if self.cards.len() == 2 {
            // Compare blackjack_value which treats J,Q,K as 10 for pairing
            self.cards[0].rank.blackjack_value() == self.cards[1].rank.blackjack_value()
        } else {
            false
        }
    }
    
    // Simplified can_split for general logic; financial checks are done by caller
    pub fn is_splittable_pair(&self, num_current_hands_for_player: usize) -> bool {
         self.is_pair() && num_current_hands_for_player < 4 && !self.is_split_ace
    }
    
    // Simplified can_double for general logic; financial checks by caller
    pub fn is_doublable(&self) -> bool {
        self.cards.len() == 2
    }


    pub fn is_natural_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value() == 21
    }
}
