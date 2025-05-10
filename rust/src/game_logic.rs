// src/game_logic.rs
use crate::card_deck::{Deck, Card, Rank};
use crate::hand::{Hand, HandStatus};
use crate::player::{Player, Dealer};
use crate::strategy::{get_basic_strategy_action, PlayerAction};
use crate::config;
use crate::utils;
use crate::stats::SessionStats;

pub fn play_blackjack_round(
    deck: &mut Deck,
    all_players_at_table: &mut Vec<Player>,
    dealer: &mut Dealer,
    your_player_balance: &mut f64,
    session_stats: &mut SessionStats,
    is_simulation_round: bool,
    run_timestamp: u64,
    _delay_multiplier: f64,
) -> bool {

    if let Some(user_player) = all_players_at_table.get_mut(0) {
        if user_player.is_user {
            user_player.reset_round_flags();
        }
    } else {
        log::error!("RUN_ID:{} - User player (index 0) not found.", run_timestamp);
        return false;
    }

    let your_initial_bet: f64;
    if let Some(user_player) = all_players_at_table.get_mut(0) {
        if user_player.is_user {
            if is_simulation_round {
                if *your_player_balance >= config::DEFAULT_BET {
                    your_initial_bet = config::DEFAULT_BET;
                    user_player.hands[0].bet = your_initial_bet;
                    println!("Simulation ('You'): Auto-betting default ${:.2}", your_initial_bet);
                } else {
                    println!("Simulation ('You'): Balance (${:.2}) too low for default bet (${:.2}).", *your_player_balance, config::DEFAULT_BET);
                    return false;
                }
            } else {
                match utils::get_your_bet_from_input(*your_player_balance, config::DEFAULT_BET, config::MIN_BET_ALLOWED) {
                    Some(bet) => {
                        your_initial_bet = bet;
                        user_player.hands[0].bet = your_initial_bet;
                    }
                    None => return false,
                }
            }
            session_stats.hands_played_session += 1;
        } else { your_initial_bet = 0.0; }
    } else { return false; }

    dealer.hand.cards.clear();
    dealer.hand.status = HandStatus::Active;
    for player in all_players_at_table.iter_mut() {
        player.hands.truncate(1);
        if let Some(hand) = player.hands.get_mut(0) {
            hand.cards.clear();
            hand.status = HandStatus::Active;
            hand.is_split_ace = false;
            hand.actions_taken.clear();
            if player.is_user {
                hand.bet = your_initial_bet;
            } else {
                hand.bet = 0.0;
            }
        } else {
            log::error!("RUN_ID:{} - Player {} has no hands to reset.", run_timestamp, player.name);
        }
    }

    println!("\n--- Dealing Cards ---");
    utils::sleep_ms(utils::get_delay_multiplied(200, is_simulation_round));
    for p_obj in all_players_at_table.iter_mut() {
        if let Some(card) = deck.deal() { p_obj.hands[0].add_card(card); }
        else { log::error!("RUN_ID:{} - Deck empty during initial deal (player 1st card)!", run_timestamp); return false; }
    }
    if let Some(card) = deck.deal() { dealer.hand.add_card(card); }
    else { log::error!("RUN_ID:{} - Deck empty during initial deal (dealer upcard)!", run_timestamp); return false; }
    for p_obj in all_players_at_table.iter_mut() {
        if let Some(card) = deck.deal() { p_obj.hands[0].add_card(card); }
        else { log::error!("RUN_ID:{} - Deck empty during initial deal (player 2nd card)!", run_timestamp); return false; }
    }
    if let Some(card) = deck.deal() { dealer.hand.add_card(card); }
    else { log::error!("RUN_ID:{} - Deck empty during initial deal (dealer hole card)!", run_timestamp); return false; }

    if let Some(user_player) = all_players_at_table.get(0) {
        if user_player.is_user {
            utils::display_your_hands_and_dealer(user_player, dealer, true);
        }
    }
    for i in 1..config::NUM_PLAYERS {
        if let Some(ai_p) = all_players_at_table.get(i) {
            let hand_val = ai_p.hands[0].value();
            let cards_str: String = ai_p.hands[0].cards.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(", ");
            println!("{} (AI): {} (Value: {})", ai_p.name, cards_str, hand_val);
        }
    }
    utils::sleep_ms(utils::get_delay_multiplied(500, is_simulation_round));

    let mut your_bj_resolved_this_round = false;
    if let Some(user_player) = all_players_at_table.get_mut(0) {
        if user_player.is_user && user_player.hands[0].is_natural_blackjack() {
            user_player.hands[0].status = HandStatus::Blackjack;
            session_stats.blackjacks_dealt_player += 1;
            println!("\n{} Blackjack!", user_player.name);
            utils::sleep_ms(utils::get_delay_multiplied(200, is_simulation_round));
            println!("Dealer checks for Blackjack...");
            utils::display_dealer_final_hand(dealer);
            if dealer.hand.is_natural_blackjack() {
                println!("Dealer also has Blackjack! It's a Push for Your hand.");
                session_stats.total_pushes += 1;
            } else {
                let winnings = user_player.hands[0].bet * config::BLACKJACK_PAYOUT_MULTIPLIER;
                *your_player_balance += winnings;
                session_stats.total_wins +=1;
                println!("You win ${:.2}! (Payout: {}/{})", winnings, config::BLACKJACK_PAYOUT_NUMERATOR, config::BLACKJACK_PAYOUT_DENOMINATOR);
            }
            println!("Your balance: ${:.2}", *your_player_balance);
            your_bj_resolved_this_round = true;
        }
    }

    for p_idx in 0..all_players_at_table.len() {
        if all_players_at_table[p_idx].is_user && your_bj_resolved_this_round {
            continue;
        }
        if !all_players_at_table[p_idx].is_user && all_players_at_table[p_idx].hands[0].is_natural_blackjack() {
            all_players_at_table[p_idx].hands[0].status = HandStatus::Blackjack;
            let cards_str: String = all_players_at_table[p_idx].hands[0].cards.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(", ");
            println!("\n{} (AI) has Blackjack: {}", all_players_at_table[p_idx].name, cards_str);
            utils::sleep_ms(utils::get_delay_multiplied(300, is_simulation_round));
            continue;
        }

        println!("\n--- {}'s Turn ---", all_players_at_table[p_idx].name);
        if all_players_at_table[p_idx].is_user {
             utils::sleep_ms(utils::get_delay_multiplied(500, is_simulation_round));
        }

        let mut current_hand_idx_for_player = 0;
        let mut auto_play_book_all_your_hands_this_round = if all_players_at_table[p_idx].is_user { is_simulation_round } else { false };

        while current_hand_idx_for_player < all_players_at_table[p_idx].hands.len() {
            let hand_status_check = all_players_at_table[p_idx].hands[current_hand_idx_for_player].status.clone();
            if hand_status_check != HandStatus::Active {
                current_hand_idx_for_player += 1;
                continue;
            }

            'action_loop: loop {
                let player_for_display = &all_players_at_table[p_idx];
                let hand_for_display = &player_for_display.hands[current_hand_idx_for_player];

                if hand_for_display.status != HandStatus::Active {
                    break 'action_loop;
                }

                if player_for_display.is_user {
                    utils::display_your_hands_and_dealer(player_for_display, dealer, true);
                } else {
                    let cards_str: String = hand_for_display.cards.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(", ");
                    let hand_val = hand_for_display.value();
                    let dealer_up_str = dealer.hand.cards[0].to_string();
                    println!("{} (AI) Hand {}: {} (Value: {}) vs Dealer Up: {}",
                        player_for_display.name, current_hand_idx_for_player + 1,
                        cards_str, hand_val, dealer_up_str);
                }

                let player_value_of_current_hand_display = hand_for_display.value();
                if player_value_of_current_hand_display > 21 {
                    all_players_at_table[p_idx].hands[current_hand_idx_for_player].status = HandStatus::Busted;
                    println!("{} Hand {} Busts!", all_players_at_table[p_idx].name, current_hand_idx_for_player + 1);
                    break 'action_loop;
                }

                let dealer_upcard_val = dealer.hand.cards[0].rank.value().0;
                let can_double = hand_for_display.is_doublable() &&
                                 (!player_for_display.is_user || *your_player_balance >= hand_for_display.bet);
                let can_split = hand_for_display.is_splittable_pair(player_for_display.hands.len()) &&
                                (!player_for_display.is_user || *your_player_balance >= hand_for_display.bet);

                let chosen_action: PlayerAction;
                let is_book_play_for_this_action = (!player_for_display.is_user) ||
                                                   (player_for_display.is_user && auto_play_book_all_your_hands_this_round) ||
                                                   (player_for_display.is_user && hand_for_display.actions_taken.contains(&"UserChoseBookMode".to_string()));

                if is_book_play_for_this_action {
                    let mut book_action = get_basic_strategy_action(
                        &hand_for_display.cards, dealer_upcard_val, player_for_display.hands.len(), is_simulation_round,
                    );
                    if book_action == PlayerAction::Double && !can_double { book_action = PlayerAction::Hit; }
                    if book_action == PlayerAction::Split && !can_split {
                        book_action = get_basic_strategy_action(&hand_for_display.cards, dealer_upcard_val, 10, is_simulation_round);
                        if book_action == PlayerAction::Split { book_action = PlayerAction::Hit; }
                    }
                    chosen_action = book_action;
                    let action_desc = match chosen_action {
                        PlayerAction::Hit => "Hit", PlayerAction::Stand => "Stand",
                        PlayerAction::Double => "Double Down", PlayerAction::Split => "Split",
                    };
                    println!("{} Hand {} (Book): {}", player_for_display.name, current_hand_idx_for_player + 1, action_desc);
                    utils::sleep_ms(utils::get_delay_multiplied(400, is_simulation_round));
                } else { // Interactive choice for "You"
                    let mut prompt = format!("Your Hand {}: (H)it, (S)tand", current_hand_idx_for_player + 1);
                    let mut valid_choices = vec!["h", "s"];
                    if can_double { prompt.push_str(", (D)ouble"); valid_choices.push("d"); }
                    if can_split { prompt.push_str(", (P)Split"); valid_choices.push("p"); }
                    if !auto_play_book_all_your_hands_this_round {
                         prompt.push_str(", (B)ook plays all your hands"); valid_choices.push("b");
                    }
                    prompt.push_str(": ");
                    let user_input = utils::get_user_input(&prompt);

                    match user_input.as_str() {
                        "h" => chosen_action = PlayerAction::Hit,
                        "s" => chosen_action = PlayerAction::Stand,
                        "d" if can_double => chosen_action = PlayerAction::Double,
                        "p" if can_split => chosen_action = PlayerAction::Split,
                        "b" if !auto_play_book_all_your_hands_this_round && player_for_display.is_user => {
                            auto_play_book_all_your_hands_this_round = true;
                            all_players_at_table[p_idx].hands[current_hand_idx_for_player]
                                .actions_taken.push("UserChoseBookMode".to_string());
                            println!("Book will play out all your remaining hands for this round.");
                            continue 'action_loop;
                        }
                        _ => { println!("Invalid action or action not allowed."); continue 'action_loop; }
                    }
                }

                let current_player_obj_mut_for_action = &mut all_players_at_table[p_idx];

                match chosen_action {
                    PlayerAction::Hit => {
                        let p_hand_mut_for_action = &mut current_player_obj_mut_for_action.hands[current_hand_idx_for_player];
                        if let Some(new_card) = deck.deal() {
                            p_hand_mut_for_action.add_card(new_card);
                            println!("{} Hand {} Hits, draws {}", current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1, new_card);
                            if p_hand_mut_for_action.value() > 21 {
                                p_hand_mut_for_action.status = HandStatus::Busted;
                                println!("{} Hand {} Busts!", current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1);
                                break 'action_loop;
                            }
                            if p_hand_mut_for_action.value() == 21 && p_hand_mut_for_action.is_split_ace {
                                p_hand_mut_for_action.status = HandStatus::Stood;
                                println!("{} Hand {} (Split Ace) has 21, stands.", current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1);
                                break 'action_loop;
                            }
                        } else { log::error!("RUN_ID:{} - Deck empty during hit!", run_timestamp); break 'action_loop; }
                    }
                    PlayerAction::Stand => {
                        let p_hand_mut_for_action = &mut current_player_obj_mut_for_action.hands[current_hand_idx_for_player];
                        p_hand_mut_for_action.status = HandStatus::Stood;
                        println!("{} Hand {} Stands.", current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1);
                        break 'action_loop;
                    }
                    PlayerAction::Double => {
                        let p_hand_mut_for_action = &mut current_player_obj_mut_for_action.hands[current_hand_idx_for_player];
                        if current_player_obj_mut_for_action.is_user {
                            session_stats.times_doubled_chosen += 1;
                            if !current_player_obj_mut_for_action.hand_involved_in_double_this_round {
                                session_stats.hands_involved_in_double += 1;
                                current_player_obj_mut_for_action.hand_involved_in_double_this_round = true;
                            }
                            p_hand_mut_for_action.bet *= 2.0;
                        }
                        if let Some(new_card) = deck.deal() {
                            p_hand_mut_for_action.add_card(new_card);
                            p_hand_mut_for_action.status = HandStatus::Doubled;
                            let bet_display = if current_player_obj_mut_for_action.is_user {
                                format!("Bet is now ${:.2}. ", p_hand_mut_for_action.bet)
                            } else { "".to_string() };
                            println!("{} Hand {} Doubles Down. {}Draws {}",
                                current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1, bet_display, new_card);
                            if p_hand_mut_for_action.value() > 21 {
                                p_hand_mut_for_action.status = HandStatus::Busted;
                                println!("{} Hand {} Busts after doubling!", current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1);
                            }
                        } else { log::error!("RUN_ID:{} - Deck empty during double!", run_timestamp); }
                        break 'action_loop;
                    }
                    PlayerAction::Split => {
                        let original_bet_for_split: f64;
                        let is_ace_split_check: bool;
                        let card_to_move_to_new_hand: Card;
                        let mut original_hand_auto_stood = false;

                        {
                            let hand_being_split = &mut current_player_obj_mut_for_action.hands[current_hand_idx_for_player];
                            original_bet_for_split = if current_player_obj_mut_for_action.is_user { hand_being_split.bet } else { 0.0 };
                            is_ace_split_check = hand_being_split.cards[0].rank == Rank::Ace;
                            card_to_move_to_new_hand = hand_being_split.cards.pop().expect("Hand should have card for split");
                            hand_being_split.is_split_ace = is_ace_split_check;
                            hand_being_split.actions_taken.push("split_created_new_hand".to_string());
                            if let Some(c1) = deck.deal() { hand_being_split.add_card(c1); }
                            else { log::error!("RUN_ID:{} - Deck empty for 1st split card!", run_timestamp); break 'action_loop; }
                            if hand_being_split.is_split_ace {
                                hand_being_split.status = HandStatus::Stood;
                                original_hand_auto_stood = true;
                            }
                        }

                        let mut new_player_hand_obj = Hand::new(original_bet_for_split);
                        new_player_hand_obj.add_card(card_to_move_to_new_hand);
                        new_player_hand_obj.is_split_ace = is_ace_split_check;
                        new_player_hand_obj.actions_taken.push(format!("split_from_hand_{}", current_hand_idx_for_player + 1));
                        if let Some(c2) = deck.deal() { new_player_hand_obj.add_card(c2); }
                        else { log::error!("RUN_ID:{} - Deck empty for 2nd split card!", run_timestamp); break 'action_loop; }
                        if new_player_hand_obj.is_split_ace {
                            new_player_hand_obj.status = HandStatus::Stood;
                        }
                        
                        current_player_obj_mut_for_action.hands.insert(current_hand_idx_for_player + 1, new_player_hand_obj);
                        println!("{} Hand {} splits. New Hand {} created.",
                            current_player_obj_mut_for_action.name, current_hand_idx_for_player + 1, current_hand_idx_for_player + 2);
                        utils::sleep_ms(utils::get_delay_multiplied(300, is_simulation_round));

                        if current_player_obj_mut_for_action.is_user {
                            session_stats.times_split_chosen += 1;
                            if !current_player_obj_mut_for_action.hand_involved_in_split_this_round {
                                session_stats.hands_involved_in_split += 1;
                                current_player_obj_mut_for_action.hand_involved_in_split_this_round = true;
                            }
                        }
                        
                        if original_hand_auto_stood {
                            // Corrected line:
                            let hand_that_stood_value = current_player_obj_mut_for_action.hands[current_hand_idx_for_player].value();
                            if hand_that_stood_value == 21 {
                                println!("Split Ace Hand {} has 21, stands.", current_hand_idx_for_player + 1);
                            } else {
                                println!("Split Ace Hand {} (Value: {}) stands.", current_hand_idx_for_player + 1, hand_that_stood_value);
                            }
                            break 'action_loop;
                        }
                        if current_player_obj_mut_for_action.is_user && current_player_obj_mut_for_action.hands[current_hand_idx_for_player].status == HandStatus::Active {
                           utils::display_your_hands_and_dealer(current_player_obj_mut_for_action, dealer, true);
                        }
                        continue 'action_loop;
                    }
                }
                utils::sleep_ms(utils::get_delay_multiplied(300, is_simulation_round));
            }
            current_hand_idx_for_player += 1;
        }
    }

    let mut any_player_hand_needs_dealer_play = false;
    for p_obj in all_players_at_table.iter() {
        for hand_data in p_obj.hands.iter() {
            if matches!(hand_data.status, HandStatus::Active | HandStatus::Stood | HandStatus::Doubled) {
                any_player_hand_needs_dealer_play = true; break;
            }
        }
        if any_player_hand_needs_dealer_play { break; }
    }

    if any_player_hand_needs_dealer_play {
        println!("\n--- Dealer's Turn ---");
        utils::sleep_ms(utils::get_delay_multiplied(1000, is_simulation_round));
        utils::display_dealer_final_hand(dealer);
        let mut dealer_value = dealer.hand.value();
        while dealer_value < 17 {
            println!("Dealer hits...");
            utils::sleep_ms(utils::get_delay_multiplied(1000, is_simulation_round));
            if let Some(new_card) = deck.deal() {
                dealer.hand.add_card(new_card);
                dealer_value = dealer.hand.value();
                utils::display_dealer_final_hand(dealer);
                utils::sleep_ms(utils::get_delay_multiplied(500, is_simulation_round));
            } else { log::error!("RUN_ID:{} - Deck empty during dealer hit!", run_timestamp); break; }
        }
        if dealer_value > 21 { dealer.hand.status = HandStatus::Busted; println!("Dealer busts!"); }
        else { dealer.hand.status = HandStatus::Stood; println!("Dealer stands with {}.", dealer_value); }
    } else {
        println!("\nAll player hands resolved before dealer's turn. Dealer does not play further.");
    }

    println!("\n--- Results for Your Hands ---");
    let dealer_final_value_for_comparison = if dealer.hand.status == HandStatus::Busted { 0 } else { dealer.hand.value() };

    if let Some(user_player) = all_players_at_table.get(0) {
        if user_player.is_user {
            for (i, p_hand) in user_player.hands.iter().enumerate() {
                if p_hand.status == HandStatus::Blackjack && your_bj_resolved_this_round {
                    println!("Your Hand {}: Blackjack win (already paid). Bet: ${:.2}", i + 1, p_hand.bet);
                    continue;
                }
                let player_final_val = p_hand.value();
                let mut result_str = format!("Your Hand {} (${:.2}): ", i + 1, p_hand.bet);
                let mut net_change_for_this_hand = 0.0;
                let is_part_of_split_scenario = user_player.hands.len() > 1 || p_hand.actions_taken.iter().any(|a| a.starts_with("split"));
                let is_doubled_this_hand = p_hand.status == HandStatus::Doubled;

                match p_hand.status {
                    HandStatus::Busted => {
                        result_str.push_str("Bust. You lose.");
                        *your_player_balance -= p_hand.bet; net_change_for_this_hand = -p_hand.bet;
                        session_stats.total_losses += 1;
                    }
                    HandStatus::Blackjack => { /* Already handled */ }
                    _ => {
                        if dealer.hand.status == HandStatus::Busted {
                            result_str.push_str("Dealer busts. You win!");
                            *your_player_balance += p_hand.bet; net_change_for_this_hand = p_hand.bet;
                            session_stats.total_wins += 1;
                        } else if player_final_val > dealer_final_value_for_comparison {
                            result_str.push_str(&format!("Win ({} vs {}).", player_final_val, dealer_final_value_for_comparison));
                            *your_player_balance += p_hand.bet; net_change_for_this_hand = p_hand.bet;
                            session_stats.total_wins += 1;
                        } else if player_final_val < dealer_final_value_for_comparison {
                            result_str.push_str(&format!("Lose ({} vs {}).", player_final_val, dealer_final_value_for_comparison));
                            *your_player_balance -= p_hand.bet; net_change_for_this_hand = -p_hand.bet;
                            session_stats.total_losses += 1;
                        } else {
                            result_str.push_str(&format!("Push ({} vs {}).", player_final_val, dealer_final_value_for_comparison));
                            net_change_for_this_hand = 0.0;
                            session_stats.total_pushes += 1;
                        }
                    }
                }
                println!("{}", result_str);
                if is_part_of_split_scenario {
                    session_stats.earnings_from_split_hands += net_change_for_this_hand;
                    session_stats.num_resolved_split_hands += 1;
                }
                if is_doubled_this_hand {
                    session_stats.earnings_from_doubled_hands += net_change_for_this_hand;
                    session_stats.num_resolved_doubled_hands += 1;
                }
            }
            if user_player.hands.len() > 1 {
                session_stats.total_hands_after_splits += user_player.hands.len() as u32;
            }
        }
    }
    println!("Your balance after round: ${:.2}", *your_player_balance);
    utils::sleep_ms(utils::get_delay_multiplied(500, is_simulation_round));
    true
}
