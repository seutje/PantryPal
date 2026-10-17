//! Suggestions only: similarity never assigns a canonical ingredient ID.
use crate::domain::validation;

pub fn rank(query: &str, name: &str) -> Option<usize> {
    let name = validation::normalized(name);
    if name == query {
        return Some(0);
    }
    if name.starts_with(query) {
        return Some(1);
    }
    if name.contains(query) {
        return Some(2);
    }
    let query: Vec<char> = query.chars().collect();
    if query.len() < 3 {
        return None;
    }
    let name: Vec<char> = name.chars().collect();
    let limit = if query.len() >= 7 { 2 } else { 1 };
    if query.len().abs_diff(name.len()) > limit {
        return None;
    }
    // Optimal string alignment includes adjacent transpositions (e.g. onoin).
    let mut rows = vec![vec![0; name.len() + 1]; query.len() + 1];
    for (i, row) in rows.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in rows[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=query.len() {
        for j in 1..=name.len() {
            rows[i][j] = (rows[i - 1][j] + 1)
                .min(rows[i][j - 1] + 1)
                .min(rows[i - 1][j - 1] + usize::from(query[i - 1] != name[j - 1]));
            if i > 1 && j > 1 && query[i - 1] == name[j - 2] && query[i - 2] == name[j - 1] {
                rows[i][j] = rows[i][j].min(rows[i - 2][j - 2] + 1);
            }
        }
    }
    let distance = rows[query.len()][name.len()];
    (distance <= limit).then_some(3 + distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_exact_prefix_substring_and_typos_without_guessing_short_names() {
        assert_eq!(rank("onion", " Onion "), Some(0));
        assert_eq!(rank("oni", "Onion"), Some(1));
        assert_eq!(rank("onion", "Red onion"), Some(2));
        for query in ["onoin", "onon", "oniom", "onnion"] {
            assert_eq!(rank(query, "Onion"), Some(4));
        }
        assert!(rank("brocolli", "Broccoli").is_some());
        assert_eq!(rank("ox", "Onion"), None);
        assert_eq!(rank("onion", "Garlic"), None);
        assert_eq!(rank("sel", "Salt"), None);
    }
}
