use std::collections::HashSet;
use crate::models::Card;

pub struct SynergyScorer {
    legend_keywords: HashSet<String>,
    champion_keywords: HashSet<String>,
    legend_interactions: HashSet<String>,
    champion_interactions: HashSet<String>,
    legend_triggers: HashSet<String>,
    champion_triggers: HashSet<String>,
}

#[derive(Debug)]
pub struct ScoredCard<'a> {
    pub card: &'a Card,
    pub score: u32,
    pub matched_keywords: Vec<String>,
    pub matched_interactions: Vec<String>,
    pub matched_triggers: Vec<String>,
}

impl SynergyScorer {
    pub fn new(legend: &Card, champion: &Card) -> Self {
        let legend_keywords: HashSet<String> = legend.extract_keywords().into_iter().collect();
        let champion_keywords: HashSet<String> = champion.extract_keywords().into_iter().collect();
        
        let legend_interactions: HashSet<String> = legend.extract_interactions().into_iter().collect();
        let champion_interactions: HashSet<String> = champion.extract_interactions().into_iter().collect();
        
        let legend_triggers: HashSet<String> = legend.extract_triggers().into_iter().collect();
        let champion_triggers: HashSet<String> = champion.extract_triggers().into_iter().collect();

        Self {
            legend_keywords,
            champion_keywords,
            legend_interactions,
            champion_interactions,
            legend_triggers,
            champion_triggers,
        }
    }

    pub fn score_card<'a>(&self, candidate: &'a Card) -> ScoredCard<'a> {
        let candidate_keywords: HashSet<String> = candidate.extract_keywords().into_iter().collect();
        let candidate_interactions: HashSet<String> = candidate.extract_interactions().into_iter().collect();
        let candidate_triggers: HashSet<String> = candidate.extract_triggers().into_iter().collect();
        
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
        let mut score = (match_legend_k as u32 * 3) + (match_champ_k as u32 * 2);
        
        // Bonus for interactions and triggers
        score += (matched_interactions.len() as u32) * 2;
        score += (matched_triggers.len() as u32) * 2;

        // Tribal synergy bonus
        if match_legend_k > 0 && match_champ_k > 0 {
            score += 2;
        }

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
        }
    }

    pub fn evaluate<'a>(&self, cards: &'a [Card]) -> Vec<ScoredCard<'a>> {
        let mut scored: Vec<ScoredCard<'a>> = cards
            .iter()
            .map(|c| self.score_card(c))
            .filter(|s| s.score > 0)
            .collect();
            
        scored.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.card.name.cmp(&b.card.name)));
        
        scored
    }
}
