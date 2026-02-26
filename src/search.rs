use strsim::levenshtein;
use crate::models::Card;

pub fn find_card<'a>(name: &str, cards: &'a [Card], expected_type: &str) -> Result<&'a Card, Vec<&'a String>> {
    let filtered_cards: Vec<&Card> = cards.iter()
        .filter(|c| c.is_type(expected_type))
        .collect();

    // 1. Exact match
    let exact_match = filtered_cards.iter().find(|c| c.name.to_lowercase() == name.to_lowercase());
    if let Some(card) = exact_match {
        return Ok(card);
    }

    // 2. Substring match in name
    let contains_match = filtered_cards.iter().find(|c| c.name.to_lowercase().contains(&name.to_lowercase()));
    if let Some(card) = contains_match {
        return Ok(card);
    }

    // 3. For Legends, if the user typed the Champion's name (e.g. "teemo"), check the rules text
    if expected_type == "Legend" {
        let text_match = filtered_cards.iter().find(|c| {
            if let Some(ref text) = c.text {
                text.to_lowercase().contains(&name.to_lowercase())
            } else {
                false
            }
        });
        if let Some(card) = text_match {
            return Ok(card);
        }
    }

    // 4. Fuzzy matching fallback
    let mut distances: Vec<(&String, usize)> = filtered_cards.iter()
        .map(|c| (&c.name, levenshtein(&c.name.to_lowercase(), &name.to_lowercase())))
        .collect();
    
    distances.sort_by_key(|&(_, d)| d);
    let suggestions: Vec<&String> = distances.into_iter().take(3).map(|(n, _)| n).collect();
    Err(suggestions)
}
