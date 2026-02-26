use strsim::levenshtein;
use crate::models::Card;

pub fn find_card<'a>(name: &str, cards: &'a [Card], expected_type: &str) -> Result<&'a Card, Vec<&'a String>> {
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
