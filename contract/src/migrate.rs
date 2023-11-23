use crate::types::Fraction;
use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PanicOnDefault};

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Fraction0 {
    pub num: u128,
    pub den: u128,
}

impl From<Fraction0> for Fraction {
    fn from(f: Fraction0) -> Self {
        Self {
            num: f.num.into(),
            den: f.den.into(),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract0 {
    pub pool: u128,
    pub max_bet_ratio: Fraction0,
    pub winning_proba: Fraction0,
    pub last_block: u64,
    pub seed: Vec<u8>,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        let state: Contract0 = env::state_read().expect("No state to migrate from");
        let mut new_state = Contract::init(state.max_bet_ratio.into(), state.winning_proba.into());
        new_state.pool = state.pool.into();
        new_state
    }
}

#[cfg(test)]
mod tests {
    use near_gas::NearGas;
    use near_sdk::json_types::U128;
    use near_workspaces::types::NearToken;
    use serde_json::json;

    use crate::test_utils::NEAR;

    #[tokio::test]
    async fn test_migrate() -> anyhow::Result<()> {
        let worker = near_workspaces::sandbox().await?;
        let wasm_v00 = std::fs::read("res/simple_bet_v00.wasm").unwrap();
        let contract = worker.dev_deploy(&wasm_v00).await?;
        const TOP_UP_AMOUNT: u128 = 1;

        contract
            .call("init")
            .args_json(json!({
                "max_bet_ratio": {
                    "num": 10,
                    "den": 100
                },
                "winning_proba": {
                    "num": 128,
                    "den": 256
                }
            }))
            .gas(NearGas::from_tgas(10))
            .transact()
            .await?
            .into_result()?;

        let owner = worker.dev_create_account().await?;
        owner
            .call(contract.id(), "top_up")
            .deposit(NearToken::from_near(TOP_UP_AMOUNT))
            .transact()
            .await?
            .into_result()?;

        let player = worker.dev_create_account().await?;
        let result = player.call(contract.id(), "pool").view().await?;
        assert_eq!(result.json::<u128>().unwrap(), TOP_UP_AMOUNT * NEAR);

        let wasm = std::fs::read("res/simple_bet.wasm").unwrap();
        contract.as_account().deploy(&wasm).await?.into_result()?;

        contract
            .call("migrate_state")
            .transact()
            .await?
            .into_result()?;

        let result = player.call(contract.id(), "pool").view().await?;
        assert_eq!(result.json::<U128>().unwrap().0, TOP_UP_AMOUNT * NEAR);

        Ok(())
    }
}
