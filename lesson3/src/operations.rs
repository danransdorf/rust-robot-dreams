use std::{ error::Error };

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
    let mut output = String::new();

    if let Some((headers, data)) = input.split_once("\n") {
        let headers_row: Vec<&str> = headers
            .split(",")
            .map(|header| header.trim())
            .collect();
        let number_of_columns = headers_row.len();

        let mut column_max_widths: Vec<usize> = headers_row
            .clone()
            .into_iter()
            .map(|header| header.len())
            .collect();

        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut rdr = csv::Reader::from_reader(input.as_bytes());
        for result in rdr.records() {
            let row = result?;

            row.iter()
                .map(|field| field.trim().len())
                .enumerate()
                .for_each(|(index, field_length)| {
                    column_max_widths[index] = std::cmp::max(
                        column_max_widths[index],
                        field_length
                    );
                });

            rows.push(
                row
                    .iter()
                    .map(|field| field.trim().to_string())
                    .collect::<Vec<String>>()
            );
        }

        output.push_str(
            &headers_row
                .iter()
                .enumerate()
                .map(|(index, field)|
                    format!(" {:<width$} ", field, width = column_max_widths[index])
                )
                .collect::<Vec<String>>()
                .join("")
        );
        output.push_str("\n");
        output.push_str(
            &"_".repeat(
                column_max_widths
                    .iter()
                    .map(|&x| x)
                    .reduce(|a, b| a + b)
                    .unwrap_or(0 as usize) +
                    number_of_columns * 2 // Every column is padded with a space from each side
            )
        );
        for row in rows {
            output.push_str("\n");
            output.push_str(
                &row
                    .iter()
                    .enumerate()
                    .map(|(index, field)|
                        format!(" {:<width$} ", field, width = column_max_widths[index])
                    )
                    .collect::<Vec<String>>()
                    .join("")
            );
        }
    } else {
        eprintln!("Only one line was passed, parsed input: {}", input);
    }

    Ok(output)
}
