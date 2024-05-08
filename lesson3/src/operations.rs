use std::{ cmp, error::Error };

pub fn lowercase(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.to_lowercase())
}

pub fn uppercase(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.to_uppercase())
}

pub fn no_spaces(input: String) -> Result<String, Box<dyn Error>> {
    Ok(input.replace(" ", ""))
}

pub fn slugify(input: String) -> Result<String, Box<dyn Error>> {
    Ok(slug::slugify(input))
}

pub fn csv(input: String) -> Result<String, Box<dyn Error>> {
    let mut reader = csv::Reader::from_reader(input.as_bytes());

    let headers = reader
        .headers()?
        .iter()
        .map(|field| String::from(field.trim()))
        .collect::<Vec<String>>();

    let rows: Vec<Vec<String>> = reader
        .records()
        .map(|row|
            row
                .unwrap()
                .iter()
                .map(|field| String::from(field.trim()))
                .collect()
        )
        .collect();

    let mut column_max_widths: Vec<usize> = headers
        .iter()
        .map(|header| header.len())
        .collect();

    for row in &rows {
        for (index, field) in row.iter().enumerate() {
            column_max_widths[index] = cmp::max(column_max_widths[index], field.len());
        }
    }

    let mut output = String::new();

    let headers_string = headers
        .iter()
        .enumerate()
        .map(|(index, header)| format!(" {:<width$} ", header, width = column_max_widths[index]))
        .collect::<Vec<String>>()
        .join("|");

    output.push_str(&headers_string);

    output.push_str("\n");
    output.push_str(&"_".repeat(headers_string.len()));

    for row in &rows {
        output.push_str("\n");
        output.push_str(
            &row
                .iter()
                .enumerate()
                .map(|(index, header)|
                    format!(" {:<width$} ", header, width = column_max_widths[index])
                )
                .collect::<Vec<String>>()
                .join("|")
        );
    }

    Ok(output)
}
