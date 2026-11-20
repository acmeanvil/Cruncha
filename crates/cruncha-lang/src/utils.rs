// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! String helpers from Hurmet's `utils.js`, plus helpers that reproduce JavaScript string
//! semantics (UTF-16 lengths, `\s`, `trim`) where Hurmet's output depends on them.

/// JavaScript's `\s` (WhiteSpace plus LineTerminator).
pub fn is_js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// JavaScript `String.prototype.trim`.
pub fn js_trim(s: &str) -> &str {
    s.trim_matches(is_js_space)
}

/// Hurmet's `leadingSpaceRegEx`: `\s` without `\n`, `\t` and U+00A0.
pub(crate) fn is_leading_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\u{0C}' | '\r' | '\u{0B}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

pub(crate) fn trim_leading_space(s: &str) -> &str {
    s.trim_start_matches(is_leading_space)
}

/// Length in UTF-16 code units, as JavaScript's `String.length`.
pub fn utf16_len(s: &str) -> usize {
    s.chars().map(char::len_utf16).sum()
}

/// `s.substring(n)` with `n` in UTF-16 code units, clamped to the string.
pub(crate) fn skip_utf16(s: &str, n: usize) -> &str {
    let mut units = 0;
    for (i, c) in s.char_indices() {
        if units >= n {
            return &s[i..];
        }
        units += c.len_utf16();
    }
    ""
}

pub(crate) fn first_char(s: &str) -> Option<char> {
    s.chars().next()
}

/// Byte length of the first char, or 0.
pub(crate) fn first_char_len(s: &str) -> usize {
    s.chars().next().map_or(0, char::len_utf8)
}

/// `s.slice(0, -1)`: drop the last char.
pub(crate) fn drop_last_char(s: &mut String) {
    s.pop();
}

/// Byte offset of the char after the one starting at `pos` (JS `pos + 1`).
pub(crate) fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    pos + s[pos..].chars().next().map_or(0, char::len_utf8)
}

/// Convert a numeral superscript (⁰¹²³…⁻) to its ASCII numeral.
pub fn numeral_from_superscript(ch: char) -> char {
    match ch {
        '²' => '2',
        '³' => '3',
        '⁻' => '-',
        '¹' => '1',
        '⁰' => '0',
        _ => char::from_u32((ch as u32).wrapping_sub(0x2040)).unwrap_or('\u{FFFD}'),
    }
}

/// Trim spaces, CR, LF and FF but not tabs (for TSV).
pub fn tabless_trim(s: &str) -> &str {
    s.trim_matches(|c| matches!(c, ' ' | '\r' | '\n' | '\u{0C}'))
}

/// If a name has a numeric subscript, e.g. `x_2`, convert it to Unicode subscript, `x₂`.
pub fn check_for_numeric_subscript(var_name: &str) -> String {
    // /_([0-9]+)(?=(′|$))/
    for (i, _) in var_name.match_indices('_') {
        let rest = &var_name[i + 1..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            continue;
        }
        let after = &rest[digits..];
        if after.is_empty() || after.starts_with('′') {
            let subs: String = rest[..digits]
                .chars()
                .map(|d| char::from_u32(0x2080 + (d as u32 - '0' as u32)).unwrap_or(d))
                .collect();
            return format!("{}{}{}", &var_name[..i], subs, after);
        }
    }
    var_name.to_string()
}

/// The argument of a verbatim TeX function such as `\text{…}`.
pub(crate) fn verbatim_arg(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let Some(&open) = chars.first() else {
        return String::new();
    };
    if open != '{' && open != '(' && open != '[' {
        return String::new();
    }
    let close = match open {
        '{' => '}',
        '[' => ']',
        _ => ')',
    };
    let mut result = String::new();
    let mut level = 1;
    for i in 1..chars.len() {
        let ch = chars[i];
        if ch == open && (open == '(' || open == '[' || i == 1 || chars[i - 1] != '\\') {
            level += 1;
        } else if ch == close && (close == ')' || close == ']' || i == 1 || chars[i - 1] != '\\') {
            level -= 1;
        }
        if level == 0 {
            return result;
        }
        result.push(ch);
    }
    String::new()
}

fn text_accent(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{0300}' => "`",
        '\u{0301}' => "'",
        '\u{0302}' => "^",
        '\u{0303}' => "~",
        '\u{0304}' | '\u{0305}' => "=",
        '\u{0306}' => "u",
        '\u{0307}' => ".",
        '\u{0308}' => "\"",
        '\u{030A}' => "r",
        '\u{030C}' => "v",
        _ => return None,
    })
}

/// Insert TeX escapes for `# $ & % _ ~ ^` and convert combining accents to text accents.
pub fn add_text_escapes(input: &str) -> String {
    let mut s = input.to_string();
    if utf16_len(&s) <= 1 {
        return s;
    }
    let matches: Vec<(usize, char)> = s
        .char_indices()
        .filter(|(_, c)| matches!(c, '#' | '$' | '&' | '%' | '_' | '~' | '^'))
        .collect();
    for &(pos, c) in matches.iter().rev() {
        if c == '~' {
            s.replace_range(pos..pos + 1, "\\textasciitilde ");
        } else if c == '^' {
            s.replace_range(pos..pos + 1, "\\textasciicircum ");
        } else if pos == 0 {
            s.insert(0, '\\');
        } else if !s[..pos].ends_with('\\') {
            s.insert(pos, '\\');
        }
    }
    let accents: Vec<(usize, char)> = s
        .char_indices()
        .filter(|(_, c)| text_accent(*c).is_some())
        .collect();
    for &(pos, c) in accents.iter().rev() {
        if pos > 0 {
            let prev_start = s[..pos].char_indices().last().map_or(0, |(i, _)| i);
            let prev = s[prev_start..pos].to_string();
            let replacement = format!("\\{}{}", text_accent(c).unwrap_or(""), prev);
            s.replace_range(prev_start..pos + c.len_utf8(), &replacement);
        }
    }
    s
}

fn is_exponent_char(c: char) -> bool {
    matches!(c, '⁰' | '¹' | '²' | '³' | '\u{2074}'..='\u{2079}' | '⁻')
}

/// TeX for a unit name, wrapped in an extra pair of braces.
pub fn unit_tex_from_string(s: &str) -> String {
    let mut unit = String::from(" {\\text{");
    let mut in_exponent = false;
    for (i, c) in s.char_indices() {
        let ch = if is_exponent_char(c) {
            numeral_from_superscript(c)
        } else {
            c
        };
        let rest = &s[i..];
        let mid_dot = rest.starts_with('*')
            || rest.starts_with('·')
            || rest.starts_with('.')
            || (rest.starts_with('-') && rest[1..].starts_with(|c: char| c.is_ascii_alphabetic()));
        if mid_dot {
            unit.push_str("}\\mkern1mu{\\cdot}\\mkern1mu\\text{");
        } else if ch.is_ascii_digit() || ch == '-' {
            let t = if ch == '-' {
                "\\text{-}".to_string()
            } else {
                ch.to_string()
            };
            if in_exponent {
                unit.push_str(&t);
            } else {
                unit.push_str("}^{");
                unit.push_str(&t);
                in_exponent = true;
            }
        } else if ch == '^' {
            unit.push_str("}^{");
            in_exponent = true;
        } else if in_exponent {
            unit.push_str("}\\text{");
            unit.push(ch);
            in_exponent = false;
        } else if ch == '$' {
            unit.push_str("\\$");
        } else {
            unit.push(ch);
        }
    }
    unit.push_str("}}");
    unit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes() {
        assert_eq!(add_text_escapes("a_b"), "a\\_b");
        assert_eq!(add_text_escapes("#3"), "\\#3");
        assert_eq!(add_text_escapes("x"), "x");
    }

    #[test]
    fn units() {
        assert_eq!(unit_tex_from_string("psi"), " {\\text{psi}}");
        assert_eq!(unit_tex_from_string("m2"), " {\\text{m}^{2}}");
        assert_eq!(
            unit_tex_from_string("AU3/days2"),
            " {\\text{AU}^{3}\\text{/days}^{2}}"
        );
    }

    #[test]
    fn numeric_subscript() {
        assert_eq!(check_for_numeric_subscript("x_12"), "x₁₂");
        assert_eq!(check_for_numeric_subscript("L_(1,2)"), "L_(1,2)");
    }
}
