use core::club::player::{Player, PlayerPositionType};
use core::club::team::tactics::{MatchTacticType, Tactics};
use core::r#match::player::MatchPlayer;
use core::r#match::{FootballEngine, MatchSquad};
use core::staff_contract_mod::NaiveDate;
use core::{AcademyGenerationContext, PeopleNameGeneratorData, PlayerGenerator, PlayerSkills};
use std::cell::RefCell;
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
// Apache 2.0): reposiciona a MEDIA de skills para BASE + nivel*STEP.
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

// Monta um MatchSquad emprestando (sem mover/clonar) os jogadores ja gerados.
fn squad_from_players(team_id: u32, players: &[Player]) -> MatchSquad {
    let main_squad: Vec<MatchPlayer> = players
        .iter()
        .zip(POSITIONS_442.iter())
        .map(|(p, &pos)| MatchPlayer::from_player(team_id, p, pos, false, None))
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

fn parse_two(dbg: &str) -> (u32, u32) {
    let mut it = dbg.split("score:").skip(1).map(|seg| {
        seg.trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(0)
    });
    (it.next().unwrap_or(0), it.next().unwrap_or(0))
}

// ===== Mundo do jogo: os times (elencos) sao gerados UMA vez por temporada e
// reaproveitados em todas as partidas (a geracao de jogador e o gargalo). =====
thread_local! {
    static WORLD: RefCell<Vec<Vec<Player>>> = RefCell::new(Vec::new());
}

// Gera uma liga com um elenco por nivel informado (ids unicos por time).
#[wasm_bindgen]
pub fn setup_league(levels: &[u8]) {
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        w.clear();
        for (ti, &lvl) in levels.iter().enumerate() {
            let team_id = (ti as u32) + 1;
            let base = team_id * 100;
            let players: Vec<Player> = POSITIONS_442
                .iter()
                .enumerate()
                .map(|(i, &pos)| make_player(base + i as u32, pos, lvl))
                .collect();
            w.push(players);
        }
    });
}

fn play_pair(home_idx: u32, away_idx: u32) -> (u32, u32) {
    WORLD.with(|w| {
        let w = w.borrow();
        let home = squad_from_players(home_idx + 1, &w[home_idx as usize]);
        let away = squad_from_players(away_idx + 1, &w[away_idx as usize]);
        let result = FootballEngine::<840, 545>::play(home, away, false, true, false);
        parse_two(&format!("{:?}", result.score))
    })
}

// Joga uma partida entre dois times ja montados por setup_league.
#[wasm_bindgen]
pub fn play_match(home_idx: u32, away_idx: u32) -> String {
    let (h, a) = play_pair(home_idx, away_idx);
    format!("{{\"home\":{},\"away\":{}}}", h, a)
}

// Compat: partida avulsa entre dois niveis (gera na hora — mais lento).
#[wasm_bindgen]
pub fn simulate_match(home_level: u8, away_level: u8) -> String {
    let home = squad_from_players(1, &(0..11).map(|i| make_player(100 + i, POSITIONS_442[i as usize], home_level)).collect::<Vec<_>>());
    let away = squad_from_players(2, &(0..11).map(|i| make_player(200 + i, POSITIONS_442[i as usize], away_level)).collect::<Vec<_>>());
    let result = FootballEngine::<840, 545>::play(home, away, false, true, false);
    let (h, a) = parse_two(&format!("{:?}", result.score));
    format!("{{\"home\":{},\"away\":{}}}", h, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn perf_and_bias() {
        let levels = [5u8, 6, 5, 7, 6, 5, 8, 6];
        let t = Instant::now();
        setup_league(&levels);
        println!("GENALL== {} ms p/ 8 times (88 jogadores) ==", t.elapsed().as_millis());

        let n = 20u32;
        let t = Instant::now();
        for i in 0..n {
            let _ = play_pair((i % 8) as u32, ((i + 1) % 8) as u32);
        }
        let total = t.elapsed().as_millis();
        println!("PLAYAVG== {} ms/partida ({} partidas em {} ms, elencos em cache) ==", total / n as u128, n, total);

        setup_league(&[16u8, 2, 16, 2, 16, 2, 16, 2]);
        let (mut sw, mut d, mut ww) = (0u32, 0u32, 0u32);
        for _ in 0..20 {
            let (h, a) = play_pair(0, 1);
            if h > a { sw += 1; } else if h < a { ww += 1; } else { d += 1; }
        }
        println!("BIAS_CACHE== forte x fraco: Vforte={} E={} Vfraco={} em 20 (com cache) ==", sw, d, ww);
    }
}
