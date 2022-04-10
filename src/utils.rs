use futures::{stream, StreamExt};
use crate::{Summoner, MatchIds, Game, SynergiesPostBody, RawUserData, SynergyMatches, ChampionsInfo};
use dotenv::dotenv;
use std::{env, time::SystemTime};
use reqwest::Client;

pub fn parse_username(s: &String) -> String {
    s.trim_start().trim_end().to_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>()
}

pub async fn fetch_matches_from_riot_api(synergiespostdata: &SynergiesPostBody, count: u8) -> Option<RawUserData> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();
    println!("{:#?}", synergiespostdata);
    let username = synergiespostdata.username.clone();
    //set puuid after you get it from summoner request
    let mut match_data = RawUserData {
        username,
        puuid: String::new(),
        amount_of_games: 0,
        games: Vec::new(),
        last_updated: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
    };

    let url = format!("https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", synergiespostdata.platform_routing_value, synergiespostdata.username, api_key);
    println!("{}", url);

    let client = Client::new();
    if let Ok(summoner) =  client.get(url).send().await.unwrap().json::<Summoner>().await {
        //set RawUserData's puuid
        println!("got user: {:#?}", summoner);
        match_data.puuid = summoner.puuid.clone();
        
        //get 5v5 ranke matches
        let queue: i16 = 420;
        let matches_url = format!("https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count={}&queue={}",
            synergiespostdata.regional_routing_value,
            summoner.puuid,
            api_key,
            count,
            queue
        );
 
        if let Ok(match_ids) = client.get(matches_url).send().await.unwrap().json::<Vec<MatchIds>>().await {
            println!("{:#?}", match_ids);
            //push game urls to a vec
            let mut game_urls = Vec::new();
            for item in match_ids.iter() {
                game_urls.push(format!("https://{}.api.riotgames.com/lol/match/v5/matches/{}?api_key={}", synergiespostdata.regional_routing_value, item.0, api_key));
            } 
            //with a max of 4 concurrent requests,
            const CONCURRENT_REQUESTS: usize = 4;
            
        
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

//organizes matches for /summoners/[region]/[usrename] on frontend
pub fn organize_games_into_synergies(raw_data: &RawUserData) -> SynergyMatches {
    //initialize synergymatches
    let mut organized_games = SynergyMatches::new();
    organized_games.amount_of_games = raw_data.amount_of_games;
    organized_games.username = raw_data.username.clone();

    
    //iterate through raw data's games
    for games in raw_data.games.iter() {
        //determine what team the user is on for this game before algo begins
        let mut user_team_id: u8 = 0;
        for person in games.info.participants.iter() {
            if person.puuid ==  raw_data.puuid {
                user_team_id = person.teamId;
            }
        }

        //go through people in game
        for person in games.info.participants.iter() {
            //if its who ur searching, dont include in synergies, because ur trying to see who u synergize *with*, not you included
            
            if parse_username(&person.summonerName) == parse_username(&raw_data.username) {
                continue;
            }
            else {
                //if persons on your team, add to your team

                if user_team_id == person.teamId {
                     //find a champ, if it destructures into a champ, add a win or loss, otherwise push a new champ
                    if let Some(champ) = organized_games.games.your_team.iter_mut().find(|champ| champ.championName == person.championName) {
                        if person.win == true {champ.wins = champ.wins + 1;} else {champ.losses += 1;}
                    }
                    else {
                        organized_games.games.your_team.push(
                            ChampionsInfo {
                                championName: person.championName.clone(),
                                wins: if person.win == true {1} else {0},
                                losses: if person.win == true {0} else {1},
                                teamId: person.teamId 
                            }
                        )
                    }
                }
                //otherwise add to enemy team
                else {
                    if let Some(champ) = organized_games.games.enemy_team.iter_mut().find(|champ| champ.championName == person.championName) {
                        if person.win == true {champ.wins += 1;} else {champ.losses += 1;}
                    }
                    else {
                        organized_games.games.enemy_team.push(
                            ChampionsInfo {
                                championName: person.championName.clone(),
                                wins: if person.win == true {1} else {0},
                                losses: if person.win == true {0} else {1},
                                teamId: person.teamId 
                            }
                        )
                    }
                }
            }
        }
    }
    organized_games
}