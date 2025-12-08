use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 게임 밸런스 설정 전체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBalanceConfig {
    pub qliphoth: QliphothConfig,
}

/// 클리포트 시스템 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QliphothConfig {
    pub thresholds: QliphothThresholds,
    pub suppress_chance: QliphothSuppressChance,
    pub changes: QliphothChanges,
    pub reward_multipliers: QliphothRewardMultipliers,
}

/// 클리포트 임계값 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QliphothThresholds {
    pub stable_min: u32,
    pub stable_max: u32,
    pub caution_min: u32,
    pub caution_max: u32,
    pub critical_min: u32,
    pub critical_max: u32,
    pub meltdown: u32,
}

/// 진압 작업 발생 확률 설정 (0~100)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QliphothSuppressChance {
    pub stable: u32,    // 안정 상태 (보통 0)
    pub caution: u32,   // 주의 상태 (보통 30~50)
    pub critical: u32,  // 위험 상태 (보통 100, 강제)
}

impl QliphothSuppressChance {
    /// 설정값이 유효한지 검증 (0~100 범위)
    pub fn validate(&self) -> Result<(), String> {
        if self.stable > 100 {
            return Err(format!(
                "Invalid suppress_chance.stable: {} (must be 0~100)",
                self.stable
            ));
        }
        if self.caution > 100 {
            return Err(format!(
                "Invalid suppress_chance.caution: {} (must be 0~100)",
                self.caution
            ));
        }
        if self.critical > 100 {
            return Err(format!(
                "Invalid suppress_chance.critical: {} (must be 0~100)",
                self.critical
            ));
        }
        Ok(())
    }
}

/// 클리포트 변화량 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QliphothChanges {
    pub battle_cost: u32,
    pub suppress_success: u32,
    pub suppress_failure: u32,
    pub breach_success: u32,
    pub breach_failure: u32,
    pub phase_recovery: u32,
    pub item_recovery: u32,
}

/// 보상 배율 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QliphothRewardMultipliers {
    pub stable: f32,
    pub caution_suppress: f32,
    pub critical_breach: f32,
}

/// 전역 게임 밸런스 설정 인스턴스
static GAME_BALANCE: Lazy<GameBalanceConfig> = Lazy::new(|| {
    GameBalanceConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load game balance config: {}. Using defaults.", e);
        GameBalanceConfig::default()
    })
});

impl GameBalanceConfig {
    /// 전역 설정 인스턴스 가져오기
    pub fn global() -> &'static GameBalanceConfig {
        &GAME_BALANCE
    }

    /// 설정 파일 로드
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // 1. 설정 파일 경로 찾기
        let config_path = Self::find_config_file()?;

        // 2. TOML 파일 읽기
        let config_content = std::fs::read_to_string(&config_path)?;

        // 3. TOML 파싱
        let config: GameBalanceConfig = toml::from_str(&config_content)?;

        // 4. 설정값 검증
        config.qliphoth.suppress_chance.validate()?;

        tracing::info!("Game balance config loaded from: {:?}", config_path);
        Ok(config)
    }

    /// 설정 파일 위치 찾기
    fn find_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 1. 현재 작업 디렉토리에서 찾기
        let cwd = std::env::current_dir()?;
        let cwd_config = cwd.join("core/config/game_balance.toml");
        if cwd_config.exists() {
            return Ok(cwd_config);
        }

        // 2. 실행 파일 위치 기준으로 찾기
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_config = exe_dir.join("core/config/game_balance.toml");
                if exe_config.exists() {
                    return Ok(exe_config);
                }
            }
        }

        // 3. 컴파일 타임에 포함된 경로 사용 (fallback)
        // include_str!로 임베드된 설정을 사용
        Err("Config file not found, using embedded defaults".into())
    }

    /// 기본 설정값
    pub fn default() -> Self {
        Self {
            qliphoth: QliphothConfig {
                thresholds: QliphothThresholds {
                    stable_min: 10,
                    stable_max: 7,
                    caution_min: 6,
                    caution_max: 4,
                    critical_min: 3,
                    critical_max: 1,
                    meltdown: 0,
                },
                suppress_chance: QliphothSuppressChance {
                    stable: 0,
                    caution: 50,
                    critical: 100,
                },
                changes: QliphothChanges {
                    battle_cost: 1,
                    suppress_success: 2,
                    suppress_failure: 1,
                    breach_success: 3,
                    breach_failure: 2,
                    phase_recovery: 1,
                    item_recovery: 2,
                },
                reward_multipliers: QliphothRewardMultipliers {
                    stable: 1.0,
                    caution_suppress: 1.5,
                    critical_breach: 2.5,
                },
            },
        }
    }
}

/// 편의 헬퍼 함수들
pub mod balance {
    use super::GameBalanceConfig;

    /// 클리포트 설정 가져오기
    pub fn qliphoth() -> &'static super::QliphothConfig {
        &GameBalanceConfig::global().qliphoth
    }

    /// 클리포트 임계값 가져오기
    pub fn qliphoth_thresholds() -> &'static super::QliphothThresholds {
        &qliphoth().thresholds
    }

    /// 클리포트 변화량 가져오기
    pub fn qliphoth_changes() -> &'static super::QliphothChanges {
        &qliphoth().changes
    }

    /// 클리포트 진압 확률 가져오기
    pub fn qliphoth_suppress_chance() -> &'static super::QliphothSuppressChance {
        &qliphoth().suppress_chance
    }

    /// 클리포트 보상 배율 가져오기
    pub fn qliphoth_reward_multipliers() -> &'static super::QliphothRewardMultipliers {
        &qliphoth().reward_multipliers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameBalanceConfig::default();

        // 임계값 검증
        assert_eq!(config.qliphoth.thresholds.stable_min, 10);
        assert_eq!(config.qliphoth.thresholds.stable_max, 7);
        assert_eq!(config.qliphoth.thresholds.caution_min, 6);
        assert_eq!(config.qliphoth.thresholds.caution_max, 4);
        assert_eq!(config.qliphoth.thresholds.critical_min, 3);
        assert_eq!(config.qliphoth.thresholds.critical_max, 1);
        assert_eq!(config.qliphoth.thresholds.meltdown, 0);

        // 진압 확률 검증
        assert_eq!(config.qliphoth.suppress_chance.stable, 0);
        assert_eq!(config.qliphoth.suppress_chance.caution, 50);
        assert_eq!(config.qliphoth.suppress_chance.critical, 100);
    }

    #[test]
    fn test_global_config() {
        let config = GameBalanceConfig::global();
        assert!(config.qliphoth.thresholds.stable_min > 0);
    }

    #[test]
    fn test_balance_helpers() {
        let thresholds = balance::qliphoth_thresholds();
        assert_eq!(thresholds.stable_min, 10);

        let changes = balance::qliphoth_changes();
        assert_eq!(changes.battle_cost, 1);

        let suppress_chance = balance::qliphoth_suppress_chance();
        assert_eq!(suppress_chance.stable, 0);
        assert_eq!(suppress_chance.caution, 50);
        assert_eq!(suppress_chance.critical, 100);
    }

    #[test]
    fn test_suppress_chance_validation() {
        // 유효한 값들
        let valid = QliphothSuppressChance {
            stable: 0,
            caution: 50,
            critical: 100,
        };
        assert!(valid.validate().is_ok());

        // stable > 100
        let invalid_stable = QliphothSuppressChance {
            stable: 101,
            caution: 50,
            critical: 100,
        };
        assert!(invalid_stable.validate().is_err());

        // caution > 100
        let invalid_caution = QliphothSuppressChance {
            stable: 0,
            caution: 150,
            critical: 100,
        };
        assert!(invalid_caution.validate().is_err());

        // critical > 100
        let invalid_critical = QliphothSuppressChance {
            stable: 0,
            caution: 50,
            critical: 200,
        };
        assert!(invalid_critical.validate().is_err());
    }

    #[test]
    fn test_suppress_chance_probability() {
        // 확률 계산 시뮬레이션
        let suppress_chance = QliphothSuppressChance {
            stable: 0,
            caution: 50,
            critical: 100,
        };

        // Stable: 0% - 절대 발생 안 함
        for roll in 0..100 {
            assert!(roll >= suppress_chance.stable);
        }

        // Caution: 50% - 0~49일 때만 발생
        let mut count = 0;
        for roll in 0..100 {
            if roll < suppress_chance.caution {
                count += 1;
            }
        }
        assert_eq!(count, 50); // 정확히 50%

        // Critical: 100% - 항상 발생
        for roll in 0..100 {
            assert!(roll < suppress_chance.critical);
        }
    }

    #[test]
    fn test_toml_serialization() {
        let config = GameBalanceConfig::default();
        let toml_string = toml::to_string_pretty(&config).unwrap();

        // 다시 역직렬화
        let deserialized: GameBalanceConfig = toml::from_str(&toml_string).unwrap();

        assert_eq!(
            config.qliphoth.thresholds.stable_min,
            deserialized.qliphoth.thresholds.stable_min
        );
    }
}
