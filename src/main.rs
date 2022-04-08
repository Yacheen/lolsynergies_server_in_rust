
//serde and env var stuff
use futures::{stream, StreamExt};
use dotenv::dotenv;
use std::{ env, error::Error, time::{SystemTime, Duration}};
use serde::{Deserialize, Serialize}; 
use reqwest::Client;
use actix_cors::Cors;
//db
use mongodb::{bson::{to_bson, doc}, options::{ResolverConfig, ClientOptions}};
//actix web
use actix_web::{get, post, web, App, HttpServer, Responder};
//use other files
mod utils;
const DB_NAME: &str = "myFirstDatabase";
const COLL_NAME: &str = "summoners";

#[derive(Deserialize)]
pub struct SynergiesPostBody {
  username: String,
  platform_routing_value: String,
  regional_routing_value: String
}
#[derive(Deserialize)] pub struct Summoner { puuid: String }
#[derive(Deserialize, Debug)] pub struct MatchIds (String);
//begin pepega json deserialization:
//structs for sequence of requesting someones info
#[derive(Debug)]
#[derive(Deserialize)] pub struct Game { info: GameInfo }
#[derive(Debug)]
#[derive(Deserialize)] pub struct GameInfo { gameCreation: u64, participants: Vec<Participant> }
#[derive(Debug)]
#[derive(Deserialize)] pub struct Participant {summonerName: String, championName: String, win: bool, teamId: u8, puuid: String}

//CHANGE THIS 
#[derive(Deserialize, Serialize, Debug)] pub struct Matches {username: String, amount_of_games: u8, last_updated: Duration, games: Winrates}
 impl Matches {
  pub fn new() -> Matches {
    let games = Winrates { your_team: Vec::new(), enemy_team: Vec::new() };
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    Matches {
      username: String::new(),
      amount_of_games: 0,
      last_updated: now,
      games
    }
  }
}
//then put data into array of SummonersYouPLayedWith


#[derive(Serialize, Debug, Deserialize)]
pub struct Winrates { your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo> }
 impl Winrates {
  fn new(your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo>) -> Winrates{
    Winrates {
      your_team,
      enemy_team
    }
  }
}
#[derive(Serialize, Debug, Deserialize)]
pub struct ChampionsInfo { championName: String, wins: u8, losses: u8, teamId: u8 }
 impl ChampionsInfo {
  pub fn new(championName: String, wins: u8, losses: u8, teamId: u8) -> ChampionsInfo {
    ChampionsInfo {
      championName,
      wins,
      losses,
      teamId
    }
  }
}

#[post("/api/synergies")]
async fn synergies(client: web::Data<mongodb::Client>, mut synergiespostdata: web::Json<SynergiesPostBody>) -> Result<impl Responder, Box<dyn Error>> {
  //env vars/data initialization
  dotenv().ok();
  let api_key = env::var("API_KEY")?;
  let summoners_collection: mongodb::Collection<Matches> = client.database(DB_NAME).collection(COLL_NAME);
  let result = summoners_collection.find_one(doc! {"username": &synergiespostdata.0.username} , None).await?;

  //if games are received by username, send to frontend, else, hit riot api for 75 games and send to frontend
  match result {
    Some(summoners_synergies) => Ok(web::Json(summoners_synergies)),
    None => {
      //hit rito db for 75 games if its their first time
      //get puuid
     
            
            let insert_result = summoners_collection.insert_one(&match_data, None).await?;
            println!("added person and their games to db: {:#?}", insert_result);
            println!("games sent to client: {}", match_data.amount_of_games);
            Ok(web::Json(match_data))

        } else {
          let no_games_found = Matches::new();
          Ok(web::Json(no_games_found))
        }

      } else {
        let no_games_found = Matches::new();
        Ok(web::Json(no_games_found)) 
      }

    } 
  }
  
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();

  //initialize database 
  let mongodb_uri = env::var("MONGODB_URI").unwrap();
  let connection_options = ClientOptions::parse_with_resolver_config(mongodb_uri, ResolverConfig::cloudflare()).await.expect("Failed to create connection options with cloudfare...");
  let client = mongodb::Client::with_options(connection_options).expect("Failed to connect to db.");


    HttpServer::new(move || {
      //initialize cors
      let cors = Cors::permissive();
          
      App::new()
          .app_data(web::Data::new(client.clone()))    
          .wrap(cors)
          .service(synergies)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await

}
