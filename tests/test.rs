#[cfg(test)]
mod tests {
    use card_game::utils::utils::parse_json_to_deck_code;
    #[test]

    fn json_to_deck_code() {
        match parse_json_to_deck_code() {
            Ok(deckcodes) => {
                println!("{:#?}", deckcodes);
            }
            Err(_) => todo!(),
        }
    }
}
