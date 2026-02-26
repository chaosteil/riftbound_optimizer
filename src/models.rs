use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub text: Option<String>,
    pub energy: Option<u32>,
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(rename = "cardType", default)]
    pub card_type: Vec<CardType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardType {
    pub id: String,
    pub label: String,
}

impl Card {
    pub fn is_type(&self, label_to_match: &str) -> bool {
        self.card_type.iter().any(|t| t.label.to_lowercase() == label_to_match.to_lowercase())
    }

    pub fn primary_domain_string(&self) -> String {
        if self.domains.is_empty() {
            return "Neutral".to_string();
        }
        let mut labels: Vec<String> = self.domains.iter().map(|d| d.label.clone()).collect();
        labels.sort();
        labels.join(", ")
    }

    pub fn extract_keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();
        
        // Add domains as keywords (e.g., "Fury", "Nature")
        for domain in &self.domains {
            keywords.push(domain.label.to_lowercase());
        }
        
        // Extract bracketed keywords from text (e.g., "[Accelerate]")
        if let Some(text) = &self.text {
            let mut current_keyword = String::new();
            let mut in_bracket = false;
            
            for c in text.chars() {
                if c == '[' {
                    in_bracket = true;
                    current_keyword.clear();
                } else if c == ']' {
                    if in_bracket && !current_keyword.is_empty() {
                        keywords.push(current_keyword.to_lowercase());
                    }
                    in_bracket = false;
                } else if in_bracket {
                    current_keyword.push(c);
                }
            }
        }
        
        // Remove duplicates
        keywords.sort();
        keywords.dedup();
        
        keywords
    }

    pub fn extract_interactions(&self) -> Vec<String> {
        let mut interactions = Vec::new();
        if let Some(text) = &self.text {
            let t = text.to_lowercase();
            let verbs = [
                "draw", "discard", "damage", "heal", "exhaust", "ready", "buff", "destroy",
                "strike", "summon", "token", "prevent", "pay", "return", "reveal", "choose"
            ];
            for v in verbs {
                if t.contains(v) {
                    interactions.push(v.to_string());
                }
            }
        }
        interactions.sort();
        interactions.dedup();
        interactions
    }

    pub fn extract_triggers(&self) -> Vec<String> {
        let mut triggers = Vec::new();
        if let Some(text) = &self.text {
            let t = text.to_lowercase();
            let patterns = [
                "when you play", "when this enters", "when you discard", "reaction", "action", "if you", "after", "before", "when you draw"
            ];
            for p in patterns {
                if t.contains(p) {
                    triggers.push(p.to_string());
                }
            }
        }
        triggers.sort();
        triggers.dedup();
        triggers
    }

    pub fn extract_deep_mechanics(&self) -> Vec<String> {
        let mut mechanics = Vec::new();
        if let Some(text) = &self.text {
            let t = text.to_lowercase();
            
            // Check if it's a Buff Source
            if t.contains("buff") || t.contains("+1 :rb_might:") || t.contains("+2 :rb_might:") || t.contains("+3 :rb_might:") || t.contains("+4 :rb_might:") || t.contains("[assault") {
                mechanics.push("BuffSource".to_string());
            }

            // Check if it's a Mighty / Buff Target
            if t.contains("[mighty]") || t.contains("5+ :rb_might:") || t.contains("might:") {
                mechanics.push("MightyConsumer".to_string());
            }

            // Meta Mechanics
            if t.contains("token") || t.contains("summon") {
                mechanics.push("TokenSpawner".to_string());
            }
            if self.is_type("Spell") && (t.contains("deal ") || t.contains("damage")) {
                mechanics.push("SpellDamage".to_string());
            }
            if t.contains("ready") || t.contains("assault") {
                mechanics.push("AggroTool".to_string());
            }
        }
        
        // High cost unit
        if self.is_type("Unit") {
            if let Some(cost) = self.energy {
                if cost >= 7 {
                    mechanics.push("HighCostUnit".to_string());
                }
            }
        }

        mechanics
    }
}
