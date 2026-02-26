use std::collections::HashSet;
use crate::models::Card;

pub struct SynergyScorer {
    legend_keywords: HashSet<String>,
    champion_keywords: HashSet<String>,
}

#[derive(Debug)]
pub struct ScoredCard<'a> {
    pub card: &'a Card,
    pub score: u32,
    pub matching_legend_keywords: usize,
    pub matching_champion_keywords: usize,
}

impl SynergyScorer {
    pub fn new(legend: &Card, champion: &Card) -> Self {
        let legend_keywords: HashSet<String> = legend.extract_keywords().into_iter().collect();
        let champion_keywords: HashSet<String> = champion.extract_keywords().into_iter().collect();
        
        Self {
            legend_keywords,
            champion_keywords,
        }
    }

    pub fn score_card<'a>(&self, candidate: &'a Card) -> ScoredCard<'a> {
        let candidate_keywords: HashSet<String> = candidate.extract_keywords().into_iter().collect();
        
        let match_legend = candidate_keywords.intersection(&self.legend_keywords).count();
        let match_champ = candidate_keywords.intersection(&self.champion_keywords).count();
        
        let mut score = (match_legend as u32 * 3) + (match_champ as u32 * 2);
        
        // Tribal synergy bonus: Matches BOTH legend and champion
        if match_legend > 0 && match_champ > 0 {
            score += 2;
        }
        
        ScoredCard {
            card: candidate,
            score,
            matching_legend_keywords: match_legend,
            matching_champion_keywords: match_champ,
        }
    }

    pub fn evaluate<'a>(&self, cards: &'a [Card]) -> Vec<ScoredCard<'a>> {
        let mut scored: Vec<ScoredCard<'a>> = cards
            .iter()
            .map(|c| self.score_card(c))
            .filter(|s| s.score > 0)
            .collect();
            
        // Sort descending by score, tie-break by name
        scored.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.card.name.cmp(&b.card.name)));
        
        scored
    }
}
