use crate::types::DataKey;
use crate::{
    ChargeExecutionResult, SubscriptionStatus, SubscriptionVault, SubscriptionVaultClient,
};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env};

const T0: u64 = 1_000_000;
const AMOUNT: i128 = 1_000_000;
const INTERVAL: u64 = 60;

#[test]
fn zero_expiration_ledger_does_not_block_charging() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = T0;
        ledger.sequence = 5;
    });

    let admin = Address::generate(&env);
    let contract_id = env.register(SubscriptionVault, ());
    let client = SubscriptionVaultClient::new(&env, &contract_id);
    let token_admin_address = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin_address);
    let token_admin = token::StellarAssetClient::new(&env, &token_id.address());

    client.init(
        &token_id.address(),
        &6,
        &admin,
        &AMOUNT,
        &(7 * 24 * 60 * 60),
    );

    let subscriber = Address::generate(&env);
    let merchant = Address::generate(&env);
    let subscription_id = client.create_subscription(
        &subscriber,
        &merchant,
        &AMOUNT,
        &INTERVAL,
        &false,
        &None::<i128>,
        &None::<u64>,
        &None::<u32>,
    );
    token_admin.mint(&subscriber, &(AMOUNT * 2));
    client.deposit_funds(
        &subscription_id,
        &(AMOUNT * 2),
        &None::<soroban_sdk::BytesN<32>>,
    );

    // Simulate a stored legacy/default value; the public setter rejects bounds
    // at or below the current ledger sequence.
    let mut subscription = client.get_subscription(&subscription_id);
    subscription.expires_at_ledger = Some(0);
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::Sub(subscription_id), &subscription);
    });

    env.ledger()
        .with_mut(|ledger| ledger.timestamp = T0 + INTERVAL);
    assert_eq!(
        client.try_charge_subscription(&subscription_id, &None::<soroban_sdk::BytesN<32>>),
        Ok(Ok(ChargeExecutionResult::Charged)),
    );

    let subscription = client.get_subscription(&subscription_id);
    assert_eq!(subscription.status, SubscriptionStatus::Active);
    assert_eq!(subscription.prepaid_balance, AMOUNT);
}
