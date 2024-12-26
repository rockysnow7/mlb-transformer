use futures::future::join_all;
use indicatif::{ProgressIterator, ProgressStyle};
use serde::{Serialize, Deserialize};
use std::io::Write;

pub trait Tokenize {
    fn tokenize(&self) -> String;
}

fn log(message: String) {
    let _ = std::fs::create_dir("data");

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/log.txt")
        .unwrap();

    writeln!(file, "{}", message).unwrap();
}

async fn get_player_name_from_id(player_id: usize) -> String {
    let url = format!("https://statsapi.mlb.com/api/v1/people/{player_id}");
    let response = reqwest::get(&url).await.unwrap();
    let player_data = response.json::<serde_json::Value>().await.unwrap();
    let player_name = player_data["people"][0]["fullName"].as_str().unwrap().to_string();

    player_name
}

fn base_value_to_option_u8(base: &serde_json::Value) -> Option<u8> {
    if base.is_null() {
        return None;
    }

    match base.as_str().unwrap() {
        "1B" => Some(1),
        "2B" => Some(2),
        "3B" => Some(3),
        "4B" | "score" => Some(4),
        _ => panic!("Unknown base value: {}", base),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl ToString for Date {
    fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl From<&str> for Date {
    fn from(date_str: &str) -> Self {
        let date_parts: Vec<&str> = date_str.split("-").collect();

        Date {
            year: date_parts[0].parse().unwrap(),
            month: date_parts[1].parse().unwrap(),
            day: date_parts[2].parse().unwrap(),
        }
    }
}

impl Tokenize for Date {
    fn tokenize(&self) -> String {
        format!("[DATE] {}", self.to_string())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Position {
    Pitcher,
    Catcher,
    FirstBase,
    SecondBase,
    ThirdBase,
    Shortstop,
    LeftField,
    CenterField,
    RightField,
    DesignatedHitter,
    PinchHitter,
    PinchRunner,
    TwoWayPlayer,
    Outfield,
    Infield,
    Utility,
    ReliefPitcher,
    StartingPitcher,
}

impl Position {
    pub fn from_abbr(position_abbr: &str) -> Self {
        match position_abbr {
            "P" => Position::Pitcher,
            "C" => Position::Catcher,
            "1B" => Position::FirstBase,
            "2B" => Position::SecondBase,
            "3B" => Position::ThirdBase,
            "SS" => Position::Shortstop,
            "LF" => Position::LeftField,
            "CF" => Position::CenterField,
            "RF" => Position::RightField,
            "DH" => Position::DesignatedHitter,
            "PH" => Position::PinchHitter,
            "PR" => Position::PinchRunner,
            "TWP" => Position::TwoWayPlayer,
            "OF" => Position::Outfield,
            "IF" => Position::Infield,
            "UTIL" => Position::Utility,
            "RP" => Position::ReliefPitcher,
            "SP" => Position::StartingPitcher,
            _ => panic!("Unknown position abbreviation: {}", position_abbr),
        }
    }
}

impl ToString for Position {
    fn to_string(&self) -> String {
        match self {
            Position::Pitcher => "PITCHER",
            Position::Catcher => "CATCHER",
            Position::FirstBase => "FIRST_BASE",
            Position::SecondBase => "SECOND_BASE",
            Position::ThirdBase => "THIRD_BASE",
            Position::Shortstop => "SHORTSTOP",
            Position::LeftField => "LEFT_FIELD",
            Position::CenterField => "CENTER_FIELD",
            Position::RightField => "RIGHT_FIELD",
            Position::DesignatedHitter => "DESIGNATED_HITTER",
            Position::PinchHitter => "PINCH_HITTER",
            Position::PinchRunner => "PINCH_RUNNER",
            Position::TwoWayPlayer => "TWO_WAY_PLAYER",
            Position::Outfield => "OUTFIELD",
            Position::Infield => "INFIELD",
            Position::Utility => "UTILITY",
            Position::ReliefPitcher => "RELIEF_PITCHER",
            Position::StartingPitcher => "STARTING_PITCHER",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    name: String,
    position: Position,
}

impl Player {
    pub async fn new(name: String, position: Position) -> Result<Self, String> {
        Ok(Self { name, position })
    }
}

impl Tokenize for Player {
    fn tokenize(&self) -> String {
        format!("[{}] {}", self.position.to_string(), self.name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: u8,
    players: Vec<Player>,
}

impl Team {
    pub async fn from_boxscore_team_data_and_date(team_data: &serde_json::Value) -> Result<Self, String> {
        let id = team_data["team"]["id"].as_u64().unwrap() as u8;
        let players_data = team_data["players"].as_object().unwrap();

        let mut players = Vec::new();
        for player_data in players_data.values() {
            let player_name = player_data["person"]["fullName"].as_str().unwrap().to_string();
            let position_abbr = if let Some(abbr) = player_data["position"]["abbreviation"].as_str() {
                abbr
            } else {
                return Err("No position abbreviation".to_string());
            };
            let position = Position::from_abbr(position_abbr);

            let player = Player::new(player_name, position).await?;

            players.push(player);
        }

        Ok(Self {
            id,
            players,
        })
    }
}

impl Tokenize for Team {
    fn tokenize(&self) -> String {
        let mut tokens = String::new();

        tokens += &format!("[TEAM] {}\n", self.id);
        for player in &self.players {
            tokens += &format!("{}\n", player.tokenize());
        }

        tokens
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    condition: String,
    temperature: u8,
    wind_speed: u8,
}

impl Weather {
    pub fn from_value(value: &serde_json::Value) -> Self {
        let condition = value["condition"].as_str().unwrap().to_string();
        let temperature = value["temp"].as_str().unwrap().parse().unwrap();
        let wind_speed = value["wind"]
            .as_str()
            .unwrap()
            .to_string()
            .split(' ')
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .parse()
            .unwrap();

        Self {
            condition,
            temperature,
            wind_speed,
        }
    }
}

impl Tokenize for Weather {
    fn tokenize(&self) -> String {
        format!("[WEATHER] {} {} {}", self.condition, self.temperature, self.wind_speed)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameContext {
    pub game_pk: usize,
    date: Date,
    venue_name: String,
    weather: Weather,
    pub home_team: Team,
    pub away_team: Team,
}

impl GameContext {
    pub async fn from_game_boxscore_data_and_date_and_weather_and_game_pk(
        game_data: &serde_json::Value,
        game_date: Date,
        weather: Weather,
        game_pk: usize,
    ) -> Result<Self, String> {
        let home_team_data = &game_data["teams"]["home"];
        let home_team = Team::from_boxscore_team_data_and_date(home_team_data).await?;
        let venue_name = home_team_data["team"]["venue"]["name"].as_str().unwrap().to_string();

        let away_team_data = &game_data["teams"]["away"];
        let away_team = Team::from_boxscore_team_data_and_date(away_team_data).await?;

        Ok(Self {
            game_pk,
            date: game_date,
            venue_name,
            weather,
            home_team,
            away_team,
        })
    }
}

impl Tokenize for GameContext {
    fn tokenize(&self) -> String {
        format!(
            "{} [DATE] {} [VENUE] {} {}\n\n{}\n{}",
            self.game_pk,
            self.date.to_string(),
            self.venue_name,
            self.weather.tokenize(),
            self.home_team.tokenize(),
            self.away_team.tokenize(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Movement {
    pub runner: String,
    pub start_base: Option<u8>,
    pub end_base: Option<u8>,
    pub is_out: bool,
}

impl Movement {
    pub fn from_runner_and_value(runner: String, movement_value: &serde_json::Value) -> Self {
        let start_base = base_value_to_option_u8(&movement_value["start"]);
        let end_base = base_value_to_option_u8(&movement_value["end"]);
        let is_out = movement_value["isOut"].as_bool().unwrap_or(false);

        Movement {
            runner,
            start_base,
            end_base,
            is_out,
        }
    }
}

impl Tokenize for Movement {
    fn tokenize(&self) -> String {
        let mut tokens = String::new();

        tokens += &format!("{} ", self.runner);

        let start_base_str = match self.start_base {
            Some(base) => base.to_string(),
            None => "home".to_string(),
        };
        let end_base_str = match self.end_base {
            Some(base) => base.to_string(),
            None => "home".to_string(),
        };

        tokens += &format!("{} -> {}", start_base_str, end_base_str);

        if self.is_out {
            tokens += " [out]";
        }

        tokens
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Play {
    // outs
    Groundout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntGroundout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Strikeout {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Lineout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntLineout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Flyout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PopOut {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntPopOut {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Forceout {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    FieldersChoiceOut {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    DoublePlay {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    TriplePlay {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    GroundedIntoDoublePlay {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    StrikeoutDoublePlay {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Pickoff {
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PickoffError {
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    CaughtStealing {
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PickoffCaughtStealing {
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    WildPitch {
        pitcher: String,
        runner: String,
        movements: Vec<Movement>,
    },
    RunnerOut {
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    FieldOut {
        fielder: String,
        runner: String,
        movements: Vec<Movement>,
    },
    // scores
    Single {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Double {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Triple {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    HomeRun {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Walk {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    IntentWalk {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    HitByPitch {
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    FieldersChoice {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    CatcherInterference {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    StolenBase {
        base: u8,
        runner: String,
        movements: Vec<Movement>,
    },
    // other
    SacFly {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    SacFlyDoublePlay {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    SacBunt {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        runner: String,
        movements: Vec<Movement>,
    },
    FieldError {
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
}

impl Play {
    // outs
    async fn groundout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Groundout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_groundout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntGroundout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn strikeout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Strikeout {
            batter,
            pitcher,
            movements,
        })
    }

    async fn lineout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Lineout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_lineout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntLineout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn flyout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Flyout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn pop_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PopOut {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_pop_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntPopOut {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn forceout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Forceout {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn fielders_choice_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let scoring_runner = match value["runners"][1]["details"]["runner"]["fullName"].as_str() {
            Some(runner) => runner.to_string(),
            None => return Err("No scoring runner".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldersChoiceOut {
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::DoublePlay {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn triple_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::TriplePlay {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn grounded_into_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::GroundedIntoDoublePlay {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn strikeout_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::StrikeoutDoublePlay {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn pickoff_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Pickoff {
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn pickoff_error_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PickoffError {
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn caught_stealing_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::CaughtStealing {
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn pickoff_caught_stealing_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PickoffCaughtStealing {
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn wild_pitch_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = vec![Movement::from_runner_and_value(
            runner.clone(),
            &value["runners"][0]["movement"],
        )];

        Ok(Play::WildPitch {
            pitcher,
            runner,
            movements,
        })
    }

    async fn runner_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::RunnerOut {
            runner,
            fielders,
            movements,
        })
    }

    async fn field_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let fielder = match value["runners"][0]["details"]["fielder"]["fullName"].as_str() {
            Some(fielder) => fielder.to_string(),
            None => return Err("No fielder".to_string()),
        };
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = vec![Movement::from_runner_and_value(
            runner.clone(),
            &value["runners"][0]["movement"],
        )];

        Ok(Play::FieldOut {
            fielder,
            runner,
            movements,
        })
    }

    // scores
    async fn single_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Single {
            batter,
            pitcher,
            movements,
        })
    }

    async fn double_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Double {
            batter,
            pitcher,
            movements,
        })
    }

    async fn triple_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Triple {
            batter,
            pitcher,
            movements,
        })
    }

    async fn home_run_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::HomeRun {
            batter,
            pitcher,
            movements,
        })
    }

    async fn walk_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Walk {
            batter,
            pitcher,
            movements,
        })
    }

    async fn intent_walk_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::IntentWalk {
            batter,
            pitcher,
            movements,
        })
    }

    async fn hit_by_pitch_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::HitByPitch {
            batter,
            pitcher,
            movements,
        })
    }

    async fn fielders_choice_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldersChoice {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn catcher_interference_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::CatcherInterference {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn stolen_base_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::StolenBase {
            base,
            runner,
            movements,
        })
    }

    // other
    async fn sac_fly_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let scoring_runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacFly {
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn sac_fly_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let scoring_runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacFlyDoublePlay {
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn sac_bunt_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;
        let runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacBunt {
            batter,
            pitcher,
            fielders,
            runner,
            movements,
        })
    }

    async fn field_error_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let fielders = join_all(
            fielder_ids.into_iter().map(|id| get_player_name_from_id(id))
        ).await;

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldError {
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    pub async fn from_value(value: &serde_json::Value) -> Result<Self, String> {
        let play_type = value["result"]["event"].as_str().unwrap();

        match play_type {
            "Groundout" => Play::groundout_from_value(value).await,
            "Bunt Groundout" => Play::bunt_groundout_from_value(value).await,
            "Strikeout" => Play::strikeout_from_value(value).await,
            "Lineout" => Play::lineout_from_value(value).await,
            "Bunt Lineout" => Play::bunt_lineout_from_value(value).await,
            "Flyout" => Play::flyout_from_value(value).await,
            "Pop Out" => Play::pop_out_from_value(value).await,
            "Bunt Pop Out" => Play::bunt_pop_out_from_value(value).await,
            "Forceout" => Play::forceout_from_value(value).await,
            "Fielders Choice Out" => Play::fielders_choice_out_from_value(value).await,
            "Catcher Interference" => Play::catcher_interference_from_value(value).await,
            "Double Play" => Play::double_play_from_value(value).await,
            "Triple Play" => Play::triple_play_from_value(value).await,
            "Grounded Into DP" => Play::grounded_into_double_play_from_value(value).await,
            "Strikeout Double Play" => Play::strikeout_double_play_from_value(value).await,
            "Pickoff 1B" => Play::pickoff_from_value_and_base(value, 1).await,
            "Pickoff 2B" => Play::pickoff_from_value_and_base(value, 2).await,
            "Pickoff 3B" => Play::pickoff_from_value_and_base(value, 3).await,
            "Pickoff Error 1B" => Play::pickoff_error_from_value_and_base(value, 1).await,
            "Pickoff Error 2B" => Play::pickoff_error_from_value_and_base(value, 2).await,
            "Pickoff Error 3B" => Play::pickoff_error_from_value_and_base(value, 3).await,
            "Caught Stealing 2B" => Play::caught_stealing_from_value_and_base(value, 2).await,
            "Caught Stealing 3B" => Play::caught_stealing_from_value_and_base(value, 3).await,
            "Caught Stealing Home" => Play::caught_stealing_from_value_and_base(value, 4).await,
            "Pickoff Caught Stealing 1B" => Play::pickoff_caught_stealing_from_value_and_base(value, 1).await,
            "Pickoff Caught Stealing 2B" => Play::pickoff_caught_stealing_from_value_and_base(value, 2).await,
            "Pickoff Caught Stealing 3B" => Play::pickoff_caught_stealing_from_value_and_base(value, 3).await,
            "Pickoff Caught Stealing Home" => Play::pickoff_caught_stealing_from_value_and_base(value, 4).await,
            "Wild Pitch" => Play::wild_pitch_from_value(value).await,
            "Runner Out" => Play::runner_out_from_value(value).await,
            "Field Out" => Play::field_out_from_value(value).await,
            "Single" => Play::single_from_value(value).await,
            "Double" => Play::double_from_value(value).await,
            "Triple" => Play::triple_from_value(value).await,
            "Home Run" => Play::home_run_from_value(value).await,
            "Walk" => Play::walk_from_value(value).await,
            "Intent Walk" => Play::intent_walk_from_value(value).await,
            "Hit By Pitch" => Play::hit_by_pitch_from_value(value).await,
            "Fielders Choice" => Play::fielders_choice_from_value(value).await,
            "Stolen Base 1B" => Play::stolen_base_from_value_and_base(value, 1).await,
            "Stolen Base 2B" => Play::stolen_base_from_value_and_base(value, 2).await,
            "Stolen Base 3B" => Play::stolen_base_from_value_and_base(value, 3).await,
            "Sac Fly" => Play::sac_fly_from_value(value).await,
            "Sac Fly Double Play" => Play::sac_fly_double_play_from_value(value).await,
            "Sac Bunt" => Play::sac_bunt_from_value(value).await,
            "Field Error" => Play::field_error_from_value(value).await,
            _ => panic!("Unknown play type: {}", play_type),
        }
    }
}

impl Tokenize for Play {
    fn tokenize(&self) -> String {
        let mut tokens = "[PLAY] ".to_string();

        match self {
            Play::Groundout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Groundout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::BuntGroundout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Bunt Groundout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Strikeout { batter, pitcher, movements } => {
                tokens += &format!(
                    "Strikeout [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Lineout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Lineout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::BuntLineout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Bunt Lineout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Flyout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Flyout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::PopOut { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Pop Out [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::BuntPopOut { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Bunt Pop Out [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Forceout { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Forceout [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::FieldersChoiceOut { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!(
                    "Fielders Choice Out [BATTER] {} [PITCHER] {} [FIELDERS] {} [SCORING_RUNNER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                    scoring_runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::DoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Double Play [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::TriplePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Triple Play [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::GroundedIntoDoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Grounded Into Double Play [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::StrikeoutDoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Strikeout Double Play [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Pickoff { base, runner, fielders, movements } => {
                tokens += &format!(
                    "Pickoff [BASE] {} [RUNNER] {} [FIELDERS] {} [MOVEMENTS] ",
                    base,
                    runner,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::PickoffError { base, runner, fielders, movements } => {
                tokens += &format!(
                    "Pickoff Error [BASE] {} [RUNNER] {} [FIELDERS] {} [MOVEMENTS] ",
                    base,
                    runner,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::CaughtStealing { base, runner, fielders, movements } => {
                tokens += &format!(
                    "Caught Stealing [BASE] {} [RUNNER] {} [FIELDERS] {} [MOVEMENTS] ",
                    base,
                    runner,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::PickoffCaughtStealing { base, runner, fielders, movements } => {
                tokens += &format!(
                    "Pickoff Caught Stealing [BASE] {} [RUNNER] {} [FIELDERS] {} [MOVEMENTS] ",
                    base,
                    runner,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::WildPitch { pitcher, runner, movements } => {
                tokens += &format!(
                    "Wild Pitch [PITCHER] {} [RUNNER] {} [MOVEMENTS] ",
                    pitcher,
                    runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::RunnerOut { runner, fielders, movements } => {
                tokens += &format!(
                    "Runner Out [RUNNER] {} [FIELDERS] {} [MOVEMENTS] ",
                    runner,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::FieldOut { fielder, runner, movements } => {
                tokens += &format!(
                    "Field Out [FIELDER] {} [RUNNER] {} [MOVEMENTS] ",
                    fielder,
                    runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Single { batter, pitcher, movements } => {
                tokens += &format!(
                    "Single [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Double { batter, pitcher, movements } => {
                tokens += &format!(
                    "Double [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Triple { batter, pitcher, movements } => {
                tokens += &format!(
                    "Triple [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::HomeRun { batter, pitcher, movements } => {
                tokens += &format!(
                    "Home Run [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::Walk { batter, pitcher, movements } => {
                tokens += &format!(
                    "Walk [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::IntentWalk { batter, pitcher, movements } => {
                tokens += &format!(
                    "Intent Walk [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::HitByPitch { batter, pitcher, movements } => {
                tokens += &format!(
                    "Hit By Pitch [BATTER] {} [PITCHER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::FieldersChoice { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Fielders Choice [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::CatcherInterference { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Catcher Interference [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::StolenBase { base, runner, movements } => {
                tokens += &format!(
                    "Stolen Base [BASE] {} [RUNNER] {} [MOVEMENTS] ",
                    base,
                    runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::SacFly { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!(
                    "Sac Fly [BATTER] {} [PITCHER] {} [FIELDERS] {} [SCORING_RUNNER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                    scoring_runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::SacFlyDoublePlay { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!(
                    "Sac Fly Double Play [BATTER] {} [PITCHER] {} [FIELDERS] {} [SCORING_RUNNER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                    scoring_runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::SacBunt { batter, pitcher, fielders, runner, movements } => {
                tokens += &format!(
                    "Sac Bunt [BATTER] {} [PITCHER] {} [FIELDERS] {} [RUNNER] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                    runner,
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &movement.tokenize();

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
            Play::FieldError { batter, pitcher, fielders, movements } => {
                tokens += &format!(
                    "Field Error [BATTER] {} [PITCHER] {} [FIELDERS] {} [MOVEMENTS] ",
                    batter,
                    pitcher,
                    fielders.join(", "),
                );

                for (i, movement) in movements.iter().enumerate() {
                    tokens += &format!("{}", movement.tokenize());

                    if movements.len() > 1 && i < movements.len() - 1 {
                        tokens += ", ";
                    }
                }
            },
        };

        tokens
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub context: GameContext,
    pub plays: Vec<Play>,
}

impl Game {
    pub async fn from_game_pk(game_pk: usize) -> Result<Self, String> {
        let url = format!("https://statsapi.mlb.com/api/v1.1/game/{game_pk}/feed/live");
        log(format!("[Game::from_game_pk] Getting game: {url}"));
        let response = reqwest::get(&url).await.unwrap();
        let game_data = response.json::<serde_json::Value>().await.unwrap();

        if game_data["gameData"]["status"]["detailedState"].as_str().unwrap() != "Final" {
            return Err("Game is not final".to_string());
        }

        let game_date = game_data["gameData"]["datetime"]["originalDate"].as_str().unwrap();
        let game_date = Date::from(game_date);
        let weather = Weather::from_value(&game_data["gameData"]["weather"]);

        let plays_data = game_data["liveData"]["plays"]["allPlays"].as_array().unwrap();

        let mut plays = Vec::new();
        for play in plays_data {
            let p = Play::from_value(play).await?; // if any data is missing, discard the game
            plays.push(p);
        }

        let boxscore_data_url = format!("https://statsapi.mlb.com/api/v1/game/{game_pk}/boxscore");
        let boxscore_response = reqwest::get(&boxscore_data_url).await.unwrap();
        let boxscore_data = boxscore_response.json::<serde_json::Value>().await.unwrap();
        let context = GameContext::from_game_boxscore_data_and_date_and_weather_and_game_pk(
            &boxscore_data,
            game_date,
            weather,
            game_pk,
        ).await?;

        Ok(Self { context, plays })
    }

    pub fn save(&self, game_pk: usize) {
        std::fs::create_dir_all(format!(
            "data/{}/{}",
            self.context.date.year,
            self.context.home_team.id,
        )).map_err(|e| format!("Failed to create directories: {}", e)).unwrap();

        let file_path = format!(
            "data/{}/{}/{}.json",
            self.context.date.year,
            self.context.home_team.id,
            game_pk,
        );

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize game: {}", e)).unwrap();
        std::fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write game to file: {}", e)).unwrap();

        log(format!("[Game::save] Saved game to {}", file_path));
    }

    pub async fn get_all_by_team_in_season(team_id: u8, season: u16, skip_game_pks: Vec<usize>) {
        let url = format!("https://statsapi.mlb.com/api/v1/schedule?sportId=1&teamId={}&season={}", team_id, season);
        let response = reqwest::get(&url).await.unwrap();
        let schedule = response.json::<serde_json::Value>().await.unwrap();
        let dates = schedule["dates"].as_array().unwrap();

        let progress_style = ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} | elapsed: {elapsed_precise}, eta: {eta_precise}").unwrap();
        for date in dates.iter().progress_with_style(progress_style) {
            let games_data = date["games"].as_array().unwrap();
            for game_data in games_data {
                let game_pk = game_data["gamePk"].as_u64().unwrap() as usize;
                if skip_game_pks.contains(&game_pk) {
                    continue;
                }

                match Game::from_game_pk(game_pk).await {
                    Ok(game) => game.save(game_pk),
                    Err(e) => log(format!("[Game::get_all_by_team_in_season] Error: {}", e)),
                };
            }
        }
    }
}

impl Tokenize for Game {
    fn tokenize(&self) -> String {
        let mut tokens = String::new();

        tokens += &format!("[GAME] {}\n[START]\n", self.context.tokenize());
        for play in &self.plays {
            tokens += &format!("{}\n", play.tokenize());
        }
        tokens += "[END]\n";

        tokens
    }
}