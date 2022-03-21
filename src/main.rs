
//serde and env var stuff
use std::thread;
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize}; 
use reqwest;


//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};


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
  summoners_you_played_with: Vec<SummonerYouPlayedWithInfo> // <- Mutex is necessary to mutate safely across threads
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

  // <- get list's MutexGuard
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();

  
  //get puuid
  let url = format!("https://na1.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.username, api_key);
  let summoner =  reqwest::get(url).await.unwrap().json::<Summoner>().await.unwrap();

//get match ids by puuid
  let matches_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count=31", summoner.puuid, api_key);
  let match_ids = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await.unwrap();
  //change this to a special cookie with cookiebuilder
  //let cookie = Cookie::new("username", &synergiespostdata.0.username);

  //foreach match_id, request
  let mut match_data: Vec<SummonerYouPlayedWithInfo> = Vec::new();
  let mut game_urls = Vec::new();
  for item in match_ids.iter() {
    game_urls.push(format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", item.0, api_key));
  } 
  let games_data: Vec<Game> = Vec::new();
  let requests_batch_1 = tokio::spawn(async move {
    for elem in game_urls.get(0..4).unwrap().iter() {

    }
  });
  let requests_batch_2 = tokio::spawn(async move {
    for elem in game_urls.get(4..8).unwrap().iter() {
      
    }
  });
  let requests_batch_1 = tokio::spawn(async move {
    for elem in game_urls.get(8..12).unwrap().into_iter() {
      
    }
  });
  let requests_batch_1 = tokio::spawn(async move {
    for elem in game_urls.get(12..16).unwrap().into_iter() {
      
    }
  });
  let requests_batch_5 = tokio::spawn(async move {
    for elem in game_urls.get(16..20).unwrap().into_iter(){
      
    }
  });
  for (i, url) in game_urls.into_iter().enumerate() {
    if i % 2 == 0 {
      let game_request = tokio::spawn( async move {
        reqwest::get(url).await.unwrap().json::<Game>().await.unwrap()
      });
      println!("{:#?}", game_request);
      
      
    }
    
  }
  


  for (index, match_id) in match_ids.iter().enumerate() {
    if index == 20 {break}
    let match_url = format!("https://americas.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", match_id.0, api_key);
    let game = reqwest::get(match_url).await.unwrap().json::<Game>().await.unwrap();
    
    //go through each person in a game
    for person in game.info.participants.iter() {
      //filter through current list of summoners
      //,   if found, go throguh their champions,
      //      if found, add a win or loss,
      //    otherwise add a new champ,
      //otherwise add a new summoner
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
