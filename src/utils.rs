use futures::{stream, StreamExt, Stream};
use crate::{Summoner, MatchIds, Game, GameInfo, Participant, SynergiesPostBody, RawUserData};
use dotenv::dotenv;
use std::{env, time::{SystemTime, Duration}};
use reqwest::Client;
use actix_web::web;

pub fn parse_username(s: &mut String) -> String {
    s.trim_start().trim_end().to_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>()
}

pub async fn fetch_matches_from_riot_api(synergiespostdata: &web::Json<SynergiesPostBody>, count: u8) -> Option<RawUserData> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    let username = synergiespostdata.0.username.clone();
    let mut match_data = RawUserData {
        amount_of_games: 0,
        games: Vec::new(),
        username,
        last_updated: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
    };

    let url = format!("https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", synergiespostdata.platform_routing_value, synergiespostdata.username, api_key);
    if let Ok(summoner) =  reqwest::get(url).await.unwrap().json::<Summoner>().await {
        //make 3 simultaneous requests here for ranked 5v5, normal draft 5v5, and normal blind 5v5
            
        //get 5v5 ranke matches
        let queue: i16 = 420;
        let matches_url = format!("https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count={}&queue={}",
            synergiespostdata.username,
            summoner.puuid,
            api_key,
            count,
            queue
        );
        //get 5v5 draft matches
        //get 5v5 blind matches

        if let Ok(match_ids) = reqwest::get(matches_url).await.unwrap().json::<Vec<MatchIds>>().await {
            //push game urls to a vec
            let mut game_urls = Vec::new();
            for item in match_ids.iter() {
                game_urls.push(format!("https://{}.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", synergiespostdata.regional_routing_value, item.0, api_key));
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
        
            while let Some(game) = games.next().await {
                match game {
                    Ok(game) =>  {
                    match_data.amount_of_games += 1;
                    match_data.games.push(game);
                    },
                    Err(_) => break
                }
            }

        }
        else {
            return None;
        }
    }
    else {
        return None;
    }
    return Some(match_data);
}

pub fn organize_games(matches: &mut SynergyMatches) -> &mut SynergyMatches {
    for (item, index) in matches.games.your_team.iter().enumerate() {
        println!("hi");
    }

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
                match_data.games.your_team.push(ChampionsInfo::new(
                    person.championName.to_string(),
                    if person.win == true {1} else {0},
                    if person.win == true {0} else {1},
                    person.teamId
                ))
            }
            else {
            //go through enemy team for whether to psuh a new champ or edit one
                match_data.games.enemy_team.push(ChampionsInfo::new(
                    person.championName.to_string(),
                    if person.win == true {1} else {0},
                    if person.win == true {0} else {1},
                    person.teamId
                ));
            }
        }
    }
    matches
}