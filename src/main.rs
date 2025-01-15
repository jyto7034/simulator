use clap::Parser;
// main
#[derive(Parser)]
#[command(
    name = "card game backend",           // 프로그램 이름
    author = env!("CARGO_PKG_AUTHORS"),       // 작성자
    version = env!("CARGO_PKG_VERSION"),           // 버전
    about = env!("CARGO_PKG_DESCRIPTION"),   // 짧은 설명
    long_about = None,         // 긴 설명 (None은 미사용)
)]
struct Args {
    #[arg(long = "p1_deck")]
    #[arg(required = true)]
    player_1_deckcode: String,

    #[arg(long = "p2_deck")]
    #[arg(required = true)]
    player_2_deckcode: String,

    #[arg(required = true)]
    attacker: usize,
}

fn main() {
    let args = Args::parse();
}
