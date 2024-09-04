pub fn normalize_name(description: &Option<String>) -> String {
    if description.is_none() {
        return String::new();
    }

    let step1 = description
        .as_ref()
        .unwrap()
        .replace('?', "")
        .replace("&amp;", "&");
    let step2: String = step1.split('(').next().unwrap_or("").to_string();

    let mut result = String::new();
    let mut capitalize_next = true;
    for c in step2.chars() {
        if c.is_whitespace() {
            capitalize_next = true;
            result.push(c);
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
