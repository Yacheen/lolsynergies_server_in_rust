use dotenv::dotenv;
use std::{ error::Error, env, time::SystemTime, sync::Mutex};
use actix_cors::Cors;
//db
use mongodb::{bson::doc, options::{ResolverConfig, ClientOptions}};
//actix web
use actix_web::{post, web::{self, Data}, App, HttpServer, Responder, dev::Server};
//use other files
mod functions;
mod definitions;
const DB_NAME: &str = "myFirstDatabase";
const COLL_NAME: &str = "summoners";

struct ServerCurrentStates {
    ran_out_of_requests: bool,
}

//request body for /api/synergies
#[post("/api/synergies")]
async fn synergies(client: Data<mongodb::Client>, server_current_states: Data<Mutex<ServerCurrentStates>>, synergiespostdata: web::Json<definitions::SynergiesPostBody>) -> Result<impl Responder, Box<dyn Error>> {
    //env vars/data initialization
    dotenv().ok();
    //check db
    
    let summoners_collection: mongodb::Collection<definitions::RawUserData> = client.database(DB_NAME).collection(COLL_NAME);
    println!("Username parsed: {}", functions::parse_username(&synergiespostdata.0.username));
    let result = summoners_collection.find_one(doc! {"username": functions::parse_username(&synergiespostdata.0.username)} , None).await.unwrap();
    //if games are received by username, send to frontend, else, hit riot api for 75 games and send to frontend
    let res = match result {
        Some(raw_user_data_from_db) => {
            
            //organize raw_user_data_from_db into SynergyMatches before sending
            let organized_games = functions::organize_games_into_synergies(raw_user_data_from_db);
            println!("sending games to client from db: {}", organized_games.amount_of_games);
            Ok(web::Json(organized_games))
        },

        None => {
            //hit rito db for 75 games if its their first time
            if let Some(match_data) = functions::fetch_matches_from_riot_api(&synergiespostdata.0, 75).await {
                // assure its at least 50 games, otherwise, dont input into db and spit error msg 
                //if u can get matches, add them to db,
                let insert_result = summoners_collection.insert_one(&match_data, None).await?;
                println!("added person and their games to db: {:#?}", insert_result);

                //organize them, then send to frontend
                println!("sending games to client from riot api: {}", match_data.amount_of_games);
                let organized_games = functions::organize_games_into_synergies(match_data);
                Ok(web::Json(organized_games))
            }
            else {
                let username = synergiespostdata.0.username.clone();
                println!("no matches have been gotten from fetch for user: {}", username);
                Ok(web::Json(definitions::SynergyMatches {
                  amount_of_games: 0,
                  display_name: None,
                  //set default leagueoflegends iconid here when i can
                  profileIconId: 0,
                  summonerLevel: 0,
                  username,
                  last_updated: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?,
                  games: definitions::Winrates { your_team: Vec::new(), enemy_team: Vec::new() },
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
    let connection_options = ClientOptions::parse_with_resolver_config(mongodb_uri, ResolverConfig::cloudflare()).await.expect("Failed to create connection options with cloudfare...");
    let client = Data::new(mongodb::Client::with_options(connection_options).expect("Failed to connect to db."));
    // api-request-preventing-variable - if set to true, prevents requests to riot api 
    let server_state_data = Data::new(Mutex::new( ServerCurrentStates { ran_out_of_requests: false, }));

    HttpServer::new(move || {
      //initialize cors
      let cors = Cors::permissive();
          
      App::new()
          .wrap(cors)
          .app_data(Data::clone(&server_state_data))
          .app_data(Data::clone(&client))    
          .service(synergies)
      })
      .bind(("127.0.0.1", 8080))?
      .run()
      .await?;

    Ok(())

}
