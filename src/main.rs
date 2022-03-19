
//serde and env var stuff
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize}; 
use reqwest;


//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, cookie::Cookie, http::StatusCode, HttpResponseBuilder};


#[derive(Deserialize)]
struct SynergiesPostBody {
  username: String
}
#[derive(Deserialize)] struct Summoner { puuid: String }
#[derive(Deserialize)] struct MatchIds (String);
//begin pepega json deserialization:
//structs for sequence of requesting someones info
#[derive(Deserialize)] struct Game { info: GameInfo }
#[derive(Deserialize)] struct GameInfo {participants: Vec<Participant>}
#[derive(Deserialize)] struct Participant {championName: String, summonerName: String, win: bool}

//then put data into array of SummonersYouPLayedWith
#[derive(Serialize)]
struct AppData {
  summoners_you_played_with: Vec<SummonerYouPlayedWithInfo> // <- Mutex is necessary to mutate safely across threads
}
#[derive(Serialize)]
struct SummonerYouPlayedWithInfo { summonerName: String, champions: ChampionsInfo }
impl SummonerYouPlayedWithInfo {
  fn new(summonerName: String, champions: ChampionsInfo) -> SummonerYouPlayedWithInfo{
    SummonerYouPlayedWithInfo {
      summonerName,
      champions
    }
  }
}
#[derive(Serialize)]
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

  // <- get list's MutexGuard
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();

  
  //get puuid
  let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.username, api_key);
  let summoner =  reqwest::get(url).await.unwrap().json::<Summoner>().await.unwrap();

//get match ids by puuid
  let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count=25", summoner.puuid, api_key);
  let match_ids = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await.unwrap();

  //change this to a special cookie with cookiebuilder
  let cookie = Cookie::new("username", &synergiespostdata.0.username);

  //foreach match_id, request
  let mut match_data: Vec<SummonerYouPlayedWithInfo> = Vec::new();
  for (index, match_id) in match_ids.iter().enumerate() {
    if index == 25 {break}
    let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
    let game = reqwest::get(match_url).await.unwrap().json::<Game>().await.unwrap();
    
    //initizalize SummonerYouPlayedWith if they're not already in summoneryouplayedwith[]
    //gothrough match_data to find summoner

    let champions_info = ChampionsInfo::new(
    game.info.participants.get(index).unwrap().championName.to_string(),
    if game.info.participants.get(index).unwrap().win == true {1} else {0},
    if game.info.participants.get(index).unwrap().win == true {0} else {1}
     );

    let summoner_you_played_with = SummonerYouPlayedWithInfo::new(
      game.info.participants.get(index).unwrap().summonerName.clone(),
       champions_info
    );
    match_data.push(summoner_you_played_with); 

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
