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
                println!("  {}x {} ({}) [{}E / {}P]", count, card.name, type_label, card.energy.unwrap_or(0), card.power.unwrap_or(0));
                println!("      {}", card.clean_text().replace('\n', "\n      "));
            }
        }
    }
}

pub struct DeckBuilder;

impl DeckBuilder {
    pub fn build<'a>(legend: &'a Card, champion: &'a Card, scored: &[ScoredCard<'a>]) -> Decklist<'a> {
        let mut deck_cards: Vec<(&'a Card, usize)> = Vec::new();
        let mut total_added = 0;
        let mut total_power = champion.power.unwrap_or(0) as usize;
        let target_size = 39; // 40 - 1 (the chosen champion)

        // Ideal curve buckets (target ranges)
        let mut buckets: HashMap<u32, Vec<&'a Card>> = HashMap::new();
        for s in scored {
            if s.card.name == legend.name || s.card.name == champion.name {
                continue;
            }

            if !s.card.is_type("Unit") && !s.card.is_type("Spell") && !s.card.is_type("Gear") {
                continue;
            }

            let cost = s.card.energy.unwrap_or(0);
            let bucket_key = if cost >= 5 { 5 } else { cost };
            buckets.entry(bucket_key).or_default().push(s.card);
        }

        let mut fill_bucket = |bucket_key: u32, target: usize, deck_cards: &mut Vec<(&'a Card, usize)>, total_added: &mut usize, total_power: &mut usize| {
            if let Some(cards) = buckets.get(&bucket_key) {
                let mut added_in_bucket = 0;
                for &card in cards {
                    if *total_added >= target_size || added_in_bucket >= target { break; }
                    
                    let space_left_in_deck = target_size - *total_added;
                    let space_left_in_bucket = target - added_in_bucket;
                    
                    // Filter logic: if it's a "Seal" (ramp card) and we don't have high power costs yet, skip it.
                    // Assuming a deck needs ramp if its average power cost is shaping up to be high (> 10 total so far)
                    let is_ramp = card.name.starts_with("Seal of") || card.clean_text().to_lowercase().contains("gold token");
                    if is_ramp && *total_power < 10 && *total_added > 15 {
                        continue; 
                    }

                    // Max 3 copies per card
                    let to_add = std::cmp::min(3, std::cmp::min(space_left_in_deck, space_left_in_bucket));
                    if to_add > 0 {
                        deck_cards.push((card, to_add));
                        *total_added += to_add;
                        added_in_bucket += to_add;
                        *total_power += card.power.unwrap_or(0) as usize * to_add;
                    }
                }
            }
        };

        // 1. Fill 2-drops first (Priority)
        fill_bucket(2, 8, &mut deck_cards, &mut total_added, &mut total_power);
        
        // 2. Fill the rest of the curve
        fill_bucket(0, 3, &mut deck_cards, &mut total_added, &mut total_power);
        fill_bucket(1, 4, &mut deck_cards, &mut total_added, &mut total_power); 
        fill_bucket(3, 9, &mut deck_cards, &mut total_added, &mut total_power);
        fill_bucket(4, 7, &mut deck_cards, &mut total_added, &mut total_power);
        fill_bucket(5, 8, &mut deck_cards, &mut total_added, &mut total_power);

        // 3. If we haven't reached 39 cards yet, backfill
        if total_added < target_size {
            let mut remaining_space = target_size - total_added;
            for s in scored {
                if remaining_space == 0 { break; }
                if !s.card.is_type("Unit") && !s.card.is_type("Spell") && !s.card.is_type("Gear") { continue; }
                if s.card.name == legend.name || s.card.name == champion.name { continue; }
                
                // Check if already in deck
                if deck_cards.iter().any(|(c, _)| c.name == s.card.name) {
                    continue;
                }

                let is_ramp = s.card.name.starts_with("Seal of") || s.card.clean_text().to_lowercase().contains("gold token");
                if is_ramp && total_power < 10 {
                    continue; 
                }

                let to_add = std::cmp::min(3, remaining_space);
                deck_cards.push((s.card, to_add));
                total_added += to_add;
                total_power += s.card.power.unwrap_or(0) as usize * to_add;
                remaining_space -= to_add;
            }
        }

        Decklist {
            legend,
            champion,
            cards: deck_cards,
        }
    }
}
