use std::error::Error;

mod csv;

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
    let csv = csv::Csv::from(&input);

    Ok(csv.to_string())
}
