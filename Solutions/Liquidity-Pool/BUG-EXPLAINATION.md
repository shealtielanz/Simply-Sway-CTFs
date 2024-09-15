# Intro
Hey there! If you’ve made it this far, it means one of four things, the fourth one being the expected:

- You tried and couldn’t find the bug.
- You found the bug but couldn’t get the tests to work.
- You didn’t try at all (lol).
No matter where you are, I encourage you to go back and try again! Every failure is just another way that something doesn’t work, and with persistence, you’ll either find the bug or successfully write the exploit.

However, if you're here to confirm that the bug you found is valid, then congratulations! 🎉 Writing tests to prove a bug exists is not an easy task. If this bug were live on the mainnet, you'd be able to exploit it, so let's dive in!


# Bug Description

To understand this bug, it helps to have a basic understanding of **Fuel assets**, the concept of **UTXOs**, and how assets are transferred between users and contracts.

In the liquidity pool, users can deposit a base asset, and upon deposit, the contract mints to the user twice the amount of the base asset as LP tokens. This logic is typical in many pools and seems straightforward. You can observe the logic in the contract right here.
```sway
    #[payable]
    fn deposit(recipient: Identity) {
        assert(BASE_TOKEN == msg_asset_id());
        assert(0 < msg_amount());
 
        // Mint two times the amount.
        let amount_to_mint = msg_amount() * 2;
 
        // Mint some LP token based upon the amount of the base token.
        mint_to(recipient, ZERO_B256, amount_to_mint);
    }
```

So, users deposit a specified asset and, in return, they get minted the LP token. Now, let’s review the constraints of this process:

- The user must deposit the base asset.
- The deposit amount must be greater than zero.
- The LP tokens minted to the user are 2x the base amount deposited.
  
**Withdraw Functionality**



Now, let’s move on to the second functionality of the contract: the withdraw function. This function allows users to retrieve their base asset by providing the LP token. Sounds cool, right?

But here’s something to keep in mind: in Fuel, assets (or tokens) are handled a bit differently. There’s no explicit `transfer()` function for tokens—assets are treated as native assets by the blockchain.

Let’s take a closer look at the `withdraw()` function and see how it works.
```sway
    #[payable]
    fn withdraw(recipient: Identity) {
        assert(0 < msg_amount());
 
        // Amount to withdraw.
        let amount_to_transfer = msg_amount() / 2;
 
        // Transfer base token to recipient.
        transfer(recipient, BASE_TOKEN, amount_to_transfer);
    }
```

**Withdraw Function**
Users are expected to send an LP token, and the amount of the base asset they receive is a fraction of the LP token they sent via the withdraw() call. Let's break down the constraints:

- The user must send an amount greater than 0.
- The base token transferred to the user is the LP token amount divided by 2.
  
Now, are you sure your bug is correct? Or did you make a mistake? How could you miss this, right? lol

There’s no constraint on the `token/asset` sent via the call. That’s it. If you don’t see the issue yet, let me explain.

In the `withdraw()` function, the contract checks the amount it receives in the call but doesn’t validate the type of asset sent. This means anyone can send any token to the function and withdraw all the base assets from the pool.
**Here's how it works:**


- Imagine our classic "Bob" deposits **1,000,000 USDC** (assuming USDC is the base asset). The contract mints **2,000,000 LP tokens** for Bob.
- When Bob calls `withdraw()` with **2,000,000 LP tokens**, the contract transfers **1,000,000 USDC** back to him. Great! This is the expected behavior.



**Now, here’s the problem:**



- Alice notices the bug: there’s no validation on the `asset_id` sent with the `withdraw()` call.
- After Bob deposits **1,000,000 USDC**, Alice could mint herself **2,000,000** units of a completely useless token.
- She then calls `withdraw()` with that useless token instead of the actual **LP token** of the contract, and the contract will still give her **1,000,000 USDC**, effectively draining Bob's deposit from the contract.
- 
## Coded Exploit
For the coded exploit, we use Rust to simulate this scenario where a user deposits a certain amount of the base asset, and an attacker comes along and uses a useless token to withdraw all the base assets from the pool.

Check the `harness.rs` file where you’ll find detailed comments explaining how to write the exploit.
Use this command line to run the test and printout the logs.
```bash
cargo test -- --nocapture
```

After the test is done you can see the output that the exploit is possible.


```bash
running 1 test
Balance of user for the base-asset: 2000000
Balance of lptoken for the user before calling deposit: 0
Balance of lptoken after the call to deposit into pool: 2000000
Balance of the attacker  before  the call to exploit withdraw(): 2000000
Balance of the attacker  after the exploit: 3000000
test can_get_contract_id ... ok

```

To ensure it works, simply copy and paste the code into your VSCode. Be sure to:

- Specify the correct paths in your `Cargo.toml`.
- Install the appropriate libraries.

