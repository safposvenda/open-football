use core::club::player::{Player, PlayerPositionType};
use core::club::team::tactics::{MatchTacticType, Tactics};
use core::r#match::player::MatchPlayer;
use core::r#match::{FootballEngine, MatchSquad};
use core::staff_contract_mod::NaiveDate;
use core::{AcademyGenerationContext, PeopleNameGeneratorData, PlayerGenerator, PlayerSkills};
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

// Curva de forca por "nivel" (copiada do harness .dev/match do open-football,
// Apache 2.0): reposiciona a MEDIA de skills do jogador para BASE + nivel*STEP,
// preservando o formato por posicao (atacante continua finalizador, zagueiro
// continua marcador). E isso que faz times mais fortes/mais fracos.
struct LevelSkillCurve;
impl LevelSkillCurve {
    const BASE: f32 = 3.6;
    const STEP: f32 = 0.575;
    const MATCH_READINESS: f32 = 14.0;
    fn target_mean(level: u8) -> f32 {
        Self::BASE + level as f32 * Self::STEP
    }
    fn retarget(skills: &mut PlayerSkills, target_mean: f32) {
        let cur_mean = Self::current_mean(skills);
        let delta = target_mean - cur_mean;
        skills.physical.match_readiness = Self::MATCH_READINESS;
        Self::shift_all(skills, delta);
    }
    fn current_mean(skills: &PlayerSkills) -> f32 {
        let s = &skills.technical;
        let m = &skills.mental;
        let p = &skills.physical;
        let g = &skills.goalkeeping;
        let total = s.corners + s.crossing + s.dribbling + s.finishing + s.first_touch
            + s.free_kicks + s.heading + s.long_shots + s.long_throws + s.marking
            + s.passing + s.penalty_taking + s.tackling + s.technique
            + m.aggression + m.anticipation + m.bravery + m.composure + m.concentration
            + m.decisions + m.determination + m.flair + m.leadership + m.off_the_ball
            + m.positioning + m.teamwork + m.vision + m.work_rate
            + p.acceleration + p.agility + p.balance + p.jumping + p.natural_fitness
            + p.pace + p.stamina + p.strength
            + g.aerial_reach + g.command_of_area + g.communication + g.eccentricity
            + g.first_touch + g.handling + g.kicking + g.one_on_ones + g.passing
            + g.punching + g.reflexes + g.rushing_out + g.throwing;
        total / (14 + 14 + 8 + 13) as f32
    }
    fn shift_all(skills: &mut PlayerSkills, delta: f32) {
        let bump = |x: &mut f32| *x = (*x + delta).clamp(1.0, 20.0);
        let s = &mut skills.technical;
        bump(&mut s.corners); bump(&mut s.crossing); bump(&mut s.dribbling);
        bump(&mut s.finishing); bump(&mut s.first_touch); bump(&mut s.free_kicks);
        bump(&mut s.heading); bump(&mut s.long_shots); bump(&mut s.long_throws);
        bump(&mut s.marking); bump(&mut s.passing); bump(&mut s.penalty_taking);
        bump(&mut s.tackling); bump(&mut s.technique);
        let m = &mut skills.mental;
        bump(&mut m.aggression); bump(&mut m.anticipation); bump(&mut m.bravery);
        bump(&mut m.composure); bump(&mut m.concentration); bump(&mut m.decisions);
        bump(&mut m.determination); bump(&mut m.flair); bump(&mut m.leadership);
        bump(&mut m.off_the_ball); bump(&mut m.positioning); bump(&mut m.teamwork);
        bump(&mut m.vision); bump(&mut m.work_rate);
        let p = &mut skills.physical;
        bump(&mut p.acceleration); bump(&mut p.agility); bump(&mut p.balance);
        bump(&mut p.jumping); bump(&mut p.natural_fitness); bump(&mut p.pace);
        bump(&mut p.stamina); bump(&mut p.strength);
        let g = &mut skills.goalkeeping;
        bump(&mut g.aerial_reach); bump(&mut g.command_of_area); bump(&mut g.communication);
        bump(&mut g.eccentricity); bump(&mut g.first_touch); bump(&mut g.handling);
        bump(&mut g.kicking); bump(&mut g.one_on_ones); bump(&mut g.passing);
        bump(&mut g.punching); bump(&mut g.reflexes); bump(&mut g.rushing_out);
        bump(&mut g.throwing);
    }
}

fn make_player(id: u32, position: PlayerPositionType, level: u8) -> Player {
    let empty = PeopleNameGeneratorData {
        first_names: Vec::new(), last_names: Vec::new(), nicknames: Vec::new(),
    };
    let now = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let mut p = PlayerGenerator::generate_with_context(
        1, now, position, &empty, &AcademyGenerationContext::average(), 25, 28, None,
    );
    LevelSkillCurve::retarget(&mut p.skills, LevelSkillCurve::target_mean(level));
    p.id = id;
    p
}

fn make_squad(team_id: u32, level: u8) -> MatchSquad {
    let base = team_id * 100;
    let main_squad: Vec<MatchPlayer> = POSITIONS_442
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            MatchPlayer::from_player(team_id, &make_player(base + i as u32, pos, level), pos, false, None)
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

// Roda uma partida e extrai (gols_casa, gols_fora) do Debug (campos privados).
fn play_once(home_level: u8, away_level: u8) -> (u32, u32) {
    let home = make_squad(1, home_level);
    let away = make_squad(2, away_level);
    let result = FootballEngine::<840, 545>::play(home, away, false, true, false);
    let dbg = format!("{:?}", result.score);
    let mut it = dbg.split("score:").skip(1).map(|seg| {
        seg.trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0)
    });
    let h = it.next().unwrap_or(0);
    let a = it.next().unwrap_or(0);
    (h, a)
}

#[wasm_bindgen]
pub fn simulate_match() -> String {
    let home = make_squad(1, 9);
    let away = make_squad(2, 9);
    let result = FootballEngine::<840, 545>::play(home, away, false, true, false);
    format!("{:?}", result.score)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn series(home_level: u8, away_level: u8, n: u32) -> (u32, u32, u32, u32, u32) {
        let (mut hw, mut d, mut aw, mut hg, mut ag) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for _ in 0..n {
            let (h, a) = play_once(home_level, away_level);
            hg += h; ag += a;
            if h > a { hw += 1; } else if h < a { aw += 1; } else { d += 1; }
        }
        (hw, d, aw, hg, ag)
    }
    #[test]
    fn quality_bias() {
        let n = 40;
        let (hw, d, aw, hg, ag) = series(16, 2, n);
        println!("EXP_STRONG_HOME== forte(16) manda x fraco(2): Vforte={} E={} Vfraco={} | gols {}-{} em {} jogos ==", hw, d, aw, hg, ag, n);
        let (hw2, d2, aw2, hg2, ag2) = series(2, 16, n);
        println!("EXP_WEAK_HOME== fraco(2) manda x forte(16): Vfraco={} E={} Vforte={} | gols {}-{} em {} jogos ==", hw2, d2, aw2, hg2, ag2, n);
        let (hw3, d3, aw3, hg3, ag3) = series(9, 9, n);
        println!("EXP_EQUAL== iguais(9) x iguais(9): V1={} E={} V2={} | gols {}-{} em {} jogos ==", hw3, d3, aw3, hg3, ag3, n);
    }
}
