
//serde and env var stuff
use futures::{stream::{self, FusedStream}, StreamExt, join, TryStreamExt, Stream, stream::select_all, AsyncReadExt, future};
use dotenv::dotenv;
use std::{env, ops::Add, slice::SliceIndex, error::Error};
use serde::{Deserialize, Serialize}; 
use reqwest::Client;


//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, http::header::TryIntoHeaderValue};


#[derive(Deserialize)]
struct SynergiesPostBody {
  username: String
}
#[derive(Deserialize)] struct Summoner { puuid: String }
#[derive(Deserialize, Debug)] struct MatchIds (String);
//begin pepega json deserialization:
//structs for sequence of requesting someones info
#[derive(Debug)]
#[derive(Deserialize)] struct Game { info: GameInfo }
#[derive(Debug)]
#[derive(Deserialize)] struct GameInfo {participants: Vec<Participant>}
#[derive(Debug)]
#[derive(Deserialize)] struct Participant {championName: String, summonerName: String, win: bool}

//then put data into array of SummonersYouPLayedWith
#[derive(Serialize, Debug)]
struct AppData {
  summoners_you_played_with: Vec<SummonerYouPlayedWithInfo>
}
#[derive(Serialize, Debug)]
struct SummonerYouPlayedWithInfo { summonerName: String, champions: Vec<ChampionsInfo> }
impl SummonerYouPlayedWithInfo {
  fn new(summonerName: String, champions: Vec<ChampionsInfo>) -> SummonerYouPlayedWithInfo {
    SummonerYouPlayedWithInfo {
      summonerName,
      champions
    }
  }
}
#[derive(Serialize, Debug)]
struct ChampionsInfo { championName: String, wins: u8, losses: u8 }
impl ChampionsInfo {
  fn new(championName: String, wins: u8, losses: u8) -> ChampionsInfo {
    ChampionsInfo {
      championName,
      wins,
      losses
    }
  }
}


#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok().body("Hewwo wowld!")
}

#[post("/api/synergies")]
async fn synergies(synergiespostdata: web::Json<SynergiesPostBody>) -> impl Responder {
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();

  //get puuid
  let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.username, api_key);
  let summoner =  reqwest::get(url).await.unwrap().json::<Summoner>().await.unwrap();

  //get match ids by puuid
  let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count=50", summoner.puuid, api_key);
  let match_ids = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await.unwrap();

  //push game urls to a vec
  let mut match_data: Vec<SummonerYouPlayedWithInfo> = Vec::new();
  let mut game_urls = Vec::new();
  for item in match_ids.iter() {
    game_urls.push(format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", item.0, api_key));
  } 

  const CONCURRENT_REQUESTS: usize = 2;
  let client = Client::new();
  
     let mut games = stream::iter(game_urls)
      .map(|url| {
        let client = &client;
        async move {
          let resp = client.get(url).send().await?;
          resp.json::<Game>().await
        }
      })
      .buffer_unordered(CONCURRENT_REQUESTS);

      while let Some(game) = games.next().await {
        match game {
          Ok(game) =>  {
            for person in game.info.participants.iter() {
            if let Some(summ) = match_data.iter_mut().find(|summ| summ.summonerName == person.summonerName) {
              if let Some(champ) = summ.champions.iter_mut().find(|champ| champ.championName == person.championName) {
                if let true = person.win {champ.wins += 1} else {champ.losses += 1}
                } else {
                summ.champions.push(ChampionsInfo::new(
                  person.championName.to_string(),
                  if person.win == true {1} else {0},
                  if person.win == true {0} else {1}
                ));
                }
            } else {
              //a champ needs to be added as a vector initially. I probably did this in the worst way possible.
              let mut champ = Vec::new();
              champ.push(ChampionsInfo::new(
                person.championName.to_string(),
                if person.win == true {1} else {0},
                if person.win == true {0} else {1}
              ));
              match_data.push(SummonerYouPlayedWithInfo::new(person.summonerName.clone(),champ));
            }

          }
        },
          Err(_) => break
        }
        
      }
    
      
  web::Json(match_data)
  
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {


    HttpServer::new(move || {
      App::new()
          .service(hello)
          .service(synergies)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}
