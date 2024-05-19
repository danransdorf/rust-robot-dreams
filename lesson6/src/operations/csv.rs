use csv::{Reader, StringRecord};
use std::{ cmp::max, error::Error, fmt };

type Row = Vec<String>;

pub struct Csv {
    headers: Row,
    rows: Vec<Row>,
}

fn to_trimmed_vector(row: &StringRecord) -> Row {
    row.iter()
        .map(|field| field.trim().to_string())
        .collect()
}

fn format_row(row: &Row, column_max_widths: &Vec<usize>) -> String {
    row.iter()
        .enumerate()
        .map(|(index, header)| format!(" {:<width$} ", header, width = column_max_widths[index]))
        .collect::<Row>()
        .join("|")
}

fn push_ln(string: &mut String, line_content: String) {
    string.push_str("\n");
    string.push_str(&line_content)
}

impl Csv {
    pub fn from(csv_string: &str) -> Result<Csv, Box<dyn Error>> {
        let mut reader = Reader::from_reader(csv_string.as_bytes());

        let headers = to_trimmed_vector(reader.headers()?);
        let rows: Vec<Row> = reader
            .records()
            .filter_map(|row_result| row_result.ok().map(|row| to_trimmed_vector(&row)))
            .collect();

        Ok(Csv { headers: headers, rows: rows })
    }

    fn get_max_widths(&self) -> Vec<usize> {
        let mut column_max_widths: Vec<usize> = self.headers
            .iter()
            .map(|header| header.len())
            .collect();

        for row in &self.rows {
            for (index, field) in row.iter().enumerate() {
                column_max_widths[index] = max(column_max_widths[index], field.len());
            }
        }

        column_max_widths
    }

    pub fn to_string(&self) -> String {
        let column_max_widths: Vec<usize> = self.get_max_widths();

        let mut formatted_csv = String::new();

        let formatted_headers = format_row(&self.headers, &column_max_widths);
        formatted_csv.push_str(&formatted_headers);

        push_ln(&mut formatted_csv, "_".repeat(formatted_headers.len()));

        for row in &self.rows {
            push_ln(&mut formatted_csv, format_row(row, &column_max_widths));
        }

        formatted_csv
    }
}

impl fmt::Display for Csv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
