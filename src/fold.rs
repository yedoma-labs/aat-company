//! Folding text into comparable tokens.

/// Words that only join parts of a name; dropped from the comparison key.
const CONNECTORS: &[&str] = &["and", "und", "et", "en", "og", "och"];

/// Lower-cases `c` and maps it to plain letters. German umlauts become `ae oe ue`
/// (`Müller` = `Mueller`), `ß` becomes `ss`; other Latin diacritics lose their mark.
fn fold_char(c: char, out: &mut String) {
    for l in c.to_lowercase() {
        match l {
            'ä' => out.push_str("ae"),
            'ö' | 'œ' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            'æ' => out.push_str("ae"),
            'þ' => out.push_str("th"),
            'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' | 'ă' | 'ą' => out.push('a'),
            'ç' | 'ć' | 'č' => out.push('c'),
            'ď' | 'đ' => out.push('d'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => out.push('e'),
            'ğ' => out.push('g'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => out.push('i'),
            'ł' => out.push('l'),
            'ñ' | 'ń' | 'ň' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ō' | 'ő' | 'ø' => out.push('o'),
            'ř' => out.push('r'),
            'ś' | 'š' | 'ş' => out.push('s'),
            'ť' | 'ţ' => out.push('t'),
            'ù' | 'ú' | 'û' | 'ū' | 'ů' | 'ű' | 'ų' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'ź' | 'ż' | 'ž' => out.push('z'),
            c if c.is_alphanumeric() => out.push(c),
            _ => out.push(' '),
        }
    }
}

/// Splits `text` into folded tokens at everything that is not a letter or digit.
/// `&` counts as the word `and`.
pub(crate) fn raw_tokens(text: &str) -> Vec<String> {
    let mut folded = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        if c == '&' {
            folded.push_str(" and ");
        } else {
            fold_char(c, &mut folded);
        }
    }
    folded.split_whitespace().map(str::to_string).collect()
}

/// `true` for the words that only join name parts (`and`, `und`, `et`, ...).
pub(crate) fn is_connector(token: &str) -> bool {
    CONNECTORS.contains(&token)
}

/// Connector spellings (`und`, `u`, `et`, ...) all stand for `and` when forms are compared.
pub(crate) fn form_token(token: &str) -> &str {
    if is_connector(token) || token == "u" {
        "and"
    } else {
        token
    }
}

/// Drops connectors and glues runs of single letters (`A.P. Moller` → `ap moller`).
pub(crate) fn core_tokens(tokens: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut run = String::new();
    for (i, t) in tokens.iter().enumerate().filter(|(_, t)| !is_connector(t)) {
        // a lone `u` between two longer words is the German `und` (`Müller u. Söhne`),
        // but in `U.S. Steel` it is an initial
        let long = |j: Option<&String>| j.is_some_and(|w| w.chars().count() > 1);
        if t == "u" && long(i.checked_sub(1).and_then(|j| tokens.get(j))) && long(tokens.get(i + 1))
        {
            continue;
        }
        if t.chars().count() == 1 {
            run.push_str(t);
            continue;
        }
        if !run.is_empty() {
            out.push(std::mem::take(&mut run));
        }
        out.push(t.clone());
    }
    if !run.is_empty() {
        out.push(run);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umlauts_follow_german_transliteration() {
        assert_eq!(raw_tokens("Müller & Söhne"), ["mueller", "and", "soehne"]);
        assert_eq!(raw_tokens("Straße"), ["strasse"]);
        assert_eq!(raw_tokens("Łódź Café"), ["lodz", "cafe"]);
    }

    #[test]
    fn single_letters_are_glued() {
        let t = raw_tokens("A.P. Møller");
        assert_eq!(core_tokens(&t), ["ap", "moller"]);
        assert_eq!(core_tokens(&raw_tokens("H&M")), ["hm"]);
        assert_eq!(core_tokens(&raw_tokens("H & M")), ["hm"]);
    }

    #[test]
    fn lone_u_is_und_only_between_words() {
        assert_eq!(
            core_tokens(&raw_tokens("Müller u. Söhne")),
            ["mueller", "soehne"]
        );
        assert_eq!(core_tokens(&raw_tokens("U.S. Steel")), ["us", "steel"]);
    }

    #[test]
    fn connectors_are_dropped() {
        assert_eq!(
            core_tokens(&raw_tokens("Johnson und Johnson")),
            ["johnson", "johnson"]
        );
    }
}
