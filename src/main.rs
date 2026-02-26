mod models;
mod engine;
mod search;
mod analyzer;

use clap::Parser;
use std::sync::OnceLock;
use std::collections::HashSet;
use models::Card;
use engine::SynergyScorer;
use search::find_card;
use analyzer::analyze_and_print;

const CARDS_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/cards.json"));

static CARDS: OnceLock<Vec<Card>> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about = "Riftbound Synergy Optimizer", long_about = None)]
struct Args {
    #[arg(short, long, help = "Name of the Legend card")]
    legend: String,

    #[arg(short, long, help = "Name of the Champion card")]
    champion: String,
}

fn load_cards() -> &'static Vec<Card> {
    CARDS.get_or_init(|| {
        serde_json::from_str(CARDS_JSON).expect("Failed to parse embedded JSON")
    })
}

fn main() {
    let args = Args::parse();
    let cards = load_cards();

    let legend_card = match find_card(&args.legend, cards, "Legend") {
        Ok(c) => c,
        Err(suggestions) => {
            eprintln!("Error: Legend '{}' not found.", args.legend);
            eprintln!("Did you mean: {}?", suggestions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
            std::process::exit(1);
        }
    };

    let champion_card = match find_card(&args.champion, cards, "Unit") {
        Ok(c) => c,
        Err(suggestions) => {
            eprintln!("Error: Champion '{}' not found.", args.champion);
            eprintln!("Did you mean: {}?", suggestions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
            std::process::exit(1);
        }
    };

    println!("Found Legend: {}", legend_card.name);
    println!("Found Champion: {}", champion_card.name);

    // Extract Legend Domains
    let legend_domains: HashSet<String> = legend_card.domains.iter()
        .map(|d| d.label.to_lowercase())
        .collect();
    
    println!("Legend Domains: {:?}", legend_domains);

    let scorer = SynergyScorer::new(legend_card, champion_card);
    println!("Detected Meta Archetype: {:?}", scorer.archetype);

    let mut seen_names = HashSet::new();
    let candidates: Vec<Card> = cards
        .iter()
        .filter(|c| c.name != legend_card.name && c.name != champion_card.name)
        // Domain filter: all of the candidate's domains must be in the legend's domains
        .filter(|c| {
            let card_domains: HashSet<String> = c.domains.iter().map(|d| d.label.to_lowercase()).collect();
            card_domains.is_subset(&legend_domains)
        })
        .filter(|c| seen_names.insert(c.name.clone()))
        .cloned()
        .collect();

    let scored = scorer.evaluate(&candidates);
    
    analyze_and_print(&scored, legend_card, champion_card);
}
