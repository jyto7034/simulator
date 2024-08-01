use crate::enums::{PlayerType, UUID};

/// 클라이언트가 서버에게 보내는 ExecuteMsg
#[derive(Clone)]
pub enum Message {
    /// 게임 생성. 멀리건 카드 및 Player ID 등 반환함.
    CreateGame,

    /// 게임 시작
    EntryGame,

    /// client 가 server 에게 멀리건 카드를 요청함
    GetMulligunCards(i32),

    /// client 가 선택한 멀리건 카드를 server 로 전송
    SelectMulligunCard,

    /// 플레이어가 손패에서 카드를 사용함 ( 유닛, 스펠 상관 없음. )
    /// TODO: 카드를 원하는 슬롯에 배치할 수 있게 해야함.
    PlayCard(UUID),

    /// PlayCard 와 연계됨. 사용된 카드가 상대를 지정하는 카드일 때, 어떤 것들을 지정했는지 client 가 server 로 전송 ( 유닛, 스펠 상관 없음. )
    PlayCardWithTarget(UUID, Vec<UUID>),

    /// 카드를 더미로부터 한 장 뽑음
    DrawCard,

    /// 유닛으로 상대를 공격함. 선택한 유닛, 선택된 유닛을 json 데이터로 전달
    AttackTo,

    /// 턴 엔드
    TurnEnd,

    None,
}

pub enum Respones {
    /// 게임 생성. 멀리건 카드 및 Player ID 등 반환함.
    CreateGame,

    /// 게임 시작
    EntryGame,

    /// client 가 server 에게 멀리건 카드를 요청함
    GetMulligunCards(i32),

    /// client 가 선택한 멀리건 카드를 server 로 전송
    SelectMulligunCard,

    /// 플레이어가 손패에서 카드를 사용함 ( 유닛, 스펠 상관 없음. )
    PlayCard(UUID),

    /// PlayCard 와 연계됨. 사용된 카드가 상대를 지정하는 카드일 때, 어떤 것들을 지정했는지 client 가 server 로 전송 ( 유닛, 스펠 상관 없음. )
    PlayCardWithTarget(UUID, Vec<UUID>),

    /// 카드를 더미로부터 한 장 뽑음
    DrawCard,

    /// 유닛으로 상대를 공격함. 선택한 유닛, 선택된 유닛을 json 데이터로 전달
    AttackTo,

    /// 턴 엔드
    TurnEnd,

    None,
}

#[derive(Clone)]
pub struct MessageInfo {
    pub sender: PlayerType,
    pub msg: Message,
}

/// Msg 로 넘어온 json 을 분석하여 어떤 Behavior 인지 파악함.
pub fn analyze_message() -> MessageInfo {
    MessageInfo {
        sender: PlayerType::Player1,
        msg: Message::None,
    }
}
