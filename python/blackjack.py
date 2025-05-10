import random
import time
import logging
import os
from pathlib import Path
try:
    import matplotlib.pyplot as plt
    MATPLOTLIB_AVAILABLE = True
except ImportError:
    MATPLOTLIB_AVAILABLE = False

# --- Game Configuration (UNCHANGED) ---
NUM_DECKS = 6
RESHUFFLE_THRESHOLD_RATIO = 0.25
NUM_PLAYERS = 3 # Total players at the table including "You" (Player 1). Min 1.

STARTING_BALANCE = 1000.00
DEFAULT_BET = 25.00
MIN_BET_ALLOWED = 1.00

BLACKJACK_PAYOUT_NUMERATOR = 6
BLACKJACK_PAYOUT_DENOMINATOR = 5
BLACKJACK_PAYOUT_MULTIPLIER = BLACKJACK_PAYOUT_NUMERATOR / BLACKJACK_PAYOUT_DENOMINATOR

DEFAULT_SIM_ITERATIONS = 1000
LOGS_DIR = Path("logs")
TEXT_LOG_FILENAME = "results.log"

# --- Global Game State and Stats (UNCHANGED) ---
shoe = []
initial_shoe_size = 0
player_balance = 0.0
results_logger = None
stats = {}
RUN_TIMESTAMP = int(time.time())

def reset_session_stats():
    global stats
    stats = {
        "hands_played_session": 0, "blackjacks_dealt_player": 0,
        "times_split_chosen": 0, "hands_involved_in_split": 0, "total_hands_after_splits": 0,
        "times_doubled_chosen": 0, "hands_involved_in_double": 0,
        "total_wins": 0, "total_losses": 0, "total_pushes": 0,
        "earnings_from_split_hands": 0.0, "num_resolved_split_hands": 0,
        "earnings_from_doubled_hands": 0.0, "num_resolved_doubled_hands": 0,
        "initial_default_bet": DEFAULT_BET, "initial_balance": 0.0, "final_balance": 0.0,
        "highest_balance_session": 0.0, "lowest_balance_session": float('inf')
    }

# --- Logging Setup (UNCHANGED) ---
def setup_results_logger():
    global results_logger, LOGS_DIR, TEXT_LOG_FILENAME, RUN_TIMESTAMP
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    log_file_path = LOGS_DIR / TEXT_LOG_FILENAME
    results_logger = logging.getLogger('blackjack_results')
    results_logger.setLevel(logging.INFO)
    if not results_logger.handlers:
        fh = logging.FileHandler(log_file_path, mode='a')
        fh.setLevel(logging.INFO)
        formatter = logging.Formatter(f'%(asctime)s - RUN_ID:{RUN_TIMESTAMP} - %(message)s', datefmt='%Y-%m-%d %H:%M:%S')
        fh.setFormatter(formatter)
        results_logger.addHandler(fh)
    return results_logger

# --- Graph Generation (UNCHANGED) ---
def generate_balance_graph(balance_history, run_timestamp, logs_dir_path):
    if not MATPLOTLIB_AVAILABLE:
        msg = "\nMatplotlib not found. Skipping graph generation. Install with: pip install matplotlib"
        print(msg);
        if results_logger: results_logger.warning("Matplotlib not found. Balance graph not generated.")
        return
    if not balance_history or len(balance_history) < 2:
        msg = "\nNot enough data to generate balance graph."
        print(msg)
        if results_logger: results_logger.info("Not enough data for balance graph.")
        return

    plt.figure(figsize=(12, 6))
    plt.plot(range(len(balance_history)), balance_history, marker='.', linestyle='-')
    plt.title(f"Your Balance Over Simulation (Run ID: {run_timestamp})")
    plt.xlabel("Hand Number (or Initial State at 0)")
    plt.ylabel("Your Balance ($)")
    plt.grid(True)
    if balance_history:
        plt.axhline(y=balance_history[0], color='r', linestyle='--', label=f'Starting Balance (${balance_history[0]:.2f})')
        plt.legend()
    graph_filename = f"{run_timestamp}.png"
    graph_file_path = logs_dir_path / graph_filename
    try:
        plt.savefig(graph_file_path)
        print(f"\nBalance graph saved to: {graph_file_path}")
        if results_logger: results_logger.info(f"Balance graph saved: {graph_file_path}")
    except Exception as e:
        print(f"\nError saving graph: {e}")
        if results_logger: results_logger.error(f"Error saving balance graph: {e}")
    plt.close()


# --- Utility Functions (get_delay_time) (UNCHANGED) ---
# --- Card and Deck (create_and_shuffle_shoe, deal_card) (UNCHANGED) ---
# --- Hand Logic (calculate_hand_value, is_soft_hand, display_cards_generic, display_your_hands_and_dealer) (UNCHANGED) ---
# --- Basic Strategy (get_basic_strategy_action) (UNCHANGED) ---
# --- Betting Logic (get_your_bet) (UNCHANGED) ---
# ... (Paste these unchanged functions here) ...
def get_delay_time(base_delay, simulation_active):
    return base_delay * 0.02 if simulation_active else base_delay

SUITS = ['♥', '♦', '♣', '♠']
RANKS = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A']
CARD_VALUES = {'2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9, '10': 10,
               'J': 10, 'Q': 10, 'K': 10, 'A': 11}

def create_and_shuffle_shoe(simulation_active=False):
    global shoe, initial_shoe_size
    shoe.clear()
    for _ in range(NUM_DECKS):
        for suit in SUITS:
            for rank in RANKS:
                shoe.append({'rank': rank, 'suit': suit})
    random.shuffle(shoe)
    initial_shoe_size = len(shoe)
    print(f"\n--- A new {NUM_DECKS}-deck shoe with {initial_shoe_size} cards has been shuffled. ---")
    time.sleep(get_delay_time(0.5, simulation_active))

def deal_card(simulation_active=False):
    global shoe
    if not shoe:
        print("SHOE IS EMPTY! This should not happen if reshuffle logic is correct.")
        create_and_shuffle_shoe(simulation_active)
        if not shoe: raise Exception("Failed to create or deal from shoe.")
    return shoe.pop()

def calculate_hand_value(hand_cards):
    value = 0
    num_aces = 0
    for card in hand_cards:
        rank = card['rank']
        value += CARD_VALUES[rank]
        if rank == 'A': num_aces += 1
    while value > 21 and num_aces > 0:
        value -= 10
        num_aces -= 1
    return value

def is_soft_hand(hand_cards):
    current_value = calculate_hand_value(hand_cards)
    if not any(card['rank'] == 'A' for card in hand_cards): return False
    value_if_all_aces_are_1 = sum(1 if c['rank'] == 'A' else CARD_VALUES[c['rank']] for c in hand_cards)
    return current_value > value_if_all_aces_are_1 and current_value <= 21

def display_cards_generic(cards_list, hide_one_idx=None, player_name="Player"):
    display_str = f"{player_name}'s hand: "
    cards_to_show = []
    for i, card in enumerate(cards_list):
        if hide_one_idx == i:
            cards_to_show.append("[Hidden Card]")
        else:
            cards_to_show.append(f"{card['rank']}{card['suit']}")
    display_str += ", ".join(cards_to_show)
    if hide_one_idx is None or (player_name != "Dealer" and hide_one_idx is not None):
        if cards_list : display_str += f" (Value: {calculate_hand_value(cards_list)})"
    elif player_name == "Dealer" and hide_one_idx is not None and len(cards_list) > 0 :
         # Determine the upcard based on hide_one_idx (0 is upcard if hole card is cards[1])
         upcard_actual_index = 0 if hide_one_idx == 1 else (1 if hide_one_idx == 0 and len(cards_list) > 1 else 0)
         if upcard_actual_index < len(cards_list):
            upcard_rank = cards_list[upcard_actual_index]['rank']
            upcard_value_str = str(CARD_VALUES[upcard_rank]) if upcard_rank != 'A' else "1/11"
            display_str += f" (Showing: {upcard_value_str})"
         else: # Should not happen with correct logic
            display_str += " (Showing: ?)"

    print(display_str)

def display_your_hands_and_dealer(your_player_obj, dealer_obj, active_player_is_you=True):
    print("-" * 30)
    # Dealer's upcard is cards[0], hole card is cards[1] if active_player_is_you is True
    display_cards_generic(dealer_obj['cards'], hide_one_idx=1 if active_player_is_you else None, player_name="Dealer")

    for i, p_hand in enumerate(your_player_obj['hands']):
        hand_val_str = f" (Value: {calculate_hand_value(p_hand['cards'])})"
        status_str = f" [{p_hand['status'].upper()}]" if p_hand['status'] not in ['active', 'blackjack'] else ""
        bet_str = f" Bet: ${p_hand['bet']:.2f}"
        cards_str = ", ".join([f"{c['rank']}{c['suit']}" for c in p_hand['cards']])
        active_marker = "*" if p_hand['status'] == 'active' and active_player_is_you else " "
        print(f"{active_marker}{your_player_obj['name']} Hand {i+1}: {cards_str}{hand_val_str}{bet_str}{status_str}")

def get_basic_strategy_action(p_hand_cards_list, dealer_upcard_value, num_total_player_hands_for_this_player, simulation_active=False):
    player_value = calculate_hand_value(p_hand_cards_list)
    is_pair = len(p_hand_cards_list) == 2 and CARD_VALUES[p_hand_cards_list[0]['rank']] == CARD_VALUES[p_hand_cards_list[1]['rank']]
    is_soft = is_soft_hand(p_hand_cards_list)
    can_double_check = len(p_hand_cards_list) == 2
    can_split_check = is_pair and num_total_player_hands_for_this_player < 4

    if can_split_check:
        card_rank = p_hand_cards_list[0]['rank']
        if card_rank == 'A' or card_rank == '8': return 'P'
        if card_rank == '9':
            if dealer_upcard_value not in [7, 10, 11]: return 'P'
        if card_rank == '7':
            if dealer_upcard_value <= 7: return 'P'
        if card_rank == '6':
            if dealer_upcard_value <= 6 : return 'P'
        if card_rank == '4':
            if dealer_upcard_value in [5, 6] : return 'P'
        if card_rank == '3' or card_rank == '2':
            if dealer_upcard_value <= 7 : return 'P'

    if is_soft:
        if player_value >= 19: return 'S'
        if player_value == 18:
            if dealer_upcard_value <= 6 and can_double_check : return 'D'
            elif dealer_upcard_value in [2,7,8]: return 'S'
            else: return 'H'
        if player_value == 17:
            if dealer_upcard_value >= 3 and dealer_upcard_value <= 6 and can_double_check: return 'D'
            else: return 'H'
        if player_value == 16:
            if dealer_upcard_value >= 4 and dealer_upcard_value <= 6 and can_double_check: return 'D'
            else: return 'H'
        if player_value == 15:
            if dealer_upcard_value >= 4 and dealer_upcard_value <= 6 and can_double_check: return 'D'
            else: return 'H'
        if player_value <= 14 :
            if dealer_upcard_value >= 5 and dealer_upcard_value <= 6 and can_double_check: return 'D'
            else: return 'H'
        return 'H'

    if player_value >= 17: return 'S'
    if player_value >= 13 and player_value <= 16:
        return 'S' if dealer_upcard_value <= 6 else 'H'
    if player_value == 12:
        return 'S' if dealer_upcard_value >= 4 and dealer_upcard_value <= 6 else 'H'
    if player_value == 11:
        return 'D' if can_double_check else 'H'
    if player_value == 10:
        return 'D' if dealer_upcard_value <= 9 and can_double_check else 'H'
    if player_value == 9:
        return 'D' if dealer_upcard_value >= 2 and dealer_upcard_value <= 6 and can_double_check else 'H'
    return 'H'

def get_your_bet(current_player_balance, simulation_active=False):
    global DEFAULT_BET, MIN_BET_ALLOWED
    if simulation_active:
        if current_player_balance >= DEFAULT_BET:
            print(f"Simulation ('You'): Auto-betting default ${DEFAULT_BET:.2f}")
            return DEFAULT_BET
        else:
            print(f"Simulation ('You'): Balance (${current_player_balance:.2f}) too low for default bet (${DEFAULT_BET:.2f}).")
            return 0
    else:
        if current_player_balance < MIN_BET_ALLOWED:
            print(f"Your balance (${current_player_balance:.2f}) is too low to place any bet (min: ${MIN_BET_ALLOWED:.2f}).")
            return 0
        while True:
            try:
                bet_str = input(f"Your balance: ${current_player_balance:.2f}. Enter bet (or press Enter for default ${DEFAULT_BET:.2f}): ")
                if not bet_str: bet_amount = DEFAULT_BET
                else: bet_amount = float(bet_str)

                if bet_amount < MIN_BET_ALLOWED: print(f"Bet must be at least ${MIN_BET_ALLOWED:.2f}.")
                elif bet_amount > current_player_balance: print(f"You cannot bet more than your current balance (${current_player_balance:.2f}).")
                else: return bet_amount
            except ValueError: print("Invalid input. Please enter a number or press Enter.")
            if not bet_str and DEFAULT_BET > current_player_balance:
                print(f"Default bet (${DEFAULT_BET:.2f}) is higher than your current balance (${current_player_balance:.2f}). Please enter a valid amount or a lower default.")

# --- Game Flow (play_blackjack - CORRECTED) ---
def play_blackjack(simulation_active=False):
    global player_balance, stats, NUM_PLAYERS

    all_players_at_table = []
    your_initial_bet = 0

    if NUM_PLAYERS >= 1:
        your_initial_bet = get_your_bet(player_balance, simulation_active=simulation_active)
        if your_initial_bet == 0: return False
        stats['hands_played_session'] += 1

        your_player_object = {
            'id': 0, 'name': "Your", 'is_user': True,
            'hands': [{'cards': [], 'bet': your_initial_bet, 'status': 'active', 'is_split_ace': False, 'actions_taken': []}],
            'hand_involved_in_split_this_round': False, # Flags specific to "Your" player for this round
            'hand_involved_in_double_this_round': False
        }
        all_players_at_table.append(your_player_object)
    else:
        print("Error: NUM_PLAYERS is less than 1.")
        return False

    for i in range(1, NUM_PLAYERS):
        ai_player_object = {
            'id': i, 'name': f"Player {i+1}", 'is_user': False,
            'hands': [{'cards': [], 'bet': 0, 'status': 'active', 'is_split_ace': False, 'actions_taken': []}]
        }
        all_players_at_table.append(ai_player_object)

    dealer_object = {'name': "Dealer", 'cards': [], 'status': 'active'}

    print("\n--- Dealing Cards ---")
    for p_obj in all_players_at_table:
        p_obj['hands'][0]['cards'].append(deal_card(simulation_active))
    dealer_object['cards'].append(deal_card(simulation_active)) # Dealer upcard
    for p_obj in all_players_at_table:
        p_obj['hands'][0]['cards'].append(deal_card(simulation_active))
    dealer_object['cards'].append(deal_card(simulation_active)) # Dealer hole card

    if NUM_PLAYERS >= 1:
        display_your_hands_and_dealer(all_players_at_table[0], dealer_object)
        for i in range(1, NUM_PLAYERS):
            ai_p = all_players_at_table[i]
            cards_str = ", ".join([f"{c['rank']}{c['suit']}" for c in ai_p['hands'][0]['cards']])
            val = calculate_hand_value(ai_p['hands'][0]['cards'])
            print(f"{ai_p['name']}: {cards_str} (Value: {val})")


    your_bj_resolved = False
    if NUM_PLAYERS >= 1 and calculate_hand_value(all_players_at_table[0]['hands'][0]['cards']) == 21:
        all_players_at_table[0]['hands'][0]['status'] = 'blackjack'
        stats['blackjacks_dealt_player'] += 1
        print(f"\n{all_players_at_table[0]['name']} Blackjack!")
        time.sleep(get_delay_time(0.2, simulation_active))
        dealer_val_for_bj_check = calculate_hand_value(dealer_object['cards'])
        if dealer_val_for_bj_check == 21 and len(dealer_object['cards']) == 2:
            print("Dealer also has Blackjack! It's a Push for Your hand.")
            stats['total_pushes'] += 1
        else:
            winnings = your_initial_bet * BLACKJACK_PAYOUT_MULTIPLIER
            player_balance += winnings
            stats['total_wins'] +=1
            print(f"You win ${winnings:.2f}! (Payout: {BLACKJACK_PAYOUT_NUMERATOR}/{BLACKJACK_PAYOUT_DENOMINATOR})")
        print(f"Your balance: ${player_balance:.2f}")
        your_bj_resolved = True

    for p_idx, current_player_obj in enumerate(all_players_at_table):
        if current_player_obj['is_user'] and your_bj_resolved:
            continue
        if not current_player_obj['is_user'] and calculate_hand_value(current_player_obj['hands'][0]['cards']) == 21:
            print(f"\n{current_player_obj['name']} has Blackjack.")
            current_player_obj['hands'][0]['status'] = 'blackjack'
            continue

        print(f"\n--- {current_player_obj['name']}'s Turn ---")
        if current_player_obj['is_user']:
             time.sleep(get_delay_time(0.5, simulation_active))

        current_hand_idx_for_player = 0
        auto_play_book_all_hands_this_round_for_you = simulation_active if current_player_obj['is_user'] else False

        while current_hand_idx_for_player < len(current_player_obj['hands']):
            p_hand = current_player_obj['hands'][current_hand_idx_for_player]
            if p_hand['status'] != 'active':
                current_hand_idx_for_player += 1
                continue

            hand_turn_over = False
            while not hand_turn_over and p_hand['status'] == 'active':
                if current_player_obj['is_user']:
                    display_your_hands_and_dealer(current_player_obj, dealer_object)
                else:
                    cards_str = ", ".join([f"{c['rank']}{c['suit']}" for c in p_hand['cards']])
                    val = calculate_hand_value(p_hand['cards'])
                    dealer_up_str = f"{dealer_object['cards'][0]['rank']}{dealer_object['cards'][0]['suit']}"
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1}: {cards_str} (Value: {val}) vs Dealer Up: {dealer_up_str}")

                player_value_of_current_hand = calculate_hand_value(p_hand['cards'])
                dealer_upcard_value = CARD_VALUES[dealer_object['cards'][0]['rank']]

                if player_value_of_current_hand > 21:
                    p_hand['status'] = 'busted'
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} Busts!")
                    hand_turn_over = True
                    break

                can_double = (len(p_hand['cards']) == 2 and (not current_player_obj['is_user'] or player_balance >= p_hand['bet']))
                can_split = (len(p_hand['cards']) == 2 and
                             CARD_VALUES[p_hand['cards'][0]['rank']] == CARD_VALUES[p_hand['cards'][1]['rank']] and
                             (not current_player_obj['is_user'] or player_balance >= p_hand['bet']) and
                             len(current_player_obj['hands']) < 4 and not p_hand['is_split_ace'])

                chosen_action_code = ''
                is_book_play_for_this_action = (current_player_obj['is_user'] and auto_play_book_all_hands_this_round_for_you) \
                                            or (not current_player_obj['is_user']) \
                                            or (current_player_obj['is_user'] and p_hand.get('book_play_this_hand', False) and not simulation_active)

                if is_book_play_for_this_action:
                    chosen_action_code = get_basic_strategy_action(p_hand['cards'], dealer_upcard_value, len(current_player_obj['hands']), simulation_active)
                    if chosen_action_code == 'D' and not can_double: chosen_action_code = 'H'
                    if chosen_action_code == 'P' and not can_split:
                        chosen_action_code = get_basic_strategy_action(p_hand['cards'], dealer_upcard_value, 10, simulation_active)
                        if chosen_action_code == 'P': chosen_action_code = 'H'
                    action_desc = {'H': "Hit", 'S': "Stand", 'D': "Double Down", 'P': "Split"}.get(chosen_action_code, "Unknown")
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} (Book): {action_desc}")
                    time.sleep(get_delay_time(0.7, simulation_active))
                elif current_player_obj['is_user']:
                    action_prompt = "Choose action for Your Hand " + str(current_hand_idx_for_player + 1) + ": (H)it, (S)tand"
                    valid_actions_input = ['h', 's']
                    if can_double: action_prompt += ", (D)ouble"; valid_actions_input.append('d')
                    if can_split: action_prompt += ", (P)Split"; valid_actions_input.append('p')
                    if not auto_play_book_all_hands_this_round_for_you:
                         action_prompt += ", (B)ook plays all your hands"; valid_actions_input.append('b')
                    action_prompt += ": "
                    user_input = input(action_prompt).lower()

                    if user_input == 'b':
                        auto_play_book_all_hands_this_round_for_you = True
                        p_hand['book_play_this_hand'] = True
                        print("Book will play out all your remaining hands for this round.")
                        chosen_action_code = get_basic_strategy_action(p_hand['cards'], dealer_upcard_value, len(current_player_obj['hands']), simulation_active)
                        if chosen_action_code == 'D' and not can_double: chosen_action_code = 'H'
                        if chosen_action_code == 'P' and not can_split:
                            chosen_action_code = get_basic_strategy_action(p_hand['cards'], dealer_upcard_value, 10, simulation_active)
                            if chosen_action_code == 'P': chosen_action_code = 'H'
                        action_desc = {'H': "Hit", 'S': "Stand", 'D': "Double Down", 'P': "Split"}.get(chosen_action_code, "Unknown")
                        print(f"You (Book takes over Hand {current_hand_idx_for_player+1}): {action_desc}")
                        time.sleep(get_delay_time(0.7, simulation_active))
                    elif user_input in valid_actions_input:
                        if user_input == 'h': chosen_action_code = 'H'
                        elif user_input == 's': chosen_action_code = 'S'
                        elif user_input == 'd' and can_double: chosen_action_code = 'D'
                        elif user_input == 'p' and can_split: chosen_action_code = 'P'
                        else: print("Invalid action or action not allowed."); continue
                    else: print("Invalid input."); continue
                else: chosen_action_code = 'S'

                if chosen_action_code == 'H':
                    p_hand['cards'].append(deal_card(simulation_active))
                    drawn_card_str = f"{p_hand['cards'][-1]['rank']}{p_hand['cards'][-1]['suit']}"
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} Hits, draws {drawn_card_str}")
                    if calculate_hand_value(p_hand['cards']) == 21 and p_hand['is_split_ace']:
                        p_hand['status'] = 'stood'; hand_turn_over = True
                elif chosen_action_code == 'S':
                    p_hand['status'] = 'stood'
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} Stands.")
                    hand_turn_over = True
                elif chosen_action_code == 'D':
                    if current_player_obj['is_user']:
                        stats['times_doubled_chosen'] += 1
                        # Use the flag from the current_player_obj (which is 'Your' player object)
                        if not current_player_obj['hand_involved_in_double_this_round']:
                            stats['hands_involved_in_double'] += 1
                            current_player_obj['hand_involved_in_double_this_round'] = True # Set the flag on 'Your' object
                        p_hand['bet'] *= 2
                    p_hand['cards'].append(deal_card(simulation_active))
                    p_hand['status'] = 'doubled'
                    bet_display = f"Bet is now ${p_hand['bet']:.2f}. " if current_player_obj['is_user'] else ""
                    drawn_card_str = f"{p_hand['cards'][-1]['rank']}{p_hand['cards'][-1]['suit']}"
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} Doubles Down. {bet_display}Draws {drawn_card_str}")
                    if calculate_hand_value(p_hand['cards']) > 21:
                        p_hand['status'] = 'busted'
                        print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} Busts after doubling!")
                    hand_turn_over = True
                elif chosen_action_code == 'P':
                    if current_player_obj['is_user']:
                        stats['times_split_chosen'] += 1
                        # Use the flag from the current_player_obj
                        if not current_player_obj['hand_involved_in_split_this_round']:
                            stats['hands_involved_in_split'] += 1
                            current_player_obj['hand_involved_in_split_this_round'] = True # Set flag on 'Your' object
                    split_card_val = p_hand['cards'][0]['rank']
                    original_second_card = p_hand['cards'].pop()
                    new_hand_for_player = {
                        'cards': [original_second_card],
                        'bet': p_hand['bet'] if current_player_obj['is_user'] else 0,
                        'status': 'active',
                        'is_split_ace': (split_card_val == 'A'),
                        'actions_taken': ['split_from_hand_' + str(current_hand_idx_for_player+1)]
                    }
                    p_hand['is_split_ace'] = (split_card_val == 'A')
                    p_hand['actions_taken'].append('split_created_new_hand')
                    p_hand['cards'].append(deal_card(simulation_active))
                    new_hand_for_player['cards'].append(deal_card(simulation_active))
                    current_player_obj['hands'].insert(current_hand_idx_for_player + 1, new_hand_for_player)
                    print(f"{current_player_obj['name']} Hand {current_hand_idx_for_player+1} splits. New Hand {current_hand_idx_for_player+2} created.")
                    time.sleep(get_delay_time(0.5, simulation_active))
                    if p_hand['is_split_ace']:
                        p_hand['status'] = 'stood'
                        if calculate_hand_value(p_hand['cards']) == 21: print(f"Split Ace Hand {current_hand_idx_for_player+1} has 21, stands.")
                        else: print(f"Split Ace Hand {current_hand_idx_for_player+1} stands.")
                time.sleep(get_delay_time(0.3, simulation_active))
            current_hand_idx_for_player += 1

    any_player_hand_needs_dealer_play = False
    for p_obj in all_players_at_table:
        for hand_data in p_obj['hands']:
            if hand_data['status'] in ['active', 'stood', 'doubled']:
                any_player_hand_needs_dealer_play = True; break
        if any_player_hand_needs_dealer_play: break

    if any_player_hand_needs_dealer_play:
        print("\n--- Dealer's Turn ---")
        time.sleep(get_delay_time(1.0, simulation_active))
        display_cards_generic(dealer_object['cards'], player_name="Dealer")
        dealer_value = calculate_hand_value(dealer_object['cards'])
        while dealer_value < 17:
            print("Dealer hits...")
            time.sleep(get_delay_time(1.0, simulation_active))
            dealer_object['cards'].append(deal_card(simulation_active))
            dealer_value = calculate_hand_value(dealer_object['cards'])
            display_cards_generic(dealer_object['cards'], player_name="Dealer")
            time.sleep(get_delay_time(0.5, simulation_active))
        if dealer_value > 21: dealer_object['status'] = 'busted'; print("Dealer busts!")
        else: dealer_object['status'] = 'stood'; print(f"Dealer stands with {dealer_value}.")
    else: print("\nAll player hands resolved before dealer's turn. Dealer does not play further.")

    print("\n--- Results for Your Hands ---")
    dealer_final_value = calculate_hand_value(dealer_object['cards']) if dealer_object['status'] != 'busted' else -1

    if NUM_PLAYERS >= 1 :
        your_player_obj = all_players_at_table[0]
        for i, p_hand in enumerate(your_player_obj['hands']):
            if p_hand['status'] == 'blackjack' and your_bj_resolved:
                print(f"Your Hand {i+1} (${p_hand['bet']:.2f}): Blackjack win (already paid).")
                continue

            player_final_value = calculate_hand_value(p_hand['cards'])
            result_str = f"Your Hand {i+1} (${p_hand['bet']:.2f}): "
            net_change_for_hand = 0
            is_part_of_split = any('split' in action for action in p_hand['actions_taken']) or len(your_player_obj['hands']) > 1
            is_doubled_hand = p_hand['status'] == 'doubled'

            if p_hand['status'] == 'busted':
                result_str += "Bust. You lose."
                player_balance -= p_hand['bet']; net_change_for_hand = -p_hand['bet']
                stats['total_losses'] += 1
            elif dealer_object['status'] == 'busted':
                result_str += "Dealer busts. You win!"
                player_balance += p_hand['bet']; net_change_for_hand = p_hand['bet']
                stats['total_wins'] += 1
            elif player_final_value > dealer_final_value:
                result_str += f"Win ({player_final_value} vs {dealer_final_value})."
                player_balance += p_hand['bet']; net_change_for_hand = p_hand['bet']
                stats['total_wins'] += 1
            elif player_final_value < dealer_final_value:
                result_str += f"Lose ({player_final_value} vs {dealer_final_value})."
                player_balance -= p_hand['bet']; net_change_for_hand = -p_hand['bet']
                stats['total_losses'] += 1
            else:
                result_str += f"Push ({player_final_value} vs {dealer_final_value})."
                net_change_for_hand = 0
                stats['total_pushes'] += 1
            print(result_str)

            if is_part_of_split :
                stats['earnings_from_split_hands'] += net_change_for_hand
                stats['num_resolved_split_hands'] += 1
            if is_doubled_hand: # This check correctly refers to p_hand's status
                stats['earnings_from_doubled_hands'] += net_change_for_hand
                stats['num_resolved_doubled_hands'] += 1

        if len(your_player_obj['hands']) > 1:
            stats["total_hands_after_splits"] += len(your_player_obj['hands'])

    print(f"Your balance after hand: ${player_balance:.2f}")
    return True

# --- Main Game Loop (UNCHANGED from previous version with graph) ---
# ... (Paste the unchanged get_num_iterations and if __name__ == "__main__": block here) ...
def get_num_iterations():
    global DEFAULT_SIM_ITERATIONS
    while True:
        try:
            iterations_str = input(f"Enter number of simulation iterations (or press Enter for default {DEFAULT_SIM_ITERATIONS}): ")
            if not iterations_str: return DEFAULT_SIM_ITERATIONS
            num_iter = int(iterations_str)
            if num_iter > 0: return num_iter
            else: print("Number of iterations must be positive.")
        except ValueError: print("Invalid input. Please enter a number.")


if __name__ == "__main__":
    if NUM_PLAYERS < 1:
        print("Error: NUM_PLAYERS must be 1 or greater.")
        exit()

    start_time = time.time()
    logger = setup_results_logger()
    reset_session_stats()

    print(f"--- Welcome to {NUM_DECKS}-Deck Blackjack! (RUN ID: {RUN_TIMESTAMP}) ---")
    print(f"Total Players at Table (incl. You): {NUM_PLAYERS}")
    print(f"Blackjack Payout: {BLACKJACK_PAYOUT_NUMERATOR}/{BLACKJACK_PAYOUT_DENOMINATOR}")
    if not MATPLOTLIB_AVAILABLE:
        print("Note: Matplotlib library not found. Balance graph will not be generated for simulations.")
        print("You can install it with: pip install matplotlib")
    logger.info(f"--- New Game Session Started (Payout: {BLACKJACK_PAYOUT_NUMERATOR}/{BLACKJACK_PAYOUT_DENOMINATOR}, Total Players: {NUM_PLAYERS}) ---")

    game_mode = ''
    while game_mode not in ['i', 's']:
        game_mode = input("Choose mode: (i)nteractive or (s)imulation (for 'Your' play)? ").lower()

    player_balance = STARTING_BALANCE
    stats['initial_balance'] = STARTING_BALANCE
    stats['highest_balance_session'] = STARTING_BALANCE
    stats['lowest_balance_session'] = STARTING_BALANCE

    logger.info(f"Mode Selected (for 'Your' play): {'Simulation (Book)' if game_mode == 's' else 'Interactive'}")
    logger.info(f"Starting Balance (You): ${STARTING_BALANCE:.2f}")
    logger.info(f"Configured Default Bet (You): ${DEFAULT_BET:.2f}")

    if game_mode == 's':
        num_iterations = get_num_iterations()
        print(f"\nStarting simulation for {num_iterations} hands. 'You' will play by Book strategy.")
        logger.info(f"Simulation Target Iterations: {num_iterations}")
        
        balance_history_sim = [player_balance]

        create_and_shuffle_shoe(simulation_active=True)

        for i in range(num_iterations):
            print(f"\n--- Sim Hand #{i + 1}/{num_iterations} | Your Bal: ${player_balance:.2f} ---")
            if len(shoe) < initial_shoe_size * RESHUFFLE_THRESHOLD_RATIO:
                create_and_shuffle_shoe(simulation_active=True)

            if player_balance < DEFAULT_BET:
                msg = f"Your Balance (${player_balance:.2f}) < Default Bet (${DEFAULT_BET:.2f}). Sim ends."
                print(msg); logger.warning(f"Sim ended early at hand {i+1}: {msg}")
                break
            
            if not play_blackjack(simulation_active=True):
                msg = "Could not place Your bet. Sim ends."
                print(msg); logger.warning(f"Sim ended early at hand {i+1}: {msg}")
                break
            
            balance_history_sim.append(player_balance)
            stats['highest_balance_session'] = max(stats['highest_balance_session'], player_balance)
            stats['lowest_balance_session'] = min(stats['lowest_balance_session'], player_balance)

            if player_balance < MIN_BET_ALLOWED :
                msg = f"Your Balance (${player_balance:.2f}) < Min Bet (${MIN_BET_ALLOWED:.2f}). Sim ends."
                print(msg); logger.warning(f"Sim ended early at hand {i+1}: {msg}")
                break
       
        elapsed_time = time.time() - start_time
        time_per_hand = elapsed_time/stats['hands_played_session']

        generate_balance_graph(balance_history_sim, RUN_TIMESTAMP, LOGS_DIR)
        stats['final_balance'] = player_balance
        stats['net_profit_loss'] = player_balance - STARTING_BALANCE
        if stats['hands_played_session'] > 0:
            stats['avg_earn_loss_per_main_hand'] = stats['net_profit_loss'] / stats['hands_played_session']
        if stats['num_resolved_split_hands'] > 0:
            stats['avg_earn_loss_per_split_hand_part'] = stats['earnings_from_split_hands'] / stats['num_resolved_split_hands']
        if stats['num_resolved_doubled_hands'] > 0:
            stats['avg_earn_loss_per_doubled_hand'] = stats['earnings_from_doubled_hands'] / stats['num_resolved_doubled_hands']

        print("\n\n--- Simulation Finished (Your Play: Book) ---")
        logger.info("--- Simulation Results (Your Play: Book) ---")
        
        results_lines = [
            f"Run ID: {RUN_TIMESTAMP}",
            f"Mode: Simulation (Your Play: Book, AI Players: Book)",
            f"Target Iterations: {num_iterations}",
            f"Hands Played by You (Main): {stats['hands_played_session']}",
            f"Starting Balance (You): ${stats['initial_balance']:.2f}",
            f"Final Balance (You):    ${stats['final_balance']:.2f}",
            f"Highest Balance (You): ${stats['highest_balance_session']:.2f}",
            f"Lowest Balance (You):  ${stats['lowest_balance_session']:.2f}",
            f"Default Bet Used (You): ${stats['initial_default_bet']:.2f}",
            f"Net Profit/Loss (You):  ${stats.get('net_profit_loss',0):+.2f}",
            f"Avg. P/L per Main Hand (You): ${stats.get('avg_earn_loss_per_main_hand', 0):+.2f}",
            f"Your Blackjacks: {stats['blackjacks_dealt_player']}",
            f"Your Times 'Split' Chosen: {stats['times_split_chosen']}",
            f"Your Original Hands Involving a Split: {stats['hands_involved_in_split']}",
            f"Your Total Individual Hands from Splits: {stats['total_hands_after_splits']}",
            f"Your Net P/L from All Split Hand Parts: ${stats['earnings_from_split_hands']:+.2f}",
            f"Your Avg. P/L per Individual Split Hand Part: ${stats.get('avg_earn_loss_per_split_hand_part', 0):+.2f} (from {stats['num_resolved_split_hands']} parts)",
            f"Your Times 'Double Down' Chosen: {stats['times_doubled_chosen']}",
            f"Your Hands Involving a Double Down: {stats['hands_involved_in_double']}",
            f"Your Net P/L from Doubled Hands: ${stats['earnings_from_doubled_hands']:+.2f}",
            f"Your Avg. P/L per Doubled Hand: ${stats.get('avg_earn_loss_per_doubled_hand', 0):+.2f} (from {stats['num_resolved_doubled_hands']} hands)",
            f"Your Total Wins: {stats['total_wins']}, Losses: {stats['total_losses']}, Pushes: {stats['total_pushes']}",
            "Strategy for All AI Players: 'Book' (H17, 6D, DAS based)",
            f"--- Timing Statistics ---",
            f"Total script execution time: {elapsed_time:.3f} seconds.",
            f"Average time per main hand played by You: {time_per_hand:.3f} seconds.",
            f"--- Session Ended (RUN ID: {RUN_TIMESTAMP}) ---",
        ]
        for line in results_lines:
            print(line)
            logger.info(line)
        logger.info("--- Simulation Session Ended ---")

    else: # Interactive Mode
        print(f"\nStarting interactive game for 'You'. Other {NUM_PLAYERS-1 if NUM_PLAYERS > 1 else 0} player(s) will play by Book.")
        create_and_shuffle_shoe(simulation_active=False)
        balance_history_interactive = [player_balance]

        while True:
            if player_balance < MIN_BET_ALLOWED:
                msg = f"\nYour balance (${player_balance:.2f}) is too low. Game over!"
                print(msg); logger.info(msg)
                break
            
            if not play_blackjack(simulation_active=False):
                msg = "Could not play Your hand (likely insufficient funds). Game over."
                print(msg); logger.info(msg)
                break

            balance_history_interactive.append(player_balance)
            stats['highest_balance_session'] = max(stats['highest_balance_session'], player_balance)
            stats['lowest_balance_session'] = min(stats['lowest_balance_session'], player_balance)

            if input("\nPlay another hand? (y/n): ").lower() != 'y':
                break
        
        stats['final_balance'] = player_balance
        print(f"\n--- Interactive Session Ended ---")
        logger.info("--- Interactive Session Results (Your Play: Interactive) ---")
        final_msg = "Thanks for playing!"
        print(final_msg); logger.info(final_msg)
        
        results_lines_interactive = [
            f"Run ID: {RUN_TIMESTAMP}",
            f"Mode: Interactive (Your Play: Manual/Book, AI Players: Book)",
            f"Configured Default Bet (You): ${stats['initial_default_bet']:.2f}",
            f"Starting Balance (You): ${stats['initial_balance']:.2f}",
            f"Final balance (You): ${stats['final_balance']:.2f}",
            f"Highest Balance (You): ${stats['highest_balance_session']:.2f}",
            f"Lowest Balance (You):  ${stats['lowest_balance_session']:.2f}",
            f"Hands Played by You (Main): {stats['hands_played_session']}",
            f"Your Blackjacks: {stats['blackjacks_dealt_player']}",
            f"Your Times Split: {stats['times_split_chosen']}, Your Times Doubled: {stats['times_doubled_chosen']}"
        ]
        for line in results_lines_interactive:
            if "Final balance:" in line and not simulation_active: pass
            elif "Thanks for playing!" in line and not simulation_active: pass
            else: print(line)
            logger.info(line)
        logger.info("--- Interactive Session Ended ---")
