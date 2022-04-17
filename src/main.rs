
//serde and env var stuff
use futures::{stream, StreamExt};
use dotenv::dotenv;
use std::{ error::Error, env, time::{SystemTime, Duration}};
use serde::{Deserialize, Serialize}; 
use reqwest::{Client, StatusCode};
use actix_cors::Cors;
//db
use mongodb::{bson::{to_bson, doc}, options::{ResolverConfig, ClientOptions}};
//actix web
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponseBuilder, HttpResponse, ResponseError, http};
//use other files
mod utils;
const DB_NAME: &str = "myFirstDatabase";
const COLL_NAME: &str = "summoners";

//request body for /api/synergies
#[derive(Deserialize, Debug)] pub struct SynergiesPostBody { username: String, platform_routing_value: String, regional_routing_value: String }
//structs for hitting riot_api
#[derive(Deserialize, Debug)] pub struct Summoner { puuid: String, name: String, profileIconId: i32, summonerLevel: u64, id: String }

#[derive(Deserialize, Serialize, Debug)] 
pub struct RankedEntry {
    queueType: String,
    tier: Option<String>,
    rank: Option<String>,
    leaguePoints: i32,
    wins: i32,
    losses: i32
}

#[derive(Deserialize, Debug)] pub struct MatchIds (String);
#[derive(Deserialize, Serialize, Debug)] pub struct Game { info: GameInfo }
#[derive(Deserialize, Serialize, Debug)] pub struct GameInfo { gameCreation: u64, participants: Vec<Participant> }
#[derive(Deserialize, Serialize, Debug)] pub struct Participant {summonerName: String, championName: String, win: bool, teamId: u8, puuid: String}
//format riot data to store in db into this struct:
#[derive(Deserialize, Serialize, Debug)] pub struct RawUserData { username: String, profileIconId: i32, summonerLevel: u64, puuid: String, amount_of_games: u8, last_updated: Duration, games: Vec<Game>, ranked_info: Vec<RankedEntry>}


//organized data struct for synergies
#[derive(Deserialize, Serialize, Debug)] pub struct SynergyMatches {username: String, profileIconId: i32, summonerLevel: u64, amount_of_games: u8, last_updated: Duration, games: Winrates, ranked_info: Vec<RankedEntry>}
 impl SynergyMatches {
  pub fn new(last_updated: Duration) -> SynergyMatches {
    let games = Winrates { your_team: Vec::new(), enemy_team: Vec::new() };
    SynergyMatches {
      username: String::new(),
      //set default league icon here when u can
      profileIconId: 0,
      summonerLevel: 0,
      amount_of_games: 0,
      last_updated,
      games,
      ranked_info: Vec::new()
    }
  }
}
#[derive(Serialize, Debug, Deserialize)] pub struct Winrates { your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo> }
 impl Winrates {
  fn new(your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo>) -> Winrates{
    Winrates {
      your_team,
      enemy_team
    }
  }
}
#[derive(Serialize, Debug, Deserialize)] pub struct ChampionsInfo { championName: String, wins: u8, losses: u8, teamId: u8 }
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
async fn synergies(client: web::Data<mongodb::Client>, synergiespostdata: web::Json<SynergiesPostBody>) -> Result<impl Responder, Box<dyn Error>> {
    //env vars/data initialization
    dotenv().ok();
    //check db
    let summoners_collection: mongodb::Collection<RawUserData> = client.database(DB_NAME).collection(COLL_NAME);
    let result = summoners_collection.find_one(doc! {"username": utils::parse_username(&synergiespostdata.0.username)} , None).await?;
    
    //if games are received by username, send to frontend, else, hit riot api for 75 games and send to frontend
    
    let res = match result {
        Some(raw_user_data_from_db) => {
            
            //organize raw_user_data_from_db into SynergyMatches before sending
            let organized_games = utils::organize_games_into_synergies(raw_user_data_from_db);
            println!("sending games to client from db: {}", organized_games.amount_of_games);
            Ok(web::Json(organized_games))
        },

        None => {
            //hit rito db for 75 games if its their first time
            if let Some(match_data) = utils::fetch_matches_from_riot_api(&synergiespostdata.0, 75).await {
             
                //if u can get matches, add them to db,
                let insert_result = summoners_collection.insert_one(&match_data, None).await?;
                println!("added person and their games to db: {:#?}", insert_result);

                //organize them, then send to frontend
                println!("sending games to client from riot api: {}", match_data.amount_of_games);
                let organized_games = utils::organize_games_into_synergies(match_data);
                Ok(web::Json(organized_games))
            }
            else {
                let username = synergiespostdata.0.username.clone();
                println!("no matches have been gotten from fetch");
                Ok(web::Json(SynergyMatches {
                  amount_of_games: 0,
                  //set default leagueoflegends iconid here when i can
                  profileIconId: 0,
                  summonerLevel: 0,
                  username,
                  last_updated: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?,
                  games: Winrates { your_team: Vec::new(), enemy_team: Vec::new() },
                  ranked_info: Vec::new()
                }))
            }
        } 
    };
    res
    
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
  dotenv().ok();

  //initialize env vars and db
  let mongodb_uri = env::var("MONGODB_URI").unwrap();
  let port = env::var("PORT").unwrap();
  let connection_options = ClientOptions::parse_with_resolver_config(mongodb_uri, ResolverConfig::cloudflare()).await.expect("Failed to create connection options with cloudfare...");
  let client = mongodb::Client::with_options(connection_options).expect("Failed to connect to db.");
  

    HttpServer::new(move || {
      //initialize cors
      let cors = Cors::default()
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::ACCESS_CONTROL_ALLOW_ORIGIN])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
          
      App::new()
          .app_data(web::Data::new(client.clone()))    
          .wrap(cors)
          .service(synergies)
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run()
    .await

}
