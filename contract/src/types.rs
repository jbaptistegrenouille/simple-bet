use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, AccountId};
use serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
pub struct Fraction {
    pub num: U128,
    pub den: U128,
}

impl Fraction {
    pub fn mul(&self, other: U128) -> U128 {
        (self.num.0 * other.0 / self.den.0).into()
    }
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum BetResult {
    Win,
    Lose,
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Event {
    bettor: AccountId,
    result: BetResult,
    bet: U128,
    timestamp: u64,
    block_height: u64,
}

impl Event {
    pub fn new(bettor: AccountId, result: BetResult, bet: U128) -> Self {
        Self {
            bettor,
            result,
            bet,
            timestamp: env::block_timestamp(),
            block_height: env::block_height(),
        }
    }

    pub fn emit(&self) {
        let data = serde_json::to_string(&self).unwrap();
        env::log_str(format!(r#"EVENT_JSON:{{"standard":"simple-bet","version":"1.0.0","event":"bet","data":[{}]}}"#, data).as_str());
    }
}
