#[derive(Clone, Copy)]
pub enum RenameRule {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

pub fn rename_rule(rule: &str) -> Option<RenameRule> {
    Some(match rule {
        "lowercase" => RenameRule::Lowercase,
        "UPPERCASE" => RenameRule::Uppercase,
        "PascalCase" => RenameRule::PascalCase,
        "camelCase" => RenameRule::CamelCase,
        "snake_case" => RenameRule::SnakeCase,
        "SCREAMING_SNAKE_CASE" => RenameRule::ScreamingSnakeCase,
        "kebab-case" => RenameRule::KebabCase,
        "SCREAMING-KEBAB-CASE" => RenameRule::ScreamingKebabCase,
        _ => return None,
    })
}

pub fn apply_rename_rule(name: &str, rule: RenameRule) -> String {
    let words = get_words(name);

    match rule {
        RenameRule::Lowercase => words.join("").to_lowercase(),
        RenameRule::Uppercase => words.join("").to_uppercase(),
        RenameRule::PascalCase => words.iter().map(|word| capitalize(word)).collect(),
        RenameRule::CamelCase => {
            let mut out = String::new();
            for (index, word) in words.iter().enumerate() {
                if index == 0 {
                    out.push_str(&word.to_lowercase());
                } else {
                    out.push_str(&capitalize(word));
                }
            }
            out
        }
        RenameRule::SnakeCase => words.join("_").to_lowercase(),
        RenameRule::ScreamingSnakeCase => words.join("_").to_uppercase(),
        RenameRule::KebabCase => words.join("-").to_lowercase(),
        RenameRule::ScreamingKebabCase => words.join("-").to_uppercase(),
    }
}

fn get_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }

        if ch.is_uppercase() && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }

        current.push(ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}
