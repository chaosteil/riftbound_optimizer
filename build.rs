use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cards.json");
    
    let url = "https://gist.githubusercontent.com/OwenMelbz/e04dadf641cc9b81cb882b4612343112/raw/fadb466cfd014c6a47da41fc291ef5c436d4cf4f/riftbound.json";
    
    if let Ok(response) = ureq::get(url).call() {
        if let Ok(text) = response.into_string() {
            // Parse existing data
            if let Ok(mut cards) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                // Mock Spiritforged Data
                let mock_cards_json = r#"[
                    {
                        "name": "The Grand Duelist",
                        "cardType": [{"id": "legend", "label": "Legend"}],
                        "domains": [{"id": "order", "label": "Order"}, {"id": "body", "label": "Body"}],
                        "text": "<p>When you play a unit with [Deflect], it gains +1 :rb_might:.</p>",
                        "set": "SFD"
                    },
                    {
                        "name": "Fiora, Spiritforged",
                        "cardType": [{"id": "unit", "label": "Unit"}],
                        "domains": [{"id": "order", "label": "Order"}, {"id": "body", "label": "Body"}],
                        "text": "<p>While I'm [Mighty], I have [Deflect]. <em>(I'm Mighty while I have 5+ :rb_might:.)</em> When I attack, if I'm Mighty, deal 2 damage to all enemies.</p>",
                        "set": "SFD",
                        "energy": 6
                    }
                ]"#;
                
                let mock_cards: Vec<serde_json::Value> = serde_json::from_str(mock_cards_json).unwrap();
                cards.extend(mock_cards);
                
                let combined_json = serde_json::to_string(&cards).unwrap();
                fs::write(&dest_path, combined_json).expect("Failed to write cards.json");
            } else {
                fs::write(&dest_path, "[]").unwrap();
            }
        } else {
            fs::write(&dest_path, "[]").unwrap();
        }
    } else {
        fs::write(&dest_path, "[]").unwrap();
    }
    
    println!("cargo:rerun-if-changed=build.rs");
}