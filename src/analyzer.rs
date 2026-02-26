use std::collections::{HashSet, HashMap};
use crate::models::Card;
use crate::engine::ScoredCard;

pub fn analyze_and_print(scored: &[ScoredCard], legend_card: &Card, champion_card: &Card) {
    if scored.is_empty() {
        println!("No synergistic cards found in the given domains.");
        return;
    }

    println!("\n--- Synergistic Cards ---");
    println!("Legend: {}", legend_card.name);
    println!("  Text: {}", legend_card.clean_text().replace('\n', "\n        "));
    println!("Champion: {}", champion_card.name);
    println!("  Cost: {}E / {}P", champion_card.energy.unwrap_or(0), champion_card.power.unwrap_or(0));
    println!("  Text: {}", champion_card.clean_text().replace('\n', "\n        "));
    
    // Cross synergy analysis
    // Pre-extract interactions/triggers for scored cards for faster comparison
    let mut card_data = HashMap::new();
    for s in scored {
        let ints: HashSet<String> = s.card.extract_interactions().into_iter().collect();
        let trigs: HashSet<String> = s.card.extract_triggers().into_iter().collect();
        let mechs: HashSet<String> = s.card.extract_deep_mechanics().into_iter().collect();
        card_data.insert(s.card.name.clone(), (ints, trigs, mechs));
    }

    let champ_mechs: HashSet<String> = champion_card.extract_deep_mechanics().into_iter().collect();
    let legend_mechs: HashSet<String> = legend_card.extract_deep_mechanics().into_iter().collect();

    let mut current_domain_group = String::new();

    for result in scored {
        let domain_str = result.card.primary_domain_string();
        if domain_str != current_domain_group {
            println!("\n========================================");
            println!("  DOMAIN: {}", domain_str);
            println!("========================================");
            current_domain_group = domain_str;
        }

        let meta_tag = if result.meta_bonus { " [META SYNERGY]" } else { "" };
        println!("\n=== {} (Score: {}){} ===", result.card.name, result.score, meta_tag);
        println!("  [Cost]: {}E / {}P", result.card.energy.unwrap_or(0), result.card.power.unwrap_or(0));
        println!("  [Text]: {}", result.card.clean_text().replace('\n', "\n          "));

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

        for other in scored {
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
