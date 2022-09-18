use std::future::Future;
use tokio_retry::Retry;
use tokio_retry::strategy::ExponentialBackoff;
use tokio::time::Instant;

use serde::{ Deserialize };
use futures::{ stream, StreamExt };
use reqwest::{ Client };

const MAX_CONCURRENT_REQUESTS: usize = 550;
const MAX_REQUESTS_RETRIES: usize = 2;

pub struct AuctionHandler
{
    pub auction_base_url: String,
    pub total_pages: i64
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuctionHouse 
{
    pub page: i64,
    pub auctions: Vec<Auction>,
}

#[derive(Debug, Deserialize)]
pub struct Auction 
{
    pub uuid: String,
    pub item_name: String,
    pub tier: String,
    pub starting_bid: i64,
    pub item_bytes: String,
    pub claimed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<bool>,
}

impl AuctionHandler 
{
    pub fn new(total_pages: i64) -> Self 
    {
        Self 
        {
            auction_base_url: "https://api.hypixel.net/skyblock/auctions?page=".to_string(),
            total_pages
        }
    }

    pub async fn collect_auction (client: &Client, page: i64, auction_base_url: &str) -> Result<AuctionHouse, reqwest::Error>
    {
        Retry::spawn(ExponentialBackoff::from_millis(100).take(MAX_REQUESTS_RETRIES), ||
                async 
                    {
                        let http_runtime = Instant::now();
                        let http_response = client.get(format!("{}{}", auction_base_url, page))
                            .send()
                            .await?
                        ;

                        let json_runtime = Instant::now();
                        let auction: AuctionHouse = http_response.json().await?;

                        println!("[MRG] Process on page {} -> Request: {:#?}, Json: {:#?}, NBT: ?ms",
                                 auction.page,
                                 http_runtime.elapsed()-json_runtime.elapsed(),
                                 json_runtime.elapsed()
                            )
                        ;

                        Ok(auction)
                    }
                )
            .
        await
    }

    pub async fn collect_auctions <Fut: Future<Output = ()>> (&self, auction_house_future: impl FnMut(AuctionHouse) -> Fut)
    {
        let client = Client::new();

        stream::iter((0..self.total_pages).into_iter())
            .map(|page| 
                {
                    let client = client.clone();
                    let auction_base_url = self.auction_base_url.clone();
                    
                    tokio::spawn
                    (
                        async move 
                        {
                            Self::collect_auction(&client, page, &auction_base_url).await.unwrap()
                        }
                    )
                }
            )
            .buffer_unordered(MAX_CONCURRENT_REQUESTS)
            .filter_map(|response| 
                async 
                { 
                    response.ok()
                }
            )
            .for_each(auction_house_future)
            .await
        ;
    }
}