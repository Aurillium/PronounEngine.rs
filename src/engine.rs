use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PronounSet {
    subjective: String,
    objective: String,
    possessive: String,
    possessive2: String,
    reflexive: String,
    plural: bool
}

static HE_HIM: Lazy<PronounSet> = Lazy::new(|| {
    PronounSet {
        subjective: "he".to_owned(),
        objective: "him".to_owned(),
        possessive: "his".to_owned(),
        possessive2: "his".to_owned(),
        reflexive: "himself".to_owned(),
        plural: false
    }
});
static SHE_HER: Lazy<PronounSet> = Lazy::new(|| {
    PronounSet{
        subjective: "she".to_owned(),
        objective: "her".to_owned(),
        possessive: "her".to_owned(),
        possessive2: "hers".to_owned(),
        reflexive: "herself".to_owned(),
        plural: false
    }
});
static THEY_THEM: Lazy<PronounSet> = Lazy::new(|| {
    PronounSet{
        subjective: "they".to_owned(),
        objective: "them".to_owned(),
        possessive: "their".to_owned(),
        possessive2: "theirs".to_owned(),
        reflexive: "themself".to_owned(),
        plural: true
    }
});
static IT_IT: Lazy<PronounSet> = Lazy::new(|| {
    PronounSet{
        subjective: "it".to_owned(),
        objective: "it".to_owned(),
        possessive: "its".to_owned(),
        possessive2: "its".to_owned(),
        reflexive: "itself".to_owned(),
        plural: false
    }
});

fn second_possessive(first: &str) -> String {
    let last_letter = first.chars().last().unwrap();
    if last_letter != 's' && last_letter != 'z' {
        let mut possessive2: String = first.to_string();
        possessive2.push_str("s");
        return possessive2;
    } else {
        return first.to_string();
    }
}

const GENERIC_FORMAT_ERROR: &str = "Pronouns must be in the form `subjective/objective/possessive/posessive/reflexive`. See `/help` for more info.";

pub fn parse_set(raw: &str) -> Result<PronounSet, &str> {
    let colon_index = raw.find(":");
    let mut plural = false;
    let mut colon_removed = raw.trim().to_lowercase();
    match colon_index {
        None => (),
        Some(index) => {
            let plural_str = &raw[index + 1..].trim().to_lowercase();
            colon_removed = raw[..index].to_lowercase();
            if plural_str.eq("p") || plural_str.eq("pl") || plural_str.eq("plural") {
                plural = true;
            } else if !(plural_str.eq("s") || plural_str.eq("singular")) {
                // This means some invalid value was specified
                // Throw an error
            }
        }
    }
    let splitter = colon_removed.split("/");
    let mut split: Vec<String> = Vec::new();
    for term in splitter {
        if term.is_empty() {
            return Err("There needs to be a pronoun between each slash.");
        }
        split.push(term.to_string());
    }

    match split.len() {
        1 => {
            let first = split[0].clone();
            if first.eq("he") {
                return Ok(HE_HIM.clone());
            } else if first.eq("she") {
                return Ok(SHE_HER.clone());
            } else if first.eq("they") {
                return Ok(THEY_THEM.clone());
            } else if first.eq("it") {
                return Ok(IT_IT.clone());
            } else {
                Err(GENERIC_FORMAT_ERROR)
            }
        }
        2 => {
            let first = split[0].clone();
            let second = split[1].clone();
            if first.eq("he") && second.eq("him") {
                return Ok(HE_HIM.clone());
            } else if first.eq("she") && second.eq("her") {
                return Ok(SHE_HER.clone());
            } else if first.eq("they") && second.eq("them") {
                return Ok(THEY_THEM.clone());
            } else if first.eq("it") && (second.eq("it") || second.eq("its")) {
                return Ok(IT_IT.clone());
            } else {
                Err(GENERIC_FORMAT_ERROR)
            }
        }
        3 => {
            let subjective = split[0].clone();
            let mut reflexive = subjective.clone();
            reflexive.push_str("self");
            let possessive = split[2].clone();
            let possessive2 = second_possessive(&possessive);
            Ok(PronounSet {
                subjective: subjective,
                objective: split[1].clone(),
                possessive: possessive,
                possessive2: possessive2,
                reflexive: reflexive.to_string(),
                plural: true
            })
        }
        4 => {
            let subjective = split[0].clone();
            let possessive = split[2].clone();
            let possessive2 = second_possessive(&possessive);
            Ok(PronounSet {
                subjective: subjective,
                objective: split[1].clone(),
                possessive: possessive,
                possessive2,
                reflexive: split[3].clone(),
                plural
            })
        }
        5 => {
            Ok(PronounSet {
                subjective: split[0].clone(),
                objective: split[1].clone(),
                possessive: split[2].clone(),
                possessive2: split[3].clone(),
                reflexive: split[4].clone(),
                plural
            })
        }
        _ => {
            Err(GENERIC_FORMAT_ERROR)
        }
    }
}

//const piece = new RegExp("[^\u200B]\[(.*?)\]");

// Zero-width space is the escape character to prevent matches on user input
// All instances of it get removed at the end but it's invisible so no one will
// notice it's gone if they added one to their input anyway
const PIECE: &str = r"(?:\{(.*?)\|(.*?)}( ?))?\[([^\u200B].*?)\](?:( ?\S*? ?)\{(.*?)\|(.*?)})?";
// Returns: [whole, singular, plural, gap, pronoun, gap, singular, plural] because even I won't be able to understand this in two days

pub fn genderify_text(text: &str, names: Vec<String>, sets: Vec<PronounSet>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(PIECE).unwrap();
    }

    let mut final_text = text.to_owned();
    
    // While there are matches, loop
    for match_result in RE.captures_iter(text) {
        let central_match = match_result.get(4);
        let mut central = "";
        let mut capitals: u8 = 0;
        let mut form_before = "";
        let mut form_after = "";

        match central_match {
            Some(central_match) => {
                central = central_match.as_str();
            }
            None => () // Invalid sentence
        };

        match central.chars().nth(0) {
            Some(char) => {
                if char == '^' {
                    capitals = 1; // First letter
                    central = &central[1..];
                }
            }
            None => () // Invalid sentence
        }
        match central.chars().nth_back(0) {
            Some(char) => {
                if char == '^' {
                    capitals = 2; // All
                    central = &central[..central.len() - 1];
                }
            }
            None => () // Error
        }
        
        match central {
            "name" => {
                let name = names.choose(&mut rand::thread_rng());

                match name {
                    Some(name) => {
                        central = name;
                        if capitals == 0 {
                            capitals = 1;
                        }
                    }
                    None => () // This is covered in a previous function
                }
            }
            "subjective" | "objective" | "possessive" | "possessive2" | "reflexive" => {
                // Pick a random set
                let set = sets.choose(&mut rand::thread_rng());

                match set {
                    Some(set) => {
                        // Set the central
                        match central {
                            // These are all &Strings coerced to &strs
                            "subjective" => central = &set.subjective,
                            "objective" => central = &set.objective,
                            "possessive" => central = &set.possessive,
                            "possessive2" => central = &set.possessive2,
                            "reflexive" => central = &set.reflexive,
                            _ => ()
                        }

                        if set.plural {
                            // Plural set

                            match match_result.get(2) {
                                Some(before) => {
                                    form_before = before.as_str();
                                }
                                None => ()
                            };
                            match match_result.get(7) {
                                Some(after) => {
                                    form_after = after.as_str();
                                }
                                None => ()
                            };
                        } else {
                            // Singular set

                            match match_result.get(1) {
                                Some(before) => {
                                    form_before = before.as_str();
                                }
                                None => ()
                            };
                            match match_result.get(6) {
                                Some(after) => {
                                    form_after = after.as_str();
                                }
                                None => ()
                            };
                        }
                    }
                    None => () // This is covered in a previous function
                }
            }
            _ => ()
        }

        // Apply capitalisation
        let applied_central: String = match capitals {
            0 => central.to_owned(),
            1 => {
                // Capitalise first letter
                match central.chars().nth(0) {
                    Some(char) => char.to_uppercase().to_string() + &central[1..].to_lowercase(),
                    None => "".to_owned() // Not possible to have a blank string here
                }
            },
            2 => central.to_uppercase(),
            _ => "".to_owned() // No other possibilities
        };

        let mut parsed: String = "".to_owned();
        match match_result.get(1) {
            Some(_) => {
                parsed += &(form_before.to_owned() + match match_result.get(3) {
                    Some(val) => val.as_str(),
                    None => ""
                })
            }
            None => ()
        }
        parsed += &applied_central.replace("[", "[ESCAPE CODE");
        match match_result.get(6) {
            Some(_) => {
                parsed += &(match match_result.get(5) {
                    Some(val) => val.as_str(),
                    None => ""
                }.to_owned() + form_after)
            }
            None => ()
        }

        final_text = final_text.replace(match_result.get(0).expect("This should be impossible").as_str(), parsed.as_str());
    }
    final_text
}