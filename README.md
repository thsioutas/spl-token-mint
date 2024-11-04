# spl-token-mint
This program is a basic Solana contract to mint tokens, written in Rust without the Anchor framework. It includes instructions for building, deploying, and testing on the Solana localnet or devnet.

## Prerequisites

1. [Rust](https://www.rust-lang.org/) (Install Rust using [rustup](https://rustup.rs/))
2. [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (Install using `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`)

## Set Up

### 1. Set Solana CLI to Use localnet or devnet
#### For localnet:
Start a local validator
```bash
solana-test-validator
```

Once the validator is running, configure the Solana CLI to connect to it:
```bash
solana config set --url http://localhost:8899
```

Add default signer:
```bash
solana-keygen new -o ~/.config/solana/id.json
```

#### For devnet:
Set your environment to use Solana's devnet
```bash
solana config set --url https://api.devnet.solana.com
```
If you don't already have a Solana wallet, create one using:
```bash
solana-keygen new --outfile ~/.config/solana/devnet.json
```

### 3. Airdrop SOL to Your Wallet
Fund your wallet with test SOL:
```bash
solana airdrop 2
```

Verify your balance:
```bash
solana balance
```

## Build and deploy program
### 1. Build program
```bash
cargo build-sbf
```
#### Resolving Rust Version Compatibility Issues

If you encounter an error like the following during `cargo build-sbf`:
```
error: package `solana-program v2.1.0` cannot be built because it requires rustc 1.79.0 or newer, while the currently active rustc version is 1.75.0-dev
Either upgrade to rustc 1.79.0 or newer, or use
cargo update solana-program@2.1.0 --precise ver
where `ver` is the latest version of `solana-program` supporting rustc 1.75.0-dev
```
You can 
Downgrade `solana-program` version**: Run the following command to use an older version of `solana-program` that’s compatible with your Rust version:
```bash
cargo update solana-program@2.1.0 --precise 2.0.14
```

### 2. Deploy your program
```bash
solana program deploy target/deploy/spl_token_mint.so
```

### Note:
In case the program was already deployed, close the previous program:
```bash
solana program close --bypass-warning <previous_program_pubkey>
```
remove previous deployed configuration:
```bash
rm -rf target/deploy/
```
and deploy:
```bash
solana program deploy target/deploy/spl_token_mint.so
```

## Test the program using the test client
### 1. Fund the payer account
Generate a keypair for the payer account:
```bash
solana-keygen new --outfile ~/.config/solana/payer-keypair.json
```
Fund payer's wallet with test SOL:
```bash
solana airdrop 2 --keypair ~/.config/solana/payer-keypair.json
```

### 2. Build and run test client
Build test client
```bash
cargo build
```
Run test client
``` bash
./target/debug/spl-token-client
```

#### Note:
The `~/.config/solana/payer-keypair.json` should be given as argument to the client test application in order to use the necessary pubkey.

