# Riftbound Synergy Optimizer

A lightning-fast, terminal-based CLI tool for the **Riftbound Trading Card Game**.

The optimizer fetches the latest community card database, analyzes mechanical text, and calculates deep, multi-level combos to help you discover the ultimate synergies for your favorite Legend and Champion. It also features a fully autonomous deckbuilder that crafts 40-card, tournament-ready lists from scratch.

## 🚀 Features

- **Deep Synergy Scanning**: Discover hidden combos by evaluating keywords, interaction triggers, and combat mechanics across the entire card pool.
- **Autonomous Deckbuilder**: Automatically generates mathematically optimized, standard-legal decks tailored to your chosen Champion.
- **Collection Filtering**: Import your own card collection (e.g., from Piltover Archive) to ensure the CLI only suggests decks using cards you actually own.
- **Fuzzy Matching**: Don't know the exact name? Just type `--legend "Fiora"` and let the engine figure out the rest.

## 📦 Installation

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

```bash
git clone https://github.com/yourusername/riftbound_optimizer.git
cd riftbound_optimizer
```

## 🛠️ Usage

### 1. Update the Database

The CLI does not hardcode data. Before your first run (or whenever a new set drops), fetch the latest community JSON directly to your local machine:

```bash
cargo run -- update
```

### 2. Optimize Synergies

Find the most synergistic cards for a specific combo. The output details rules text, power costs, and cross-card "Deep Combos."

```bash
cargo run -- optimize --legend "Might of Demacia - Starter" --champion "Fiora, Victorious"
```

### 3. Build a Deck

Autonomously generate a 40-card deck with an optimized Energy/Power curve.

```bash
cargo run -- deck --legend "Swift Scout" --champion "Teemo, Strategist"
```

### Filtering by Collection

Export your card list into a text file (e.g., `my_collection.txt`):

```text
3x Vanguard Captain
2x Call to Glory
1x Fiora, Victorious
```

Pass the file to the CLI to strict-filter the suggestions:

```bash
cargo run -- deck --legend "Might of Demacia" --champion "Fiora" --collection my_collection.txt
```

## 🤝 Contributing

Contributions are always welcome! If you'd like to improve the synergy algorithms or add features:

1. **Fork the repository** on GitHub.
2. **Clone your fork** locally.
3. **Create a new branch** for your feature or bugfix (`feat/my-new-feature`).
4. **Make your changes** and commit them using [Conventional Commits](https://www.conventionalcommits.org/).
5. **Push your branch** to your fork.
6. **Open a Pull Request** against the `main` branch of the original repository.

Please ensure your code passes `cargo check` and `cargo test` before submitting!

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

