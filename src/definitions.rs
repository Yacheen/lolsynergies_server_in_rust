use std::{ time::Duration };
use serde::{Deserialize, Serialize}; 
//db
use mongodb::bson::doc;

#[derive(Deserialize, Debug)] pub struct SynergiesPostBody { 
  pub username: String, 
  pub platform_routing_value: String, 
  pub regional_routing_value: String 
}
#[derive(Deserialize, Debug)] pub struct Summoner { 
  pub puuid: String, 
  pub name: String, 
  pub profileIconId: i32, 
  pub summonerLevel: u64, 
  pub id: String 
}

#[derive(Deserialize, Serialize, Debug)] 
pub struct RankedEntry {
  pub queueType: String,
  pub tier: Option<String>,
  pub rank: Option<String>,
  pub leaguePoints: i32,
  pub wins: i32,
  pub losses: i32
}

#[derive(Deserialize, Debug)] pub struct MatchIds (pub String);
#[derive(Deserialize, Serialize, Debug)] pub struct Game { pub info: GameInfo }
#[derive(Deserialize, Serialize, Debug)] pub struct GameInfo { pub gameCreation: u64, pub participants: Vec<Participant> }

#[derive(Deserialize, Serialize, Debug)] pub struct Participant {
  pub champLevel: u16,
  pub assists: u8,
  pub damageDealtToObjectives: u32,
  pub damageSelfMitigated: u32,
  pub deaths: u8,
  pub goldEarned: u32,
  pub kills: u8,
  pub totalDamageDealtToChampions: u32,
  pub summonerName: String,
  pub championName: String,
  pub win: bool, pub teamId: u8,
  pub puuid: String
}

//format riot data to store in db into this struct:
#[derive(Deserialize, Serialize, Debug)] pub struct RawUserData {
  pub username: String, 
  pub display_name: Option<String>, 
  pub profileIconId: i32, 
  pub summonerLevel: u64, 
  pub puuid: String, 
  pub amount_of_games: u8, 
  pub last_updated: Duration, 
  pub games: Vec<Game>, 
  pub ranked_info: Vec<RankedEntry>
}


//organized data struct for synergies
#[derive(Deserialize, Serialize, Debug)] pub struct SynergyMatches {
  pub username: String, 
  pub display_name: Option<String>, 
  pub profileIconId: i32, 
  pub summonerLevel: u64, 
  pub amount_of_games: u8, 
  pub last_updated: Duration, 
  pub games: Winrates, 
  pub ranked_info: Vec<RankedEntry>
}
 impl SynergyMatches {
  pub fn new(last_updated: Duration) -> SynergyMatches {
    let games = Winrates { your_team: Vec::new(), enemy_team: Vec::new() };
    SynergyMatches {
      username: String::new(),
      display_name: None,
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


#[derive(Serialize, Debug, Deserialize)] pub struct Winrates { 
  pub your_team: Vec<ChampionsInfo>, 
  pub enemy_team: Vec<ChampionsInfo> 
}
 impl Winrates {
  fn new(your_team: Vec<ChampionsInfo>, enemy_team: Vec<ChampionsInfo>) -> Winrates{
    Winrates {
      your_team,
      enemy_team
    }
  }
}


#[derive(Serialize, Debug, Deserialize)] pub struct ChampionsInfo { 
  pub championName: String, 
  pub wins: u8, 
  pub losses: u8, 
  pub teamId: u8 
}
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