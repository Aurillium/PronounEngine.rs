use mysql_async::{Conn, prelude::{Queryable}};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::engine::{PronounSet, genderify_text};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag="type")]
pub enum SentenceType {
    Invalid = 255,
    NamesPronouns = 0,
    PronounsOnly = 1,
    NamesOnly = 2
}

// Ahhh, good old `generate_sentences`, like the return of an old friend
pub async fn generate_sentences(names: Vec<String>, sets: Vec<PronounSet>, mut db: Conn, before: &str, after: &str) -> Result<String, String> {
    if sets.len() == 0 && names.len() == 0 {
        return Ok("Can't make sentences with no names or pronouns :(".to_owned());
    }

    let mut text: String = before.to_owned();
    // Importantly, we don't compare filter whitespace. This allows us to still
    // not have any before text as Discord will remove spaces at the start for us
    if before.eq("") {
        text = "Okay, how do these look?".to_owned();
    }

    let sentence_type = match sets.len() {
        0 => 2,
        _ => match names.len() {
            0 => 1,
            _ => 0
        }
    };
    
    // It's an integer so we don't need to sanitise it
    let mut raw_sentences: Vec<String> = match db.query("SELECT Sentence FROM Sentences WHERE Type=".to_owned() + &sentence_type.to_string()).await {
        Ok(result) => result,
        Err(error) => return Err(error.to_string())
    };

    let mut rng = rand::thread_rng();
    for i in 1..4 {
        let index = rng.gen_range(0..raw_sentences.len());
        text += &format!("\n\n**Sentence {}**\n", i).to_owned();
        text += raw_sentences.swap_remove(index).as_str();
    }
    text += after;

    Ok(genderify_text(text.as_str(), names, sets))
}