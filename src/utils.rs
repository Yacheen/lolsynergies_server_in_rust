use futures::{stream, StreamExt, Future};
use dotenv::dotenv;
use std::{ env, error::Error, time::{SystemTime, Duration} };
use serde::{Deserialize, Serialize}; 
//db
use mongodb::{options::{ClientOptions, ResolverConfig}, bson::doc};
use bson::to_bson;
use chrono::{TimeZone, Utc};
//actix web
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
//stuff from main.rs
use crate::Matches;

pub fn parse_username(s: &mut String) -> String {
    s.trim_start().trim_end().to_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>()
}

pub async fn check_db() -> Result<Vec<Matches>, Box<dyn Error>> {
    dotenv().ok();
    let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    let connection_options = ClientOptions::parse_with_resolver_config(client_uri, ResolverConfig::cloudflare()).await?;
    let client = mongodb::Client::with_options(connection_options)?;

    let mut summoners_collection: mongodb::Cursor<Matches> = client.database("myFirstDatabase").collection("summoners").find(None, None).await?;

    let mut games: Vec<Matches> = Vec::new();

    while let Some(game) = summoners_collection.next().await {
        games.push(game?);
    }

    Ok(games)

}