# Riftbound TCG Synergy Optimizer & Deckbuilder

Welcome to the **Riftbound Optimizer**. This repository contains a Rust-based CLI application designed to analyze card synergies and autonomously construct highly competitive, tournament-ready decks for the Riftbound Trading Card Game.

## 🚀 Purpose
The application solves complex deckbuilding math by cross-referencing keywords, mechanical interactions (e.g., Draw, Buff, Damage), game triggers, and specific power/energy costs across the entire card pool. It matches a given "Legend" and "Champion" to build a standard-legal 40-card Main Deck.

## 🧠 Core Architecture
The codebase is compartmentalized into specific modules to handle distinct tasks:
- `src/main.rs`: The CLI entry point (Clap subcommands) and data loader.
- `src/search.rs`: Uses Levenshtein distance and substring matching to reliably find cards even if the user only provides partial names or archetype identifiers.
- `src/models.rs`: Parses the raw JSON into structs and extracts semantic gameplay tags (e.g., "BuffSource", "MightyConsumer").
- `src/engine.rs`: The heart of the optimizer. It calculates raw synergy scores and applies "Meta Archetype" bonuses based on recent tournament trends.
- `src/builder.rs`: The autonomous deck assembly algorithm.
- `src/analyzer.rs`: Formats and prints terminal output, calculating cross-card interactions ("Deep Combos").

## 🛠️ Data Dependency
The application **does not** hardcode card data. It relies on a local `cards.json` file in the project root.
- **Updating**: The user can run `cargo run -- update` to fetch the latest community JSON directly from GitHub/Gists at runtime.

## 📐 Deckbuilding Rules (For Agents)
When modifying the `DeckBuilder` algorithm, you MUST adhere to the following competitive Riftbound principles embedded in the logic:

### 1. CABS (Cards Affect Board State)
Non-interactive cards (e.g., pure draw spells) are heavily penalized in scoring. The builder prioritizes Units, Gear, and Spells that actively deal damage, buff, or remove pieces from the board.

### 2. The SBREAD Priority Queue
The deck is constructed sequentially using the SBREAD priority system. Cards are tagged and added to the deck in this exact order to ensure competitive viability:
*   **S (Synergy)**: Top 8 highest raw synergy core cards.
*   **B (Bombs)**: Heavy hitters (6+ Energy cost). Max 4 per deck.
*   **R (Removal)**: Board clear or target destruction. Max 6 per deck.
*   **E (Evasion)**: Cards with `[Ganking]`, `[Deflect]`, or `[Hidden]`. Max 5 per deck.
*   **A (Aggro)**: 1-2 drop units that can contest early battlefields (`[Assault]`). Max 8 per deck.
*   **D (Dump)**: Card draw/refill and filler. Max 4 per deck.

### 3. Dynamic Copy Strategy
The algorithm does not blindly add 3 copies of every card. It scales based on the card's SBREAD role:
*   **3x Copies**: Core synergy pieces and Aggro units (maximum consistency).
*   **2x Copies**: Evasion, Removal, and Dump cards (seen roughly once per game).
*   **1x Copies**: Bombs (massive finishers; only want to see once per set, avoiding opening hand bricks).

### 4. Power Tracking & Ramp
Riftbound uses both Energy (E) and Power (P). The algorithm tracks the total Power cost of the deck being built. "Ramp" cards (e.g., `Seal of Strength`) are conditionally included *only* if the total deck Power requirement is projected to be high (> 10 Power).

### 5. Collection Filtering
Users can pass a `--collection <FILE>` flag. The builder must strictly intersect its optimal choices with the quantities the user actually owns.

## ⚠️ Version Control System (VCS) Rule
**CRITICAL**: This repository uses **Jujutsu (`jj`)**, not Git. 
- Never use `git commit`, `git add`, etc.
- Always use `jj new` to create empty commits.
- Always use `jj desc -m "message"` to describe your work.
- Keep commits atomic and logical.