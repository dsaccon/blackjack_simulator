// src/card_deck.rs
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Heart, Diamond, Club, Spade,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Suit::Heart   => write!(f, "♥"),
            Suit::Diamond => write!(f, "♦"),
            Suit::Club    => write!(f, "♣"),
            Suit::Spade   => write!(f, "♠"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_str = match self {
            Rank::Two   => "2", Rank::Three => "3", Rank::Four  => "4",
            Rank::Five  => "5", Rank::Six   => "6", Rank::Seven => "7",
            Rank::Eight => "8", Rank::Nine  => "9", Rank::Ten   => "10",
            Rank::Jack  => "J", Rank::Queen => "Q", Rank::King  => "K",
            Rank::Ace   => "A",
        };
        write!(f, "{}", display_str)
    }
}

impl Rank {
    // Returns (primary_value, is_ace_initially_11)
    pub fn value(&self) -> (u8, bool) {
        match self {
            Rank::Two   => (2, false), Rank::Three => (3, false), Rank::Four  => (4, false),
            Rank::Five  => (5, false), Rank::Six   => (6, false), Rank::Seven => (7, false),
            Rank::Eight => (8, false), Rank::Nine  => (9, false),
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => (10, false),
            Rank::Ace   => (11, true),
        }
    }
    // Value for pair checking (10 for face cards)
    pub fn blackjack_value(&self) -> u8 {
         match self {
            Rank::Ace => 11, // For pair checking, an Ace has a distinct value conceptually
            _ => self.value().0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
    pub initial_size: usize,
}

impl Deck {
    pub fn new(num_decks: usize) -> Self {
        let mut cards = Vec::new();
        let suits = [Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
            Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];

        for _ in 0..num_decks {
            for &suit_val in suits.iter() {
                for &rank_val in ranks.iter() {
                    cards.push(Card { rank: rank_val, suit: suit_val });
                }
            }
        }
        let initial_size = cards.len();
        let mut deck = Deck { cards, initial_size };
        deck.shuffle(); // Shuffle on creation
        deck
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn needs_reshuffle(&self, threshold_ratio: f64) -> bool {
        (self.cards.len() as f64) < (self.initial_size as f64 * threshold_ratio)
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    #[allow(dead_code)] // For potential debugging
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}
