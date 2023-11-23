use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Gas, PanicOnDefault, Promise};
use types::{BetResult, Event, Fraction};

pub mod migrate;
#[cfg(test)]
pub mod test_utils;
pub mod types;

const KEEP_EVENTS: usize = 32;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub pool: U128,
    pub max_bet_ratio: Fraction,
    pub winning_proba: Fraction,
    pub last_block: u64,
    pub seed: Vec<u8>,
    pub last_events: Vec<Event>,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init(ignore_state)]
    pub fn init(max_bet_ratio: Fraction, winning_proba: Fraction) -> Self {
        require!(
            0 < max_bet_ratio.num.0 && max_bet_ratio.num.0 <= max_bet_ratio.den.0,
            "Max bet ratio must be in the range (0, 1]"
        );

        require!(
            0 < winning_proba.num.0 && winning_proba.num.0 <= winning_proba.den.0,
            "Winning probability must be in the range (0, 1]"
        );

        require!(winning_proba.den.0 == 256, "Denominator must be 256");

        Self {
            pool: U128(0),
            max_bet_ratio,
            winning_proba,
            last_block: env::block_height(),
            seed: env::random_seed(),
            last_events: vec![],
        }
    }

    pub fn pool(&self) -> U128 {
        self.pool.into()
    }

    pub fn max_bet_ratio(&self) -> Fraction {
        self.max_bet_ratio.clone()
    }

    pub fn winning_proba(&self) -> Fraction {
        self.winning_proba.clone()
    }

    pub fn events(&self) -> Vec<Event> {
        self.last_events.clone()
    }

    #[private]
    pub fn set_max_bet_ratio(&mut self, max_bet_ratio: Fraction) {
        require!(
            0 < max_bet_ratio.num.0 && max_bet_ratio.num.0 <= max_bet_ratio.den.0,
            "Max bet ratio must be in the range (0, 1]"
        );

        self.max_bet_ratio = max_bet_ratio;
    }

    #[private]
    pub fn set_winning_proba(&mut self, winning_proba: Fraction) {
        require!(
            0 < winning_proba.num.0 && winning_proba.num.0 <= winning_proba.den.0,
            "Winning probability must be in the range (0, 1]"
        );

        require!(winning_proba.den.0 == 256, "Denominator must be 256");

        self.winning_proba = winning_proba;
    }

    #[payable]
    pub fn top_up(&mut self) {
        self.pool.0 += env::attached_deposit();
    }

    #[payable]
    pub fn bet(&mut self) {
        let max_bet = self.max_bet_ratio.mul(self.pool).0;
        let deposit = env::attached_deposit();
        let current_bet = std::cmp::min(deposit, max_bet);
        let refund_balance = deposit - current_bet;

        if current_bet == 0 {
            env::log_str("Current bet is 0, refunding the deposit.");
            Promise::new(env::predecessor_account_id()).transfer(refund_balance);
            return;
        }

        self.pool.0 -= current_bet;

        if refund_balance > 0 {
            env::log_str(format!("Refund balance {}", refund_balance).as_str());
            Promise::new(env::predecessor_account_id()).transfer(refund_balance);
        }

        Self::ext(env::current_account_id())
            .with_static_gas(Gas(5_000_000_000_000))
            .do_bet(U128(current_bet), env::predecessor_account_id());
    }

    #[private]
    pub fn do_bet(&mut self, bet: U128, bettor: AccountId) {
        // Fetch entropy from latest block
        if self.last_block != env::block_height() {
            self.seed = env::random_seed();
            self.last_block = env::block_height();
        }

        env::log_str(format!("Seed {:?}", self.seed).as_str());

        if let Some(seed) = self.seed.pop() {
            let bet_result = if seed < self.winning_proba.num.0 as u8 {
                Promise::new(bettor.clone()).transfer(2 * bet.0);
                BetResult::Win
            } else {
                self.pool.0 += 2 * bet.0;
                BetResult::Lose
            };

            let event = Event::new(bettor, bet_result, bet);
            event.emit();

            self.last_events.push(event.clone());
            if self.last_events.len() > KEEP_EVENTS {
                self.last_events.remove(0);
            }
        } else {
            env::log_str("Not enough entropy to play, refunding the deposit.");
            Promise::new(bettor).transfer(bet.0);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use near_gas::NearGas;
    use near_workspaces::types::NearToken;
    use serde_json::json;

    use crate::test_utils::{debug_result, NEAR};

    #[tokio::test]
    async fn test_workspace() -> anyhow::Result<()> {
        let worker = near_workspaces::sandbox().await?;
        let wasm = std::fs::read("res/simple_bet.wasm").unwrap();
        let contract = worker.dev_deploy(&wasm).await?;
        const TOP_UP_AMOUNT: u128 = 1;

        contract
            .call("init")
            .args_json(json!({
                "max_bet_ratio": {
                    "num": "10",
                    "den": "100"
                },
                "winning_proba": {
                    "num": "128",
                    "den": "256"
                }
            }))
            .gas(NearGas::from_tgas(10))
            .transact()
            .await?
            .into_result()
            .unwrap();

        let owner = worker.dev_create_account().await?;
        owner
            .call(contract.id(), "top_up")
            .deposit(NearToken::from_near(TOP_UP_AMOUNT))
            .transact()
            .await?
            .into_result()
            .unwrap();

        let player = worker.dev_create_account().await?;
        let result = player.call(contract.id(), "pool").view().await?;
        assert_eq!(
            result.json::<near_sdk::json_types::U128>().unwrap().0,
            TOP_UP_AMOUNT * NEAR
        );

        let result = player
            .call(contract.id(), "bet")
            .deposit(NearToken::from_near(2))
            .gas(near_gas::NearGas::from_tgas(14))
            .transact()
            .await?
            .into_result()
            .unwrap();

        debug_result(&result);

        Ok(())
    }
}
