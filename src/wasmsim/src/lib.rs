use core::club::player::{Player, PlayerPositionType};
use core::club::team::tactics::{MatchTacticType, Tactics};
use core::r#match::player::MatchPlayer;
use core::r#match::{FootballEngine, MatchSquad};
use core::staff_contract_mod::NaiveDate;
use core::{AcademyGenerationContext, PeopleNameGeneratorData, PlayerGenerator};
use wasm_bindgen::prelude::*;

const POSITIONS_442: [PlayerPositionType; 11] = [
    PlayerPositionType::Goalkeeper,
    PlayerPositionType::DefenderLeft,
    PlayerPositionType::DefenderCenterLeft,
    PlayerPositionType::DefenderCenterRight,
    PlayerPositionType::DefenderRight,
    PlayerPositionType::MidfielderLeft,
    PlayerPositionType::MidfielderCenterLeft,
    PlayerPositionType::MidfielderCenterRight,
    PlayerPositionType::MidfielderRight,
    PlayerPositionType::ForwardLeft,
    PlayerPositionType::ForwardRight,
];

fn make_player(id: u32, position: PlayerPositionType) -> Player {
    let empty = PeopleNameGeneratorData {
        first_names: Vec::new(),
        last_names: Vec::new(),
        nicknames: Vec::new(),
    };
    let now = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let mut p = PlayerGenerator::generate_with_context(
        1, now, position, &empty, &AcademyGenerationContext::average(), 25, 28, None,
    );
    p.id = id;
    p
}

fn make_squad(team_id: u32) -> MatchSquad {
    let base = team_id * 100;
    let main_squad: Vec<MatchPlayer> = POSITIONS_442
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            MatchPlayer::from_player(team_id, &make_player(base + i as u32, pos), pos, false, None)
        })
        .collect();
    MatchSquad {
        team_id,
        team_name: format!("Team {}", team_id),
        tactics: Tactics::new(MatchTacticType::T442),
        main_squad,
        substitutes: Vec::new(),
        captain_id: None,
        vice_captain_id: None,
        penalty_taker_id: None,
        free_kick_taker_id: None,
        selection_omissions: Vec::new(),
        coach_snapshot: None,
    }
}

#[wasm_bindgen]
pub fn simulate_match() -> String {
    let home = make_squad(1);
    let away = make_squad(2);
    let result = FootballEngine::<840, 545>::play(home, away, false, true, false);
    match result.score {
        Some(s) => format!("{{\"home\":{},\"away\":{}}}", s.home_team.score, s.away_team.score),
        None => "{\"home\":0,\"away\":0}".to_string(),
    }
}
