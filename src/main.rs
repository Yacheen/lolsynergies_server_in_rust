
//serde and env var stuff
use futures::{stream, StreamExt};
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize}; 
use reqwest::Client;
use actix_cors::Cors;
//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
//use other files
mod utils;

#[derive(Deserialize)]
struct SynergiesPostBody {
  username: String,
  platform_routing_value: String,
  regional_routing_value: String
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
#[derive(Deserialize)] struct Participant {summonerName: String, championName: String, win: bool, teamId: u8, puuid: String}

//CHANGE THIS 
#[derive(Deserialize, Serialize)] struct Matches {amount_of_games: u8, games: Winrates}
impl Matches {
  fn new() -> Matches {
    let games = Winrates { your_team: Vec::new(), enemy_team: Vec::new() };
    Matches {
      amount_of_games: 0,
      games
    }
  }
}
//then put data into array of SummonersYouPLayedWith


#[derive(Serialize, Debug, Deserialize)]
struct Winrates { your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo> }
impl Winrates {
  fn new(your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo>) -> Winrates{
    Winrates {
      your_team,
      enemy_team
    }
  }
}
#[derive(Serialize, Debug, Deserialize)]
struct ChampionsInfo { championName: String, wins: u8, losses: u8, teamId: u8 }
impl ChampionsInfo {
  fn new(championName: String, wins: u8, losses: u8, teamId: u8) -> ChampionsInfo {
    ChampionsInfo {
      championName,
      wins,
      losses,
      teamId
    }
  }
}


#[get("/")]
async fn hello() -> impl Responder {
  HttpResponse::Ok().body("Hewwo wowld!")
}

#[post("/api/synergies")]
async fn synergies(mut synergiespostdata: web::Json<SynergiesPostBody>) -> impl Responder {
  dotenv().ok();
  let api_key = env::var("API_KEY").unwrap();
  //get puuid
  let url = format!("https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", &synergiespostdata.0.platform_routing_value, &synergiespostdata.0.username, api_key);
  if let Ok(summoner) =  reqwest::get(url).await.unwrap().json::<Summoner>().await {
    //make 3 simultaneous requests here for ranked 5v5, normal draft 5v5, and normal blind 5v5

    //get 5v5 ranke matches
    let queue: i16 = 420;
    let matches_url = format!("https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count=75&queue={}",synergiespostdata.0.regional_routing_value, summoner.puuid, api_key, queue);
    //get 5v5 draft matches
    //get 5v5 blind matches

    if let Ok(match_ids) = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await {
       //push game urls to a vec
      let mut match_data = Matches::new();
      let mut game_urls = Vec::new();

      for item in match_ids.iter() {
        game_urls.push(format!("https://{}.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", synergiespostdata.0.regional_routing_value, item.0, api_key));
      } 
      //with a max of 4 concurrent requests,
      const CONCURRENT_REQUESTS: usize = 4;
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
  
        //while there are still games left to iterate through,
        //for each game, get user_teamId
        //go through each person, if their id is the same, add the champ
        //they played to your team, otherwise add to enemy team
        while let Some(game) = games.next().await {
          match game {
            Ok(game) =>  {
              match_data.amount_of_games += 1;
              
              // get users team_id
              let mut user_team_id: u8 = 0;
              for person in game.info.participants.iter() {
                if person.puuid == summoner.puuid {
                  user_team_id = person.teamId;
                }
              }

              for person in game.info.participants.iter() {
                //if person is on your team, add to your_team, otherwise add to enemy_team
                let mut username = person.summonerName.clone();
                if utils::parse_username(&mut username) == utils::parse_username(&mut synergiespostdata.0.username) {
                  continue;
                }
                else {
                  if user_team_id == person.teamId {
                    //find a champ, if it destructures into a champ, add a win or loss, otherwise push a new champ
                    if let Some(champ) = match_data.games.your_team.iter_mut().find(|champ| champ.championName == person.championName) {
                      if let true = person.win { champ.wins += 1; } else { champ.losses += 1; }
                    }
                    else {
                      match_data.games.your_team.push(ChampionsInfo::new(
                        person.championName.to_string(),
                        if person.win == true {1} else {0},
                        if person.win == true {0} else {1},
                        person.teamId
                      ))
                    }
                  }
                  else {
                    //go through enemy team for whether to psuh a new champ or edit one
                    if let Some(champ) = match_data.games.enemy_team.iter_mut().find(|champ| champ.championName == person.championName) {
                      if let true = person.win { champ.wins += 1; } else { champ.losses += 1; }
                    }
                    else {
                      match_data.games.enemy_team.push(ChampionsInfo::new(
                        person.championName.to_string(),
                        if person.win == true {1} else {0},
                        if person.win == true {0} else {1},
                        person.teamId
                      ));
                    }
                  }

                }
            }
          },
            Err(_) => break
          }
        }
        println!("games sent to client: {}", match_data.amount_of_games);
        web::Json(match_data)

    } else {
      let no_games_found = Matches::new();
      web::Json(no_games_found) 
    }

  } else {
    let no_games_found = Matches::new();
    web::Json(no_games_found) 
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(move || {
      let cors = Cors::permissive();
          
      App::new()
          .wrap(cors)
          .service(hello)
          .service(synergies)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}
