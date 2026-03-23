use comfy_table::{ContentArrangement, Table};

pub fn print_table(value: &serde_json::Value) {
    match value {
        serde_json::Value::Array(arr) => print_array_table(arr),
        serde_json::Value::Object(obj) => print_object_table(obj),
        _ => println!("{}", value),
    }
}

fn print_array_table(arr: &[serde_json::Value]) {
    if arr.is_empty() {
        println!("(empty)");
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    if let Some(serde_json::Value::Object(first)) = arr.first() {
        let headers: Vec<String> = first.keys().cloned().collect();
        table.set_header(&headers);

        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                let row: Vec<String> = headers.iter().map(|k| format_value(obj.get(k))).collect();
                table.add_row(row);
            }
        }
    } else {
        table.set_header(vec!["value"]);
        for item in arr {
            table.add_row(vec![item.to_string()]);
        }
    }

    println!("{table}");
}

fn print_object_table(obj: &serde_json::Map<String, serde_json::Value>) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Key", "Value"]);

    for (k, v) in obj {
        table.add_row(vec![k.clone(), format_value(Some(v))]);
    }

    println!("{table}");
}

fn format_value(v: Option<&serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value_string() {
        let v = serde_json::Value::String("hello".to_string());
        assert_eq!(format_value(Some(&v)), "hello");
    }

    #[test]
    fn test_format_value_number() {
        let v = serde_json::json!(42);
        assert_eq!(format_value(Some(&v)), "42");
    }

    #[test]
    fn test_format_value_none() {
        assert_eq!(format_value(None), "");
    }

    #[test]
    fn test_format_value_null() {
        let v = serde_json::Value::Null;
        assert_eq!(format_value(Some(&v)), "null");
    }
}
