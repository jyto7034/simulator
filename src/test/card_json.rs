/// TODO: 마지막에 불필요한 , 를 붙힌다거나 json 문법이 틀렸을 경우 쉽게 확인할 수 있는 방법이 없음.

pub fn init_cards_json() -> (String, String) {
    let p1_deck = r#"
{
  "decks": [
    {
      "Hero": [
        {
          "name": "player1"
        }
      ],
      "cards": [
        {
          "id": "HM_001",
          "num": 2
        }
      ]
    }
  ]
}
    "#;
 
    let p2_deck = r#"
{
  "decks": [
    {
      "Hero": [
        {
          "name": "player1"
        }
      ],
      "cards": [
        {
          "id": "HM_001",
          "num": 2
        },
        {
          "id": "HM_002",
          "num": 2
        }
      ]
    }
  ]
}

    "#;

    (p1_deck.to_string(), p2_deck.to_string())
}
