#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ListParserError {
    // use these u32's to return the exact place where invalidness was
    // found, this way we can present an error in the GUI if we want
    InvalidCsv(u32),
    IllegalCharacter(u32),
    // may want to add more error cases, not sure of all of the ways that
    // parse() may fail
}
/// Parse a csv into an array of strings
/// <br>
/// Returns:
/// <br>
/// Ok(&[&str]) if the string could be parsed as valid CSV and split into
/// an array of sanitised strings
/// <br>
/// Err(ListParserError::InvalidCsv(column) if the parameter is invalid CSV.
/// <br>
/// Err(ListParserError::IllegalCharacter(column) if the parameter contains
/// an illegal character.
pub fn parse(csv_of_shopping_list_items: &str) -> Result<&[&str], ListParserError> {
    let _shut_up_warning = csv_of_shopping_list_items;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
}
