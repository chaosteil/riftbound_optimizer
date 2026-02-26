mod models;
mod engine;
mod search;
mod analyzer;
mod builder;

use clap::{Parser, Subcommand};
use std::collections::{HashSet, HashMap};
use models::Card;
use engine::SynergyScorer;
use search::find_card;
use analyzer::analyze_and_print;
use builder::DeckBuilder;

use std::io::BufRead;

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

        #[arg(long, help = "Path to a text file containing your card collection (e.g. '3x Card Name')")]
        collection: Option<String>,
    },
    /// Generate a 40-card sample deck for a given Legend and Champion
    Deck {
        #[arg(short, long, help = "Name of the Legend card")]
        legend: String,

        #[arg(short, long, help = "Name of the Champion card")]
        champion: String,

        #[arg(long, help = "Path to a text file containing your card collection (e.g. '3x Card Name')")]
        collection: Option<String>,
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

fn load_collection(path: &str) -> HashMap<String, usize> {
    let mut collection = HashMap::new();
    let file = std::fs::File::open(path).unwrap_or_else(|e| {
        eprintln!("Error opening collection file '{}': {}", path, e);
        std::process::exit(1);
    });

    let reader = std::io::BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // parse "3x Card Name" or "Card Name"
        let mut parts = trimmed.splitn(2, 'x');
        let first = parts.next().unwrap().trim();
        let second = parts.next();

        if let Some(rest) = second {
            if let Ok(count) = first.parse::<usize>() {
                collection.insert(rest.trim().to_lowercase(), count);
            } else {
                collection.insert(trimmed.to_lowercase(), 3); // Default max if "x" is part of a name but not a quantity
            }
        } else {
            collection.insert(first.to_lowercase(), 3); // Default max copies if no quantity specified
        }
    }
    collection
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
        Commands::Optimize { legend, champion, collection } => {
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

            let owned_cards = collection.map(|path| load_collection(&path));

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
                .filter(|c| {
                    // If collection is provided, ensure card is in it
                    if let Some(ref owned) = owned_cards {
                        owned.contains_key(&c.name.to_lowercase())
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            let scored = scorer.evaluate(&candidates);
            
            analyze_and_print(&scored, legend_card, champion_card);
        }
        Commands::Deck { legend, champion, collection } => {
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

            let owned_cards = collection.map(|path| load_collection(&path));

            let legend_domains: HashSet<String> = legend_card.domains.iter()
                .map(|d| d.label.to_lowercase())
                .collect();
            
            let scorer = SynergyScorer::new(legend_card, champion_card);

            let mut seen_names = HashSet::new();
            let candidates: Vec<Card> = cards
                .iter()
                .filter(|c| c.name != legend_card.name && c.name != champion_card.name)
                .filter(|c| {
                    let card_domains: HashSet<String> = c.domains.iter().map(|d| d.label.to_lowercase()).collect();
                    card_domains.is_subset(&legend_domains)
                })
                .filter(|c| seen_names.insert(c.name.clone()))
                .filter(|c| {
                    // If collection is provided, ensure card is in it
                    if let Some(ref owned) = owned_cards {
                        owned.contains_key(&c.name.to_lowercase())
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            let scored = scorer.evaluate(&candidates);
            
            let deck = DeckBuilder::build(legend_card, champion_card, &scored, owned_cards.as_ref());
            deck.print();
        }
    }
}
