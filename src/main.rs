use actix_web::{HttpResponseBuilder};
//serde and env var stuff
use dotenv::dotenv;
use std::env;
use serde::Deserialize; 
use reqwest;


//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, cookie::Cookie, http::StatusCode};


#[derive(Deserialize, Debug)]
struct SynergiesPostBody {
  username: String
}
#[derive(Deserialize, Debug)] struct Summoner { puuid: String }
#[derive(Deserialize)] struct MatchIds (String);
//begin pepega json deserialization:
#[derive(Deserialize)] struct Game { info: GameInfo }
#[derive(Deserialize)] struct GameInfo {participants: Vec<Participant>}
#[derive(Deserialize, Debug)] struct Participant {championName: String, summonerName: String, win: bool}
//structs for sequence of requesting someones info

#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok().body("Hewwo wowld!")
}

#[post("/api/synergies")]
async fn echo(synergiespostdata: web::Json<SynergiesPostBody>) -> impl Responder {
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();
  let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.username, api_key);
  //get puuid
  let summoner =  reqwest::get(url).await.unwrap().json::<Summoner>().await.unwrap();

  let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}", summoner.puuid, api_key);

  //get match ids by puuid
  let match_ids = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await.unwrap();

  //change this to a special cookie with cookiebuilder
  let cookie = Cookie::new("username", &synergiespostdata.0.username);

  for (index, match_id) in match_ids.iter().enumerate() {
    if index == 2 {break}
    let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
    let game = reqwest::get(match_url).await.unwrap().json::<Game>().await.unwrap();
    println!("{:#?}", game.info.participants.get(0..2).unwrap());
  }
  
  let res = HttpResponseBuilder::new(StatusCode::OK)
      .cookie(cookie)
      .body(synergiespostdata.0.username);
  return res
}

async fn manual_hello() -> impl Responder {
  HttpResponse::Ok().body("I manually did this smiley face :)")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    


    HttpServer::new(|| {
      App::new()
          .service(hello)
          .service(echo)
          .route("/hey",web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}
