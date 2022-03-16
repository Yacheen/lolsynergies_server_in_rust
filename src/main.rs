extern crate dotenv;
use serde::Deserialize;
use dotenv::dotenv;
use std::env;
use std::io;
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}


#[derive(Deserialize, Debug)]
struct Summoner {
    accountId: String,
    profileIconId: i32,
    revisionDate: u64,
    name: String,
    id: String,
    puuid: String,
    summonerLevel: u64,

}
#[derive(Deserialize, Debug)]
struct MatchIds {
    matchId: String
}

impl Iterator for MatchIds {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.matchId.len() == 0 {
            None
        }
        else {
            Some(self.matchId)
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //set env vars
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    // take input for querying api
    let mut username = String::new();
    println!("Please enter a username to find...");

    io::stdin().read_line(&mut username)?;

    
    let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", username, api_key);
    //get puuid
    let summoner = reqwest::get(url).await?.json::<Summoner>().await?;

    let matches_url = format!("https://na1.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}", summoner.puuid, api_key);
    //get match ids by puuid
    let mut match_ids = reqwest::get(matches_url).await?.json::<MatchIds>().await?.next().filter(|game| game.len() > 0);

    println!("{:#?}", match_ids);

  //  println!("{:?}", json);
    Ok(())

}