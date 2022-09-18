use tokio::time::Instant;
mod auction_house;

#[tokio::main]
async fn main()
{

    let runtime= Instant::now();

    auction_house::AuctionHandler::new(60).collect_auctions(|_auction_house_|
        async move
            {
                /**/
            }
        )
        .await
    ;

    println!("\n[MRG] Runtime: {:#?}\n", runtime.elapsed());
}