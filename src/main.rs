mod models;
mod engine;
mod search;
mod analyzer;

use clap::{Parser, Subcommand};
use std::collections::HashSet;
use models::Card;
use engine::SynergyScorer;
use search::find_card;
use analyzer::analyze_and_print;

#[derive(Parser, Debug)]
#[command(author, version, about = "Riftbound Synergy Optimizer", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Optimize and find synergies for a given Legend and Champion
    Optimize {
        #[arg(short, long, help = "Name of the Legend card")]
        legend: String,

        #[arg(short, long, help = "Name of the Champion card")]
        champion: String,
    },
    /// Refresh the local cards.json database from the latest online data
    Update,
}

fn load_cards() -> Vec<Card> {
    let json_data = std::fs::read_to_string("cards.json").unwrap_or_else(|_| {
        eprintln!("Error: cards.json not found. Please run `riftbound_optimizer update` to download the latest card database.");
        std::process::exit(1);
    });
    
    serde_json::from_str(&json_data).unwrap_or_else(|e| {
        eprintln!("Error parsing cards.json: {}", e);
        std::process::exit(1);
    })
}

fn handle_update() {
    println!("Fetching latest card database...");
    let url = "https://gist.githubusercontent.com/OwenMelbz/e04dadf641cc9b81cb882b4612343112/raw/riftbound.json";
    
    match ureq::get(url).call() {
        Ok(mut response) => {
            if let Ok(text) = response.body_mut().read_to_string() {
                if std::fs::write("cards.json", text).is_ok() {
                    println!("Successfully updated cards.json!");
                } else {
                    eprintln!("Failed to write to cards.json. Check file permissions.");
                }
            } else {
                eprintln!("Failed to read response data.");
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch the card data: {}", e);
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Update => {
            handle_update();
        }
        Commands::Optimize { legend, champion } => {
            let cards = load_cards();

            let legend_card = match find_card(&legend, &cards, "Legend") {
                Ok(c) => c,
                Err(suggestions) => {
                    eprintln!("Error: Legend '{}' not found.", legend);
                    eprintln!("Did you mean: {}?", suggestions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                    std::process::exit(1);
                }
            };

            let champion_card = match find_card(&champion, &cards, "Unit") {
                Ok(c) => c,
                Err(suggestions) => {
                    eprintln!("Error: Champion '{}' not found.", champion);
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
    }
}
