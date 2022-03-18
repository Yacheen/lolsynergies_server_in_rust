
//serde and env var stuff
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;
use serde::Deserialize; 
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
struct SummonerYouPlayedWithInfo { summonerName: String, champions: ChampionsInfo }
impl SummonerYouPlayedWithInfo {
  fn new(summonerName: String, champions: ChampionsInfo) -> SummonerYouPlayedWithInfo{
    SummonerYouPlayedWithInfo {
      summonerName,
      champions
    }
  }
}

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
async fn echo(synergiespostdata: web::Json<SynergiesPostBody>) -> impl Responder {
  //i guess do redis stuff here regarding timeouts when u can refresh data
  
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();

  
  //get puuid
  let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.username, api_key);
  let summoner =  reqwest::get(url).await.unwrap().json::<Summoner>().await.unwrap();

//get match ids by puuid
  let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}", summoner.puuid, api_key);
  let match_ids = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await.unwrap();

  //change this to a special cookie with cookiebuilder
  let cookie = Cookie::new("username", &synergiespostdata.0.username);

  //foreach match_id, request
  //let mut match_data: Vec<SummonerYouPlayedWithInfo> = Vec::new();
  for (index, match_id) in match_ids.iter().enumerate() {
    if index == 2 {break}
    let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
    let game = reqwest::get(match_url).await.unwrap().json::<Game>().await.unwrap();
    
    //initizalize SummonerYouPlayedWith if they're not already in summoneryouplayedwith[]
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
    //println!("{:#?}", game.info.participants.get(0..).unwrap());
  }

  
  let res = HttpResponseBuilder::new(StatusCode::OK)
      .cookie(cookie)
      .body(synergiespostdata.0.username);
  return res
}


#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let match_data: Vec<SummonerYouPlayedWithInfo> = Vec::new();
    let mut data = web::Data::new(Mutex::new(match_data));


    HttpServer::new(|| {
      App::new()
          .app_data(data.clone())
          .service(hello)
          .service(echo)
          .route("/hey",web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}
