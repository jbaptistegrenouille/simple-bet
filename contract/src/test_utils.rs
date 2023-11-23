use near_workspaces::result::{ExecutionOutcome, ExecutionResult, Value};

/// Compile and use the latest binary for testing
#[ctor::ctor]
fn compile_code() {
    std::process::Command::new("cargo")
        .args(&[
            "build",
            "--target=wasm32-unknown-unknown",
            "--package=simple-bet",
            "--release",
        ])
        .status()
        .expect("Failed to compile contract");

    std::fs::create_dir_all("res").expect("Failed to create res directory");

    std::fs::copy(
        "target/wasm32-unknown-unknown/release/simple_bet.wasm",
        "res/simple_bet.wasm",
    )
    .expect("Failed to copy contract");
}

pub const NEAR: u128 = 1000000000000000000000000;

pub fn debug_result(result: &ExecutionResult<Value>) {
    for receipt in result.receipt_outcomes() {
        debug_receipt(receipt);
    }
}

pub fn debug_receipt(receipt: &ExecutionOutcome) {
    let gas_burnt = receipt.gas_burnt.as_gas() as f64 / 1e12;
    let receipt_id = receipt.transaction_hash.to_string()[..4].to_string();
    let executor_id = receipt.executor_id.clone();
    let receipts = receipt
        .receipt_ids
        .iter()
        .map(|id| id.to_string()[..4].to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let status = if receipt.is_success() {
        "SUCCESS"
    } else {
        "FAILURE"
    };

    println!(
        "<{} @ {}> ({:.3} Tgas) [{}] Receipts: [{}] ",
        receipt_id, executor_id, gas_burnt, status, receipts
    );

    for log in &receipt.logs {
        println!("  {}", log);
    }
}
