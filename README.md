```markdown
# Blackjack Simulator (Python & Rust)

This project provides a command-line Blackjack simulator implemented in both Python and Rust. It allows users to play interactively or run simulations to observe outcomes based on basic strategy.

## Features

*   **Player vs. Dealer:** Standard Blackjack rules.
*   **Configurable Decks:** Simulates a shoe with a configurable number of decks (default: 6).
*   **Configurable Blackjack Payout:** Default 6:5, can be changed in the code.
*   **Multiple AI Players:** Simulates other players at the table (consuming cards) who play by basic strategy.
*   **Player Actions:**
    *   Hit
    *   Stand
    *   Double Down (on any first two cards, after splits - DAS)
    *   Split Pairs (up to 3 times, making 4 hands; split Aces get one card).
*   **"Follow Book" Mode:**
    *   Player can choose to have their hand played automatically according to a simplified basic strategy.
    *   In simulation mode, "Your" play is always by the book.
*   **Betting & Balance:**
    *   Tracks player balance.
    *   Allows custom bets or a default bet.
*   **Simulation Mode:**
    *   Run a specified number of hands automatically.
    *   Outputs detailed end-of-simulation statistics.
*   **Statistics Tracking:** Records various game metrics, including:
    *   Win/Loss/Push rates
    *   Blackjack counts
    *   Split and Double Down frequencies and P/L
    *   Balance progression (highest/lowest)
    *   Total runtime and average time per hand.
*   **Logging:** Game results and statistics are logged to `logs/results.log` with a run-specific timestamp.
*   **Balance Graph (Simulation Mode):** Generates a PNG graph (`logs/<timestamp>.png`) showing "Your" balance over the course of a simulation.

## Setup and Running

### Python Version

**Prerequisites:**
*   uv (https://docs.astral.sh/uv/)

**Installation (matplotlib):**
```bash
uv python install 3.13
uv venv
source .venv/bin/activate
uv pip install matplotlib
```

**Running:**
1.  Navigate to the python/ directory.
    ```bash
    cd python
    ```
2.  Run the script from your terminal:
    ```bash
    python blackjack.py
    ```
3.  Follow the on-screen prompts:
    *   Choose "(i)nteractive" or "(s)imulation" mode.
    *   If simulation, enter the number of iterations (or press Enter for default).
    *   If interactive, follow prompts for betting and actions (Hit `h`, Stand `s`, Book `b`).

### Rust Version

**Prerequisites:**
*   Rust programming language toolchain (Rustup recommended: [https://rustup.rs/](https://rustup.rs/))
*   Cargo (Rust's package manager, comes with Rustup)

**Setup (Compiling Dependencies):**
1.  Navigate to the rust/ directory.
2.  Build the project and fetch dependencies:
    ```bash
    cargo build
    ```
    (For a release build, which is optimized and faster: `cargo build --release`)

**Running:**
1.  After a successful build, run the compiled executable:
    *   Development build:
        ```bash
        cargo run
        ```
    *   Release build (if you built with `--release`):
        ```bash
        ./target/release/rust_blackjack_simulator 
        ```
        (The executable name might vary based on your `Cargo.toml` `name` field).
2.  Follow the on-screen prompts, similar to the Python version.

## Output

*   **Console:** Displays game progress, player hands, dealer actions, and results.
*   **`logs/` directory:**
    *   `results.log`: A text file appended with detailed statistics and a summary for each game session (interactive or simulation). Each session log includes a unique `RUN_ID` (Unix timestamp).
    *   `<RUN_ID>.png` (e.g., `1678886400.png`): Generated after each simulation run, this image file is a graph plotting "Your" balance over the hands played in that simulation. The filename matches the `RUN_ID` in `results.log`.

## Code Configuration

Both Python and Rust versions have constants defined near the top of their main script/module files (e.g., `config.rs` for Rust, top of Python script) that can be modified to change game parameters:

*   `NUM_DECKS`
*   `NUM_PLAYERS`
*   `STARTING_BALANCE`
*   `DEFAULT_BET`
*   `BLACKJACK_PAYOUT_NUMERATOR` / `BLACKJACK_PAYOUT_DENOMINATOR`
*   `RESHUFFLE_THRESHOLD_RATIO`
*   `DEFAULT_SIM_ITERATIONS`

## Basic Strategy Implemented

The "Book" mode and AI players follow a simplified basic strategy generally aligned with:
*   Dealer Hits Soft 17 (H17)
*   6 Decks
*   Double After Split (DAS) allowed
*   Split up to 4 hands
*   Split Aces get one card

*(The exact strategy rules are implemented in `get_basic_strategy_action` in Python and `src/strategy.rs` in Rust.)*

## Future Enhancements (Potential)

*   More complex betting strategies for simulation.
*   Insurance option.
*   Surrender option.
*   More detailed basic strategy charts for different rule variations.
*   GUI instead of command-line interface.
```

**Key things this README includes:**

*   Project title and brief description.
*   A list of main features.
*   Clear setup and running instructions for both Python and Rust versions, including prerequisites.
*   Description of the output (console, log file, graph).
*   Mention of configurable parameters in the code.
*   A note on the basic strategy being used.
*   A placeholder for potential future enhancements.

You can copy and paste this into a `README.md` file in the root of your project directory. Remember to replace `blackjack_simulator.py` or `rust_blackjack_simulator` with your actual script/executable names if they are different.
