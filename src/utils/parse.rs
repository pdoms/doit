pub fn parse_search_value(text: &str) -> (String, Vec<String>) {
    let n = text[1..].find(':').unwrap() + 1;
    let key = text[1..n].to_string();
    let value = text[n+1..].to_string();
    let mut current = String::new();
    let mut values = Vec::new();
    for ch in value.chars() {
        if ch == ';' {
            values.push(current.clone());
            current = String::new();
        } else {
            current.push(ch.to_ascii_lowercase())
        }
    }
    if current.len() > 0 {
        values.push(current)
    }
    (key, values)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_search_value_one_value() {
        let status = "status".to_string();
        let value = ":status:Overdue";
        let value_2 = ":status:Done;";
        let parsed = parse_search_value(value);
        let parsed_2 = parse_search_value(value_2);
        assert_eq!(parsed.0, status);
        assert_eq!(parsed.1, vec!["overdue".to_string()]);
        assert_eq!(parsed_2.0, status);
        assert_eq!(parsed_2.1, vec!["done".to_string()]);
    }
    #[test]
    fn test_parse_search_value_multiple_value() {
        let status = "status".to_string();
        let value = ":status:Overdue;Done";
        let parsed = parse_search_value(value);
        assert_eq!(parsed.0, status);
        assert_eq!(parsed.1, vec!["overdue".to_string(), "done".to_string()]);
    }
}

