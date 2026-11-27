pub fn acronym(name: &str) -> String {
    name.replace("'s ", " ")
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn possessives_are_dropped() {
        assert_eq!(super::acronym("Alpha's Cool Server"), "ACS");
    }
}
