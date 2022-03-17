//serde and env var stuff
use dotenv::dotenv;
use std::env;
use std::io;
use serde::Deserialize;
use reqwest;


//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, cookie::Cookie, HttpRequest, HttpMessage, http::StatusCode};


#[derive(Deserialize)] struct Summoner { puuid: String }
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
  let code = StatusCode::OK;
  let cookie = Cookie::new("authi", &req_body);
  println!("{}", cookie);
  match HttpResponse::add_cookie(&mut HttpResponse::new(code), &cookie) {
    Ok(val) => val,
    Err(_) => println!("err")
  };
  
  HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
  HttpResponse::Ok().body("I manually did this smiley face :)")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    //set env vars
    //dotenv().ok();
    //let api_key = env::var("API_KEY").unwrap();
    HttpServer::new(|| {
      App::new()
          .service(hello)
          .service(echo)
          .route("/hey",web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
   
    // // take input for querying api
    // let mut username = String::new();
    // println!("Please enter a username to find...");

    // io::stdin().read_line(&mut username)?;

    
    // let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", username, api_key);
    // //get puuid
    // let summoner =  reqwest::get(url).await?.json::<Summoner>().await?;

    // let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}", summoner.puuid, api_key);
    // //get match ids by puuid
    // let match_ids = reqwest::get(matches_url).await?.json::<Vec<MatchIds>>().await?;

    // //for each match id, get match participants name, champ, and if they won (max of 3 matches)
    // for (index, match_id) in match_ids.iter().enumerate() {
    //   if index == 2 {break}
    //   let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
    //   let game = reqwest::get(match_url).await?.json::<Game>().await?;
    //   println!("{:#?}", game.info.participants);
    // }

   // Ok(())
}
