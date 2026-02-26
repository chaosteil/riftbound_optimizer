mod models;
mod engine;

use clap::Parser;
use std::sync::OnceLock;
use std::collections::{HashSet, HashMap};
use models::Card;
use engine::SynergyScorer;
use strsim::levenshtein;

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

fn find_card<'a>(name: &str, cards: &'a [Card], expected_type: &str) -> Result<&'a Card, Vec<&'a String>> {
    let filtered_cards: Vec<&Card> = cards.iter()
        .filter(|c| c.is_type(expected_type))
        .collect();

    let exact_match = filtered_cards.iter().find(|c| c.name.to_lowercase() == name.to_lowercase());
    if let Some(card) = exact_match {
        return Ok(card);
    }

    // Fuzzy matching
    let mut distances: Vec<(&String, usize)> = filtered_cards.iter()
        .map(|c| (&c.name, levenshtein(&c.name.to_lowercase(), &name.to_lowercase())))
        .collect();
    
    distances.sort_by_key(|&(_, d)| d);
    let suggestions: Vec<&String> = distances.into_iter().take(3).map(|(n, _)| n).collect();
    Err(suggestions)
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
    
    if scored.is_empty() {
        println!("No synergistic cards found in the given domains.");
        return;
    }

    println!("\n--- Synergistic Cards ---");
    
    // Cross synergy analysis
    // Pre-extract interactions/triggers for scored cards for faster comparison
    let mut card_data = HashMap::new();
    for s in &scored {
        let ints: HashSet<String> = s.card.extract_interactions().into_iter().collect();
        let trigs: HashSet<String> = s.card.extract_triggers().into_iter().collect();
        let mechs: HashSet<String> = s.card.extract_deep_mechanics().into_iter().collect();
        card_data.insert(s.card.name.clone(), (ints, trigs, mechs));
    }

    let champ_mechs: HashSet<String> = champion_card.extract_deep_mechanics().into_iter().collect();
    let legend_mechs: HashSet<String> = legend_card.extract_deep_mechanics().into_iter().collect();

    let mut current_domain_group = String::new();

    for result in &scored {
        let domain_str = result.card.primary_domain_string();
        if domain_str != current_domain_group {
            println!("\n========================================");
            println!("  DOMAIN: {}", domain_str);
            println!("========================================");
            current_domain_group = domain_str;
        }

        let meta_tag = if result.meta_bonus { " [META SYNERGY]" } else { "" };
        println!("\n=== {} (Score: {}){} ===", result.card.name, result.score, meta_tag);
        if !result.matched_keywords.is_empty() {
            println!("  [Keywords]: {}", result.matched_keywords.join(", "));
        }
        if !result.matched_interactions.is_empty() {
            println!("  [Interactions]: {}", result.matched_interactions.join(", "));
        }
        if !result.matched_triggers.is_empty() {
            println!("  [Triggers]: {}", result.matched_triggers.join(", "));
        }

        // Find combos with other top cards
        let (my_ints, my_trigs, my_mechs) = card_data.get(&result.card.name).unwrap();
        let mut combos = Vec::new();
        let mut deep_combos = Vec::new();

        // Check deep combos directly with Champion or Legend
        if my_mechs.contains("BuffSource") {
            if champ_mechs.contains("MightyConsumer") {
                deep_combos.push(format!("[{}] buffs [{}] (Champion) to hit Mighty threshold!", result.card.name, champion_card.name));
            }
            if legend_mechs.contains("MightyConsumer") {
                deep_combos.push(format!("[{}] buffs [{}] (Legend) to hit Mighty threshold!", result.card.name, legend_card.name));
            }
        }
        if my_mechs.contains("MightyConsumer") {
            if champ_mechs.contains("BuffSource") {
                deep_combos.push(format!("[{}] (Champion) buffs [{}] to hit Mighty threshold!", champion_card.name, result.card.name));
            }
            if legend_mechs.contains("BuffSource") {
                deep_combos.push(format!("[{}] (Legend) buffs [{}] to hit Mighty threshold!", legend_card.name, result.card.name));
            }
        }

        for other in &scored {
            if result.card.name == other.card.name { continue; }
            let (other_ints, other_trigs, other_mechs) = card_data.get(&other.card.name).unwrap();
            
            // Cross-card Deep Combos
            if my_mechs.contains("BuffSource") && other_mechs.contains("MightyConsumer") {
                deep_combos.push(format!("[{}] buffs [{}] to hit Mighty threshold!", result.card.name, other.card.name));
            }
            
            let mut shared = Vec::new();
            for i in my_ints.intersection(other_ints) { shared.push(i.clone()); }
            for t in my_trigs.intersection(other_trigs) { shared.push(t.clone()); }
            
            // Only add if there is a shared interaction/trigger that isn't already 
            // shared with the legend/champion (i.e., not in matched_interactions/triggers)
            let mut new_shared = Vec::new();
            for s in shared {
                if !result.matched_interactions.contains(&s) && !result.matched_triggers.contains(&s) {
                    new_shared.push(s);
                }
            }

            if !new_shared.is_empty() {
                combos.push(format!("{} (via {})", other.card.name, new_shared.join(", ")));
            }
        }

        if !deep_combos.is_empty() {
            println!("  [Deep Combos]:");
            for combo in deep_combos.into_iter().take(3) {
                println!("    - {}", combo);
            }
        }

        if !combos.is_empty() {
            println!("  [Combos well with]:");
            // limit combos printed to avoid huge walls of text, top 3 is usually good
            for combo in combos.into_iter().take(5) {
                println!("    - {}", combo);
            }
        }
    }
}