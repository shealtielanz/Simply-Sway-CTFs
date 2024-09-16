# Challenge: Exploit The Liquidity Pool
Welcome to the Liquidity Pool Exploit Challenge! The code for this challenge is based on an example from the [Fuel Docs](https://docs.fuel.network/docs/fuels-rs/cookbook/deposit-and-withdraw/). Although it was designed as a basic example to demonstrate using the Rust SDK in tests, the code wasn’t written with security in mind. During exploration, I discovered a bug and created this challenge to help others learn how vulnerabilities in Sway can be identified and exploited.

In this challenge, you'll exploit a bug in the provided code, using VSCode. I'll guide you through each step!

## Prerequisites
Before we dive into the challenge, make sure you have the necessary tools installed:

Fuel Toolchain: You'll need Fuel to run a local node for testing since Sway allows writing tests in both Sway and Rust. This challenge focuses on Rust-based tests.
Rust: Ensure you have Rust installed as well.

1) Install the Fuel Toolchain

To install the Fuel toolchain, run the following command:

```bash
curl https://install.fuel.network | sh
```
For additional information on Fuel and its usage, refer to the official Fuel Installation Guide.

2)  Install Rust

Next, install Rust using the command below:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3) Set Up Your Project
   
Now that your environment is set up, it’s time to start the challenge.

**Create a new directory for your project:**

```bash
mkdir liquidity-pool-exploit
```
**Initialize the project using forc:**

```bash
forc init --path liquidity-pool-exploit
```
**Navigate into the project directory:**

```bash
cd liquidity-pool-exploit
```

4) Complete the Challenge
   
**To complete the challenge, follow these steps:**

- Copy the liquidity pool code (provided below) into the `main.sw` file of your project.
- Identify the bug in the code.
- Write a proof-of-concept (PoC) test in the `harness.rs` file that demonstrates the bug.
Submit your solution, proving that you've exploited the vulnerability.

**Here’s the liquidity pool contract code for you to exploit:**

```sway
contract;
/*
Exploit this contract bby ;)
*/
use std::{
    asset::{
        mint_to,
        transfer,
    },
    call_frames::{
        msg_asset_id,
    },
    constants::ZERO_B256,
    context::msg_amount,
};
 
abi LiquidityPool {
    #[payable]
    fn deposit(recipient: Identity);
    #[payable]
    fn withdraw(recipient: Identity);
}
 
const BASE_TOKEN: AssetId = AssetId::from(0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c);
 
impl LiquidityPool for Contract {
    #[payable]
    fn deposit(recipient: Identity) {
        assert(BASE_TOKEN == msg_asset_id());
        assert(0 < msg_amount());
 
        // Mint two times the amount.
        let amount_to_mint = msg_amount() * 2;
 
        // Mint some LP token based upon the amount of the base token.
        mint_to(recipient, ZERO_B256, amount_to_mint);
    }
 
    #[payable]
    fn withdraw(recipient: Identity) {
        assert(0 < msg_amount());
 
        // Amount to withdraw.
        let amount_to_transfer = msg_amount() / 2;
 
        // Transfer base token to recipient.
        transfer(recipient, BASE_TOKEN, amount_to_transfer);
    }
}
```



## Set Up Tests
I'll give a quick rundown on how to use the Rust SDK for testing.

1) Build the Liquidity Pool
First, build the liquidity pool contract:
``` bash
forc build
```

2) Install Cargo for Testing
   
Next, install Cargo, which is needed to run the tests:
```bash
cargo install cargo-generate --locked
```

3) Generate and Initialize Your Project
   
Now, generate and initialize your test project using the following command:
```bash
cargo generate --init fuellabs/sway templates/sway-test-rs --name liquidity-pool-exploit --force
```

4) Prepare Your Test Environment
   
After setting up, you can clear the harness.rs file and use it to launch your provider and wallets. Follow the steps outlined in the [Fuel Docs under the section](https://docs.fuel.network/docs/fuels-rs/cookbook/deposit-and-withdraw/) for testing pool deposits and withdrawals.

**Notes**
For reference, use the [Fuel Docs](https://docs.fuel.network/docs/fuels-rs/cookbook/deposit-and-withdraw/) to help you get started with writing the tests for the liquidity pool.

Good luck with the challenge! 🚀

