use glob::glob;
use indicatif::{ProgressIterator, ProgressStyle};

mod model;

use model::Tokenize;

// all mlb team ids
const TEAM_IDS: [u8; 30] = [108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 158];

/// Remove the completed team from the list of teams to be processed in the given season.
fn save_progress(season: u16, completed_team_id: u8) {
    let mut progress = serde_json::from_str::<serde_json::Value>(std::fs::read_to_string("data/progress.json").unwrap_or("{}".to_string()).as_str()).unwrap();

    if progress.get(&season.to_string()).is_none() {
        progress[season.to_string()] = serde_json::Value::Array(TEAM_IDS.iter().map(|id| serde_json::Value::Number(serde_json::Number::from(*id))).collect());
    }

    let progress_season = progress.get_mut(&season.to_string()).unwrap().as_array_mut().unwrap();
    progress_season.retain(|id| id.as_u64().unwrap() != completed_team_id as u64);

    std::fs::write("data/progress.json", serde_json::to_string_pretty(&progress).unwrap()).unwrap();
}

/// Get all game pks for a given team in a given season.
fn game_pks_for_team_in_season(team_id: u8, season: u16) -> Vec<usize> {
    let all_games = glob(format!("data/{season}/**/*.json").as_str()).unwrap();

    let mut game_pks = Vec::new();
    for game_path in all_games {
        let game_path = game_path.unwrap();
        let game = serde_json::from_str::<model::Game>(&std::fs::read_to_string(game_path).unwrap()).unwrap();

        if game.context.home_team.id == team_id || game.context.away_team.id == team_id {
            game_pks.push(game.context.game_pk);
        }
    }

    game_pks
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1) {
        Some(command) => match command.as_str() {
            "get" => {
                let season = std::env::args().nth(2).unwrap().parse::<u16>().unwrap();
                let progress = serde_json::from_str::<serde_json::Value>(std::fs::read_to_string("data/progress.json").unwrap_or("{}".to_string()).as_str()).unwrap();
                let progress_season = match progress.get(&season.to_string()) {
                    Some(progress_season) => progress_season.as_array().unwrap().iter().map(|id| id.as_u64().unwrap() as u8).collect(),
                    None => TEAM_IDS.to_vec(),
                };
                println!("Processing season {} for {} teams ({:?})", season, progress_season.len(), progress_season);

                let progress_style = ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} | elapsed: {elapsed_precise}, eta: {eta_precise}").unwrap();
                for team_id in progress_season.iter().progress_with_style(progress_style) {
                    let _ = model::Game::get_all_by_team_in_season(
                        *team_id,
                        season,
                        game_pks_for_team_in_season(*team_id, season),
                    ).await;
                    save_progress(season, *team_id);
                }
            },
            "tokenize" => {
                let ignore_paths = ["data/log.txt", "data/progress.json"];
                let all_games = glob("data/**/*.json")
                    .unwrap()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|game_path| game_path.as_ref().unwrap().to_str().unwrap().to_string())
                    .filter(|game_path| !ignore_paths.contains(&game_path.as_str()))
                    .collect::<Vec<String>>();

                let progress_style = ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} | elapsed: {elapsed_precise}, eta: {eta_precise}").unwrap();
                for game_path in all_games.iter().progress_with_style(progress_style) {
                    let game = serde_json::from_str::<model::Game>(&std::fs::read_to_string(game_path).unwrap()).unwrap();
                    let tokens = game.tokenize();

                    let tokens_path = game_path
                        .replace("data", "tokenized_data")
                        .replace(".json", ".txt");

                    let parts = tokens_path
                        .split('/')
                        .rev()
                        .skip(1)
                        .collect::<Vec<&str>>()
                        .iter()
                        .rev()
                        .map(|part| part.to_string())
                        .collect::<Vec<String>>()
                        .join("/");

                    std::fs::create_dir_all(parts).unwrap();
                    std::fs::write(tokens_path, tokens).unwrap();
                }
            }
            "getone" => {
                let game_pk = std::env::args().nth(2).unwrap().parse::<usize>().unwrap();
                let _ = model::Game::from_game_pk(game_pk).await.unwrap();
            },
            _ => eprintln!("Unknown command."),
        },
        None => eprintln!("Please provide a command."),
    }
}
