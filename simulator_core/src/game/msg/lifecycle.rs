use actix::{Context, Handler, Message};
use tracing::info;

use crate::{
    card::types::PlayerKind,
    exception::{GameError, GameplayError},
    game::GameActor,
    game::GameConfig,
};

/// `InitializeGame` 메시지는 게임 액터를 초기화하는 데 사용됩니다.
/// 이 메시지는 게임 설정 정보를 담고 있으며, 게임 액터가 시작될 때 전송됩니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::msg::lifecycle::InitializeGame;
/// use simulator_core::game::GameConfig;
///
/// let config = GameConfig::default(); // 또는 사용자 정의 설정
/// let init_msg = InitializeGame(config);
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct InitializeGame(pub GameConfig);

/// `RemovePlayerActor` 메시지는 게임에서 특정 플레이어 액터를 제거하는 데 사용됩니다.
/// 이 메시지는 제거할 플레이어의 종류를 지정합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::msg::lifecycle::RemovePlayerActor;
/// use simulator_core::card::types::PlayerKind;
///
/// let remove_msg = RemovePlayerActor { player_kind: PlayerKind::Human };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RemovePlayerActor {
    pub player_kind: PlayerKind,
}

/// `PlayerReady` 메시지는 플레이어가 게임에 참여할 준비가 되었음을 알리는 데 사용됩니다.
/// 이 메시지는 준비된 플레이어의 종류를 지정합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::msg::lifecycle::PlayerReady;
/// use simulator_core::card::types::PlayerKind;
///
/// let ready_msg = PlayerReady(PlayerKind::Human);
/// ```
#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerReady(pub PlayerKind);

/// `CheckReEntry` 메시지는 플레이어가 게임에 재진입할 수 있는지 확인하는 데 사용됩니다.
/// 이 메시지는 확인하려는 플레이어의 종류를 지정합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::msg::lifecycle::CheckReEntry;
/// use simulator_core::card::types::PlayerKind;
///
/// let check_msg = CheckReEntry { player_type: PlayerKind::Human };
/// ```
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct CheckReEntry {
    pub player_type: PlayerKind,
}

impl Handler<InitializeGame> for GameActor {
    type Result = Result<(), GameError>;

    /// `InitializeGame` 메시지를 처리합니다.
    /// 현재는 아무 작업도 수행하지 않고 성공을 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `msg` - `InitializeGame` 메시지.
    /// * `_` - 액터 컨텍스트.
    ///
    /// # Returns
    ///
    /// * `Result<(), GameError>` - 항상 `Ok(())`를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// // 이 핸들러는 실제로 호출되는 예시를 보여주기 어렵습니다.
    /// // 액터 시스템 내부에서 호출되기 때문입니다.
    /// ```
    fn handle(&mut self, msg: InitializeGame, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RemovePlayerActor> for GameActor {
    type Result = Result<(), GameError>;

    /// `RemovePlayerActor` 메시지를 처리합니다.
    /// 지정된 플레이어 액터를 게임에서 제거합니다.
    ///
    /// # Arguments
    ///
    /// * `msg` - `RemovePlayerActor` 메시지.
    /// * `_` - 액터 컨텍스트.
    ///
    /// # Returns
    ///
    /// * `Result<(), GameError>` - 성공하면 `Ok(())`, 플레이어를 찾을 수 없으면 `Err(GameError)`를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// // 이 핸들러는 실제로 호출되는 예시를 보여주기 어렵습니다.
    /// // 액터 시스템 내부에서 호출되기 때문입니다.
    /// ```
    fn handle(&mut self, msg: RemovePlayerActor, _: &mut Self::Context) -> Self::Result {
        info!("Removing player actor: {:?}", msg.player_kind);
        let player_identity = self
            .get_player_identity_by_kind(msg.player_kind)
            .cloned()
            .ok_or_else(|| {
                GameError::Gameplay(GameplayError::ResourceNotFound {
                    kind: "player_identity",
                    id: format!("{:?}", msg.player_kind),
                })
            })?;
        if let None = self.players.remove(&player_identity) {
            return Err(GameError::Gameplay(GameplayError::ResourceNotFound {
                kind: "player_identity",
                id: format!("{:?}", msg.player_kind),
            }));
        }
        Ok(())
    }
}

impl Handler<PlayerReady> for GameActor {
    type Result = ();

    /// `PlayerReady` 메시지를 처리합니다.
    /// 현재는 아무 작업도 수행하지 않습니다.
    ///
    /// # Arguments
    ///
    /// * `msg` - `PlayerReady` 메시지.
    /// * `_` - 액터 컨텍스트.
    ///
    /// # Returns
    ///
    /// * `()` - 항상 `()`를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// // 이 핸들러는 실제로 호출되는 예시를 보여주기 어렵습니다.
    /// // 액터 시스템 내부에서 호출되기 때문입니다.
    /// ```
    fn handle(&mut self, msg: PlayerReady, _: &mut Self::Context) -> Self::Result {}
}

impl Handler<CheckReEntry> for GameActor {
    type Result = Result<(), GameError>;

    /// `CheckReEntry` 메시지를 처리합니다.
    /// 플레이어의 재진입을 확인하는 로직을 구현해야 합니다.
    ///
    /// # Arguments
    ///
    /// * `msg` - `CheckReEntry` 메시지.
    /// * `_` - 액터 컨텍스트.
    ///
    /// # Returns
    ///
    /// * `Result<(), GameError>` - 성공하면 `Ok(())`, 실패하면 `Err(GameError)`를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// // 이 핸들러는 실제로 호출되는 예시를 보여주기 어렵습니다.
    /// // 액터 시스템 내부에서 호출되기 때문입니다.
    /// ```
    fn handle(&mut self, msg: CheckReEntry, _: &mut Context<Self>) -> Self::Result {
        todo!() // TODO: 재진입 로직 구현
    }
}
