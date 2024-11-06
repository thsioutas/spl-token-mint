rm -rf target/deploy
cargo build-sbf
solana program deploy target/deploy/spl_token_mint.so
