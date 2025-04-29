use tokio;

#[tokio::main]
async fn main() {
    let args = partymode::parse();
    
    partymode::run(args).await.unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}
