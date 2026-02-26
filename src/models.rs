use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub text: Option<String>,
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
}
