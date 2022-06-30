use futures::{stream, StreamExt};
use crate::definitions::{Summoner, MatchIds, Game, SynergiesPostBody, RawUserData, SynergyMatches, ChampionsInfo, RankedEntry};
use dotenv::dotenv;
use std::{env, time::SystemTime};
use reqwest::Client;

pub fn parse_username(s: &String) -> String {
    s.trim_start().trim_end().to_lowercase().chars().filter(|c| !c.is_whitespace()).collect::<String>()
}

pub async fn fetch_matches_from_riot_api(synergiespostdata: &SynergiesPostBody, count: u8) -> Option<RawUserData> {
    dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    //set puuid after you get it from summoner request
    let mut match_data = RawUserData {
        username: String::new(),
        display_name: None,
        profileIconId: 0,
        summonerLevel: 0,
        puuid: String::new(),
        amount_of_games: 0,
        games: Vec::new(),
        last_updated: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(),
        ranked_info: Vec::new()
    };

    let url = format!("https://{}.api.riotgames.com/lol/summoner/v4/summoners/by-name/{}?api_key={}", synergiespostdata.platform_routing_value, synergiespostdata.username, api_key);

    let client = Client::new();
    if let Ok(summoner) =  client.get(url).send().await.unwrap().json::<Summoner>().await {
        //after getting summoner, use summoner to get league rank info
        let ranked_url = format!("https://{}.api.riotgames.com/lol/league/v4/entries/by-summoner/{}?api_key={}", synergiespostdata.platform_routing_value, summoner.id, api_key);
        if let Ok(user_ranked_info) = client.get(ranked_url).send().await.unwrap().json::<Vec<RankedEntry>>().await {
            
            //set user's general info
            match_data.puuid = summoner.puuid.clone();
            match_data.profileIconId = summoner.profileIconId;
            match_data.summonerLevel = summoner.summonerLevel;
            match_data.username = parse_username(&summoner.name);
            match_data.ranked_info = user_ranked_info;
            match_data.display_name = Some(summoner.name);
            
            //get 5v5 ranked matches
            let queue: i16 = 420;
            let matches_url = format!("https://{}.api.riotgames.com/lol/match/v5/matches/by-puuid/{}/ids?api_key={}&count={}&queue={}",
                synergiespostdata.regional_routing_value,
                summoner.puuid,
                api_key,
                count,
                queue
            );
    
            if let Ok(match_ids) = client.get(matches_url).send().await.unwrap().json::<Vec<MatchIds>>().await {
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
    }
    else {
        return None;
    }
    return Some(match_data);
}

//organizes matches for /summoners/[region]/[usrename] on frontend
pub fn organize_games_into_synergies(raw_data: RawUserData) -> SynergyMatches {
    //initialize synergymatches
    let mut organized_games = SynergyMatches::new(raw_data.last_updated);
    organized_games.amount_of_games = raw_data.amount_of_games;
    organized_games.username = raw_data.username.clone();
    organized_games.display_name = Some(raw_data.display_name.clone().unwrap());
    organized_games.profileIconId = raw_data.profileIconId.clone();
    organized_games.summonerLevel = raw_data.summonerLevel.clone();
    organized_games.username = raw_data.username.clone();
    organized_games.ranked_info =  raw_data.ranked_info;

    
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
            // checks if its u, if so, dont include in synergies list
            if parse_username(&person.puuid) == parse_username(&raw_data.puuid) {
                continue;
            }
            else {
                //if persons on your team, add to your team
                // get average of all scores  then put them as synergies
                if user_team_id == person.teamId {
                     //find a champ, if it destructures into a champ, add a win or loss, otherwise push a new champ
                    if let Some(champ) = organized_games.games.your_team.iter_mut().find(|champ| champ.championName == person.championName) {
                        // set new averages
                        champ.average_assists += person.assists;
                        champ.average_damage_dealt_to_objectives += person.damageDealtToObjectives;
                        champ.average_deaths += person.deaths;
                        champ.average_gold_earned += person.goldEarned;
                        champ.average_kills += person.kills;
                        champ.average_total_damage_dealt_to_champions += person.totalDamageDealtToChampions;
                        champ.average_total_damage_shielded_on_teammates += person.totalDamageShieldedOnTeammates;
                        champ.average_total_heals_on_teammates += person.totalHealsOnTeammates;
                        champ.average_total_minions_killed += person.totalMinionsKilled;
                        champ.average_neutral_minions_killed += person.neutralMinionsKilled;
                        champ.average_vision_score += person.visionScore; 
                        if person.win == true { champ.wins = champ.wins + 1; } else { champ.losses += 1; }
                        
                        
                    }
                    else {
                        organized_games.games.your_team.push(
                            ChampionsInfo {
                                championName: person.championName.clone(),
                                wins: if person.win == true {1} else {0},
                                losses: if person.win == true {0} else {1},
                                teamId: person.teamId,
                                synergy_score: None,
                                average_assists: person.assists,
                                average_damage_dealt_to_objectives: person.damageDealtToObjectives,
                                average_deaths: person.deaths,
                                average_gold_earned: person.goldEarned,
                                average_kills: person.kills,
                                average_total_damage_dealt_to_champions: person.totalDamageDealtToChampions,
                                average_total_damage_shielded_on_teammates: person.totalDamageShieldedOnTeammates,
                                average_total_heals_on_teammates: person.totalHealsOnTeammates,
                                average_total_minions_killed: person.totalMinionsKilled,
                                average_neutral_minions_killed: person.neutralMinionsKilled,
                                average_vision_score: person.visionScore,
                            }
                        )
                    }
                }
                //otherwise add to enemy team
                else {
                    if let Some(champ) = organized_games.games.enemy_team.iter_mut().find(|champ| champ.championName == person.championName) {
                        champ.average_assists += person.assists;
                        champ.average_damage_dealt_to_objectives += person.damageDealtToObjectives;
                        champ.average_deaths += person.deaths;
                        champ.average_gold_earned += person.goldEarned;
                        champ.average_kills += person.kills;
                        champ.average_total_damage_dealt_to_champions += person.totalDamageDealtToChampions;
                        champ.average_total_damage_shielded_on_teammates += person.totalDamageShieldedOnTeammates;
                        champ.average_total_heals_on_teammates += person.totalHealsOnTeammates;
                        champ.average_total_minions_killed += person.totalMinionsKilled;
                        champ.average_neutral_minions_killed += person.neutralMinionsKilled;
                        champ.average_vision_score += person.visionScore; 
                        if person.win == true {champ.wins += 1;} else {champ.losses += 1;}
                    }
                    else {
                        organized_games.games.enemy_team.push(
                            ChampionsInfo {
                                championName: person.championName.clone(),
                                wins: if person.win == true {1} else {0},
                                losses: if person.win == true {0} else {1},
                                teamId: person.teamId,
                                synergy_score: None,
                                average_assists: person.assists,
                                average_damage_dealt_to_objectives: person.damageDealtToObjectives,
                                average_deaths: person.deaths,
                                average_gold_earned: person.goldEarned,
                                average_kills: person.kills,
                                average_total_damage_dealt_to_champions: person.totalDamageDealtToChampions,
                                average_total_damage_shielded_on_teammates: person.totalDamageShieldedOnTeammates,
                                average_total_heals_on_teammates: person.totalHealsOnTeammates,
                                average_total_minions_killed: person.totalMinionsKilled,
                                average_neutral_minions_killed: person.neutralMinionsKilled,
                                average_vision_score: person.visionScore,
                            }
                        )
                    }
                }
            }
        }
    }
    // get averages of games
    organized_games.games.enemy_team.iter_mut().for_each(|champ_stat| {
        champ_stat.average_assists /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_damage_dealt_to_objectives /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_deaths /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_gold_earned /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_kills /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_total_damage_dealt_to_champions /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_damage_shielded_on_teammates /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_heals_on_teammates /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_minions_killed /= (champ_stat.wins + champ_stat.losses) as u16;
        champ_stat.average_neutral_minions_killed /= (champ_stat.wins + champ_stat.losses) as u16;
        champ_stat.average_vision_score /= (champ_stat.wins + champ_stat.losses) as u16;
        
    });
    organized_games.games.your_team.iter_mut().for_each(|champ_stat| {
        champ_stat.average_assists /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_damage_dealt_to_objectives /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_deaths /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_gold_earned /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_kills /= champ_stat.wins + champ_stat.losses;
        champ_stat.average_total_damage_dealt_to_champions /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_damage_shielded_on_teammates /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_heals_on_teammates /= (champ_stat.wins + champ_stat.losses) as u32;
        champ_stat.average_total_minions_killed /= (champ_stat.wins + champ_stat.losses) as u16;
        champ_stat.average_neutral_minions_killed /= (champ_stat.wins + champ_stat.losses) as u16;
        champ_stat.average_vision_score /= (champ_stat.wins + champ_stat.losses) as u16;
        
    });
    calculate_synergy_score(&mut organized_games);
    organized_games
}

pub fn calculate_synergy_score(organized_games: &mut SynergyMatches) -> &mut SynergyMatches  {
    for mut champ_stats in organized_games.games.your_team.iter_mut() {
        champ_stats.synergy_score = Some((champ_stats.average_assists as f32) as f32
        + (champ_stats.average_damage_dealt_to_objectives / 2000) as f32
        - (champ_stats.average_deaths) as f32
        + (champ_stats.average_gold_earned / 1000) as f32
        + (champ_stats.average_kills) as f32
        + (champ_stats.average_total_damage_dealt_to_champions / 1500) as f32
        + (champ_stats.average_total_damage_shielded_on_teammates / 1000) as f32
        + (champ_stats.average_total_heals_on_teammates / 800) as f32
        + (champ_stats.average_total_minions_killed / 15) as f32
        + (champ_stats.average_neutral_minions_killed / 12) as f32
        + (champ_stats.average_vision_score / 5) as f32 
        //   pub totalDamageShieldedOnTeammates: u32,
        //   pub totalHealsOnTeammates: u32,
        //   pub totalMinionsKilled: u16,
        //   pub visionScore: u16,
        - (champ_stats.losses * 10) as f32
        + (champ_stats.wins * 12) as f32);
        // more synergy added for how much u played with that champ
    }
    for mut champ_stats in organized_games.games.enemy_team.iter_mut() {
        champ_stats.synergy_score = Some((champ_stats.average_assists as f32) as f32
        + (champ_stats.average_damage_dealt_to_objectives / 3000) as f32
        - (champ_stats.average_deaths) as f32
        + (champ_stats.average_gold_earned / 1000) as f32
        + (champ_stats.average_kills) as f32
        + (champ_stats.average_total_damage_dealt_to_champions / 2500) as f32
        + (champ_stats.average_total_damage_shielded_on_teammates / 1200) as f32
        + (champ_stats.average_total_heals_on_teammates / 1200) as f32
        + (champ_stats.average_total_minions_killed / 20) as f32
        + (champ_stats.average_neutral_minions_killed / 18) as f32
        + (champ_stats.average_vision_score / 3) as f32 
        //   pub totalDamageShieldedOnTeammates: u32,
        //   pub totalHealsOnTeammates: u32,
        //   pub totalMinionsKilled: u16,
        //   pub visionScore: u16,
        - (champ_stats.losses * 14) as f32
        + (champ_stats.wins * 20 ) as f32
        // more synergy added for how much u played with that champ
        + (champ_stats.wins as f32 + champ_stats.losses as f32));
    }
    organized_games
}