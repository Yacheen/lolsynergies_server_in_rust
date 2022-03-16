use dotenv::dotenv;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io;
use std::iter::Map;
use serde::Deserialize;
use reqwest;

#[derive(Deserialize, Debug)]
struct Summoner {
  puuid: String, 
}

#[derive(Deserialize, Debug, Serialize)]
struct Game {
  metadata: HashMap<String, Vec<String>>,
  info: HashMap<String, Vec<String>>,
  dataVersion: String
}

#[derive(Deserialize, Debug, Serialize)]
struct MatchIds (String);
//structs for sequence of requesting someones info
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
    //let summoner = 
    let summoner =  reqwest::get(url).await?.json::<Summoner>().await?;
    println!("{:#?}", summoner);

    let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}", summoner.puuid, api_key);
    //get match ids by puuid
    //declare vec that derives deserialize & debug struct

    let match_ids = reqwest::get(matches_url).await?.json::<Vec<MatchIds>>().await?;
    for match_id in match_ids.iter() {
      //game.0 is from a struct tuple, so uhh game.0 i guess..
      println!("{:#?}", match_id.0);
      let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
      let game = reqwest::get(match_url).await?.json::<Game>().await?;
      println!("{:?}", game);
    }
    //println!("{:#?}", match_ids);
    
    // for item in match_ids {
    //     println!("made it here: {:#?}", match_ids);   
    // }
    

  //  println!("{:?}", json);
    Ok(())

}
