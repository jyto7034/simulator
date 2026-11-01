use std::io::Read;

fn main() {
    let mut source = String::new();
    std::io::stdin().take(2_000_001).read_to_string(&mut source).unwrap();
    if source.len() > 2_000_000 { eprintln!("input limit"); std::process::exit(2); }
    match std::env::args().nth(1).as_deref() {
        Some("kdl") => match kdl::KdlDocument::parse_v2(&source) {
            Ok(_) => println!("parsed"),
            Err(error) => { eprintln!("{error}"); std::process::exit(1); }
        },
        Some("yaml") => {
            use yaml_rust2::parser::{Event, Parser};
            let mut parser = Parser::new_from_str(&source);
            let mut depth = 0;
            loop {
                let (event, _) = match parser.next_token() {
                    Ok(event) => event,
                    Err(error) => { eprintln!("{error}"); std::process::exit(1); }
                };
                match event {
                    Event::MappingStart(..) | Event::SequenceStart(..) => {
                        depth += 1;
                        if depth > 64 { eprintln!("depth limit"); std::process::exit(2); }
                    }
                    Event::MappingEnd | Event::SequenceEnd => depth -= 1,
                    Event::StreamEnd => break,
                    _ => {},
                }
            }
            println!("parsed");
        }
        _ => std::process::exit(2),
    }
}
