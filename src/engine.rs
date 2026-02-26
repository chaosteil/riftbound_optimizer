use std::collections::HashSet;
use crate::models::Card;

#[derive(Debug, PartialEq, Clone)]
pub enum Archetype {
    AggroTempo,
    SpellslingerControl,
    ComboCheat,
    TokenFlood,
    Midrange,
}

impl Archetype {
    pub fn determine(champion_name: &str) -> Self {
        let name = champion_name.to_lowercase();
        if name.contains("draven") || name.contains("irelia") || name.contains("annie") || name.contains("yasuo") {
            Archetype::AggroTempo
        } else if name.contains("kai'sa") || name.contains("ezreal") || name.contains("karma") {
            Archetype::SpellslingerControl
        } else if name.contains("miss fortune") {
            Archetype::ComboCheat
        } else if name.contains("viktor") || name.contains("azir") {
            Archetype::TokenFlood
        } else {
            Archetype::Midrange
        }
    }
}

pub struct SynergyScorer {
    legend_keywords: HashSet<String>,
    champion_keywords: HashSet<String>,
    legend_interactions: HashSet<String>,
    champion_interactions: HashSet<String>,
    legend_triggers: HashSet<String>,
    champion_triggers: HashSet<String>,
    pub archetype: Archetype,
}

#[derive(Debug)]
pub struct ScoredCard<'a> {
    pub card: &'a Card,
    pub score: i32, // Changed to i32 to allow negative scoring for penalties
    pub matched_keywords: Vec<String>,
    pub matched_interactions: Vec<String>,
    pub matched_triggers: Vec<String>,
    pub meta_bonus: bool,
    pub cabs: bool,
    pub sbread: Vec<String>,
}

impl SynergyScorer {
    pub fn new(legend: &Card, champion: &Card) -> Self {
        let legend_keywords: HashSet<String> = legend.extract_keywords().into_iter().collect();
        let champion_keywords: HashSet<String> = champion.extract_keywords().into_iter().collect();
        
        let legend_interactions: HashSet<String> = legend.extract_interactions().into_iter().collect();
        let champion_interactions: HashSet<String> = champion.extract_interactions().into_iter().collect();
        
        let legend_triggers: HashSet<String> = legend.extract_triggers().into_iter().collect();
        let champion_triggers: HashSet<String> = champion.extract_triggers().into_iter().collect();

        let archetype = Archetype::determine(&champion.name);

        Self {
            legend_keywords,
            champion_keywords,
            legend_interactions,
            champion_interactions,
            legend_triggers,
            champion_triggers,
            archetype,
        }
    }

    pub fn score_card<'a>(&self, candidate: &'a Card) -> ScoredCard<'a> {
        let candidate_keywords: HashSet<String> = candidate.extract_keywords().into_iter().collect();
        let candidate_interactions: HashSet<String> = candidate.extract_interactions().into_iter().collect();
        let candidate_triggers: HashSet<String> = candidate.extract_triggers().into_iter().collect();
        let candidate_mechs: HashSet<String> = candidate.extract_deep_mechanics().into_iter().collect();
        
        let mut matched_keywords: HashSet<String> = HashSet::new();
        for k in candidate_keywords.intersection(&self.legend_keywords) { matched_keywords.insert(k.clone()); }
        for k in candidate_keywords.intersection(&self.champion_keywords) { matched_keywords.insert(k.clone()); }

        let mut matched_interactions: HashSet<String> = HashSet::new();
        for i in candidate_interactions.intersection(&self.legend_interactions) { matched_interactions.insert(i.clone()); }
        for i in candidate_interactions.intersection(&self.champion_interactions) { matched_interactions.insert(i.clone()); }

        let mut matched_triggers: HashSet<String> = HashSet::new();
        for t in candidate_triggers.intersection(&self.legend_triggers) { matched_triggers.insert(t.clone()); }
        for t in candidate_triggers.intersection(&self.champion_triggers) { matched_triggers.insert(t.clone()); }

        let match_legend_k = candidate_keywords.intersection(&self.legend_keywords).count();
        let match_champ_k = candidate_keywords.intersection(&self.champion_keywords).count();

        // Base score for keywords
        let mut score: i32 = (match_legend_k as i32 * 3) + (match_champ_k as i32 * 2);
        
        // Bonus for interactions and triggers
        score += (matched_interactions.len() as i32) * 2;
        score += (matched_triggers.len() as i32) * 2;

        // Tribal synergy bonus
        if match_legend_k > 0 && match_champ_k > 0 {
            score += 2;
        }

        let mut meta_bonus = false;
        match self.archetype {
            Archetype::AggroTempo => {
                if candidate_mechs.contains("AggroTool") {
                    score += 5;
                    meta_bonus = true;
                }
            }
            Archetype::SpellslingerControl => {
                if candidate_mechs.contains("SpellDamage") {
                    score += 5;
                    meta_bonus = true;
                }
            }
            Archetype::ComboCheat => {
                if candidate_mechs.contains("HighCostUnit") {
                    score += 5;
                    meta_bonus = true;
                }
            }
            Archetype::TokenFlood => {
                if candidate_mechs.contains("TokenSpawner") {
                    score += 5;
                    meta_bonus = true;
                }
            }
            Archetype::Midrange => {}
        }

        let cabs = candidate.has_cabs();
        if !cabs {
            score -= 10; // Heavily penalize non-CABS cards
        }

        let sbread = candidate.extract_sbread();

        let mut mk_vec: Vec<String> = matched_keywords.into_iter().collect();
        mk_vec.sort();
        let mut mi_vec: Vec<String> = matched_interactions.into_iter().collect();
        mi_vec.sort();
        let mut mt_vec: Vec<String> = matched_triggers.into_iter().collect();
        mt_vec.sort();

        ScoredCard {
            card: candidate,
            score,
            matched_keywords: mk_vec,
            matched_interactions: mi_vec,
            matched_triggers: mt_vec,
            meta_bonus,
            cabs,
            sbread,
        }
    }

    pub fn evaluate<'a>(&self, cards: &'a [Card]) -> Vec<ScoredCard<'a>> {
        let mut scored: Vec<ScoredCard<'a>> = cards
            .iter()
            .map(|c| self.score_card(c))
            .filter(|s| s.score > 0 || s.meta_bonus) // Keep if it has meta bonus even if score was 0
            .collect();
            
        // Sort by Domain, then Meta Bonus, then Score descending
        scored.sort_by(|a, b| {
            a.card.primary_domain_string().cmp(&b.card.primary_domain_string())
                .then_with(|| b.meta_bonus.cmp(&a.meta_bonus))
                .then_with(|| b.score.cmp(&a.score))
                .then_with(|| a.card.name.cmp(&b.card.name))
        });
        
        scored
    }
}
