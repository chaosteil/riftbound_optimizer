mod models;
mod engine;

use clap::Parser;
use std::sync::OnceLock;
use models::Card;
use engine::SynergyScorer;

const CARDS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/cards.json"));

static CARDS: OnceLock<Vec<Card>> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about = "Riftbound Synergy Optimizer", long_about = None)]
struct Args {
    #[arg(short, long, help = "Name of the Legend card")]
    legend: String,

    #[arg(short, long, help = "Name of the Champion card")]
    champion: String,
    
    #[arg(long, default_value_t = 15, help = "Number of top results to display")]
    limit: usize,
}

fn load_cards() -> &'static Vec<Card> {
    CARDS.get_or_init(|| {
        serde_json::from_str(CARDS_JSON).expect("Failed to parse embedded JSON")
    })
}

fn main() {
    let args = Args::parse();
    let cards = load_cards();

    // Find the Legend and Champion in the dataset
    let legend = cards.iter().find(|c| c.name.to_lowercase() == args.legend.to_lowercase());
    let champion = cards.iter().find(|c| c.name.to_lowercase() == args.champion.to_lowercase());

    let (legend_card, champion_card) = match (legend, champion) {
        (Some(l), Some(c)) => (l, c),
        (None, _) => {
            eprintln!("Error: Legend '{}' not found in database.", args.legend);
            std::process::exit(1);
        }
        (_, None) => {
            eprintln!("Error: Champion '{}' not found in database.", args.champion);
            std::process::exit(1);
        }
    };

    println!("Found Legend: {}", legend_card.name);
    println!("Found Champion: {}", champion_card.name);
    println!("Extracting keywords...");
    
    let scorer = SynergyScorer::new(legend_card, champion_card);

    let mut seen_names = std::collections::HashSet::new();
    let candidates: Vec<Card> = cards
        .iter()
        .filter(|c| c.name != legend_card.name && c.name != champion_card.name)
        .filter(|c| seen_names.insert(c.name.clone()))
        .cloned()
        .collect();

    let scored = scorer.evaluate(&candidates);
    
    println!("--- Top {} Synergistic Cards ---", args.limit);
    
    for (i, result) in scored.into_iter().take(args.limit).enumerate() {
        println!("{}. {} (Score: {})", i + 1, result.card.name, result.score);
        println!("   - Matches Legend Keywords: {}", result.matching_legend_keywords);
        println!("   - Matches Champion Keywords: {}", result.matching_champion_keywords);
    }
}