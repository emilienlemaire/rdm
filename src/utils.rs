pub fn full_expand(str: &str) -> String {
    let mut expanded = shellexpand::full(&str).unwrap().to_string();
    let mut new_expanded = shellexpand::full(&expanded).unwrap().to_string();

    while expanded != new_expanded {
        expanded = new_expanded;
        new_expanded = shellexpand::full(&expanded).unwrap().to_string();
    }

    expanded
}
