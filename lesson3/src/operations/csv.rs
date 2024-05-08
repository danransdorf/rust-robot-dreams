use std::{ cmp::max, fmt };
use ::csv::Reader;
use csv::{ Error, StringRecord, StringRecordIter };

type Row = Vec<String>;

pub struct Csv {
    headers: Row,
    rows: Vec<Row>,
}

fn trim_collect(row: StringRecordIter) -> Row {
    row.map(|field| String::from(field.trim())).collect()
}
impl Csv {
    pub fn from(csv_string: &str) -> Csv {
        let mut reader = Reader::from_reader(csv_string.as_bytes());

        let headers = trim_collect(reader.headers().unwrap().iter());
        let rows: Vec<Vec<String>> = reader
            .records()
            .map(|row| trim_collect(row.unwrap().iter()))
            .collect();

        Csv { headers: headers, rows: rows }
    }
    fn get_max_widths(&self) -> Vec<usize> {
        self.headers
            .iter()
            .map(|header| header.len())
            .collect()
    }
    pub fn to_string(&self) -> String {
        let mut column_max_widths: Vec<usize> = self.get_max_widths();

        for row in &self.rows {
            for (index, field) in row.iter().enumerate() {
                column_max_widths[index] = max(column_max_widths[index], field.len());
            }
        }

        let mut output = String::new();

        let headers_string = self.headers
            .iter()
            .enumerate()
            .map(|(index, header)|
                format!(" {:<width$} ", header, width = column_max_widths[index])
            )
            .collect::<Vec<String>>()
            .join("|");

        output.push_str(&headers_string);

        output.push_str("\n");
        output.push_str(&"_".repeat(headers_string.len()));

        for row in &self.rows {
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

        output
    }
}

impl fmt::Display for Csv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
