use std::collections::HashMap;
use crate::models::Card;
use crate::engine::ScoredCard;

pub struct Decklist<'a> {
    pub legend: &'a Card,
    pub champion: &'a Card,
    pub cards: Vec<(&'a Card, usize)>, // Card and quantity (1-3)
}

impl<'a> Decklist<'a> {
    pub fn print(&self) {
        println!("\n=== GENERATED DECKLIST ===");
        println!("Legend: {}", self.legend.name);
        println!("  Text: {}", self.legend.clean_text().replace('\n', "\n        "));
        println!("Starting Champion: {}", self.champion.name);
        println!("  Cost: {}E / {}P", self.champion.energy.unwrap_or(0), self.champion.power.unwrap_or(0));
        println!("  Text: {}", self.champion.clean_text().replace('\n', "\n        "));
        println!("--------------------------");
        
        let total_cards: usize = self.cards.iter().map(|(_, count)| count).sum();
        let total_power: usize = self.cards.iter().map(|(c, count)| c.power.unwrap_or(0) as usize * count).sum();
        // Plus 1 for the champion which is technically in the deck but starts in champion zone
        println!("Total Main Deck Cards: {} / 40", total_cards + 1); 
        println!("Total Deck Power Requirement: {}", total_power + self.champion.power.unwrap_or(0) as usize);
        println!("--------------------------");

        // Group by cost
        let mut by_cost: HashMap<u32, Vec<(&Card, usize)>> = HashMap::new();
        for (card, count) in &self.cards {
            let cost = card.energy.unwrap_or(0);
            by_cost.entry(cost).or_default().push((card, *count));
        }

        let mut costs: Vec<u32> = by_cost.keys().cloned().collect();
        costs.sort();

        for cost in costs {
            let group = by_cost.get(&cost).unwrap();
            let sum_in_cost: usize = group.iter().map(|(_, c)| c).sum();
            println!("\n[Cost {} Energy] ({} cards)", cost, sum_in_cost);
            
            let mut sorted_group = group.clone();
            sorted_group.sort_by(|a, b| a.0.name.cmp(&b.0.name));

            for (card, count) in sorted_group {
                let type_label = if card.is_type("Unit") { "Unit" } else if card.is_type("Spell") { "Spell" } else { "Gear" };
                
                let mut display_tags = Vec::new();
                if card.has_cabs() {
                    display_tags.push("CABS".to_string());
                }
                let mut sbread = card.extract_sbread();
                display_tags.append(&mut sbread);
                
                let tags_str = if display_tags.is_empty() { String::new() } else { format!(" [{}]", display_tags.join(", ")) };
                
                println!("  {}x {} ({}) [{}E / {}P]{}", count, card.name, type_label, card.energy.unwrap_or(0), card.power.unwrap_or(0), tags_str);
                println!("      {}", card.clean_text().replace('\n', "\n      "));
            }
        }
    }
}

pub struct DeckBuilder;

impl DeckBuilder {
    pub fn build<'a>(legend: &'a Card, champion: &'a Card, scored: &[ScoredCard<'a>], collection_limits: Option<&HashMap<String, usize>>) -> Decklist<'a> {
        let mut deck_cards: Vec<(&'a Card, usize)> = Vec::new();
        let mut total_added = 0;
        let mut total_power = champion.power.unwrap_or(0) as usize;
        let target_size = 39; // 40 - 1 (the chosen champion)

        // Quotas for SBREAD
        let max_bombs = 4;
        let mut bombs_added = 0;
        
        let max_removal = 6;
        let mut removal_added = 0;
        
        let max_evasion = 5;
        let mut evasion_added = 0;
        
        let max_aggro = 8;
        let mut aggro_added = 0;
        
        let max_dump = 4;
        let mut dump_added = 0;

        // Filter valid candidates first
        let mut valid_candidates: Vec<&ScoredCard<'a>> = scored.iter()
            .filter(|s| s.card.name != legend.name && s.card.name != champion.name)
            .filter(|s| s.card.is_type("Unit") || s.card.is_type("Spell") || s.card.is_type("Gear"))
            .collect();
            
        // Sort by score descending
        valid_candidates.sort_by(|a, b| b.score.cmp(&a.score));

        // Helper to determine optimal copies
        let determine_copies = |s: &ScoredCard, max_allowed: usize| -> usize {
            let base_copies = if s.sbread.contains(&"Bomb".to_string()) {
                1
            } else if s.sbread.contains(&"Removal".to_string()) || s.sbread.contains(&"Evasion".to_string()) || s.sbread.contains(&"Dump".to_string()) {
                2
            } else {
                // High synergy, aggro, core combo pieces
                3
            };
            
            let requested_copies = std::cmp::min(base_copies, max_allowed);
            
            if let Some(owned) = collection_limits {
                let owned_count = owned.get(&s.card.name.to_lowercase()).copied().unwrap_or(0);
                std::cmp::min(requested_copies, owned_count)
            } else {
                requested_copies
            }
        };

        // Pass 1 (S - Synergy Core): Add top 8 highest scoring cards that have CABS
        let mut core_added_count = 0;
        for s in &valid_candidates {
            if total_added >= target_size || core_added_count >= 8 { break; }
            if !s.cabs { continue; }
            
            let allowed = target_size - total_added;
            let to_add = determine_copies(s, allowed);
            
            if to_add > 0 {
                deck_cards.push((s.card, to_add));
                total_added += to_add;
                total_power += s.card.power.unwrap_or(0) as usize * to_add;
                core_added_count += 1;
            }
        }

        // Pass 2 (B - Bombs)
        for s in &valid_candidates {
            if total_added >= target_size || bombs_added >= max_bombs { break; }
            if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
            if !s.cabs { continue; }
            
            if s.sbread.contains(&"Bomb".to_string()) {
                let allowed = std::cmp::min(max_bombs - bombs_added, target_size - total_added);
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                    bombs_added += to_add;
                }
            }
        }

        // Pass 3 (R - Removal)
        for s in &valid_candidates {
            if total_added >= target_size || removal_added >= max_removal { break; }
            if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
            if !s.cabs { continue; }
            
            if s.sbread.contains(&"Removal".to_string()) {
                let allowed = std::cmp::min(max_removal - removal_added, target_size - total_added);
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                    removal_added += to_add;
                }
            }
        }

        // Pass 4 (E - Evasion)
        for s in &valid_candidates {
            if total_added >= target_size || evasion_added >= max_evasion { break; }
            if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
            if !s.cabs { continue; }
            
            if s.sbread.contains(&"Evasion".to_string()) {
                let allowed = std::cmp::min(max_evasion - evasion_added, target_size - total_added);
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                    evasion_added += to_add;
                }
            }
        }

        // Pass 5 (A - Aggro)
        for s in &valid_candidates {
            if total_added >= target_size || aggro_added >= max_aggro { break; }
            if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
            if !s.cabs { continue; }
            
            if s.sbread.contains(&"Aggro".to_string()) {
                let allowed = std::cmp::min(max_aggro - aggro_added, target_size - total_added);
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                    aggro_added += to_add;
                }
            }
        }

        // Pass 6 (D - Dump & Ramp Filler)
        for s in &valid_candidates {
            if total_added >= target_size { break; }
            if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
            
            let is_ramp = s.card.name.starts_with("Seal of") || s.card.clean_text().to_lowercase().contains("gold token");
            if is_ramp && total_power < 10 {
                continue; 
            }

            if s.sbread.contains(&"Dump".to_string()) || is_ramp {
                let allowed = std::cmp::min(max_dump - dump_added, target_size - total_added);
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                    dump_added += to_add;
                }
            }
        }

        // Pass 7 (Curve / Filler)
        if total_added < target_size {
            for s in &valid_candidates {
                if total_added >= target_size { break; }
                if deck_cards.iter().any(|(c, _)| c.name == s.card.name) { continue; }
                
                let allowed = target_size - total_added;
                let to_add = determine_copies(s, allowed);
                if to_add > 0 {
                    deck_cards.push((s.card, to_add));
                    total_added += to_add;
                    total_power += s.card.power.unwrap_or(0) as usize * to_add;
                }
            }
        }

        Decklist {
            legend,
            champion,
            cards: deck_cards,
        }
    }
}
