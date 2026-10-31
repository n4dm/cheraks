use cheraks::{Blockchain, MAX_SUPPLY_CHK};

fn main() {
    println!("Cheraks (CHK) — educational blockchain prototype");
    println!("Maximum intended supply: {MAX_SUPPLY_CHK} CHK");
    println!("WARNING: prototype only. Coins have no monetary value.");
    println!("This is not a secure cryptocurrency or a public network yet.\n");

    let mut chain = match Blockchain::try_new(2) {
        Ok(chain) => chain,
        Err(error) => {
            eprintln!("Failed to initialize Cheraks: {error}");
            std::process::exit(1);
        }
    };
    println!("Created genesis block: {}", chain.latest_block().hash());

    match chain.mine_block("First locally mined demonstration block") {
        Some(block) => {
            println!("Mined block #{}: {}", block.height(), block.hash());
            println!("Previous hash: {}", block.previous_hash());
            println!("Nonce: {}", block.nonce());
        }
        None => println!("Supply cap reached."),
    }

    println!("Chain valid: {}", chain.is_valid());
    println!("Blocks in local chain: {}", chain.block_count());
    println!("\nNext milestone: add real transaction validation and multi-node networking.");
}
