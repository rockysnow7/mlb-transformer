use futures::future::join_all;
use indicatif::{ProgressIterator, ProgressStyle};
use serde::{Serialize, Deserialize};
use std::io::Write;

fn indent_spaces(amount: usize) -> String {
    " ".repeat(4 * amount)
}

pub trait Tokenize {
    fn tokenize(&self, indent: usize) -> String;
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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<DATE>", indent_spaces(indent));

        tokens += &format!("\n{}<YEAR>{}</YEAR>", indent_spaces(indent), self.year);
        tokens += &format!("\n{}<MONTH>{}</MONTH>", indent_spaces(indent), self.month);
        tokens += &format!("\n{}<DAY>{}</DAY>", indent_spaces(indent), self.day);

        tokens += &format!("\n{}</DATE>", indent_spaces(indent));

        tokens
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

    // pub fn role(&self) -> PositionRole {
    //     match self {
    //         Position::Pitcher => PositionRole::Pitcher,
    //         Position::Catcher
    //             | Position::FirstBase
    //             | Position::SecondBase
    //             | Position::ThirdBase
    //             | Position::Shortstop
    //             | Position::LeftField
    //             | Position::CenterField
    //             | Position::RightField
    //             | Position::Outfield
    //             | Position::Infield => PositionRole::Fielder,
    //         Position::Hitter => PositionRole::Hitter,
    //         Position::TwoWayPlayer => PositionRole::TwoWayPlayer,
    //     }
    // }
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
    fn tokenize(&self, indent: usize) -> String {
        format!(
            "{}<{}>{}</{}>",
            indent_spaces(indent),
            self.position.to_string(),
            self.name,
            self.position.to_string(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Team {
    pub id: u8,
    players: Vec<Player>,
}

impl Team {
    pub async fn from_boxscore_team_data_and_date(team_data: &serde_json::Value) -> Result<Self, String> {
        // let team_name = team_data["team"]["abbreviation"].as_str().unwrap().to_string();
        let id = team_data["team"]["id"].as_u64().unwrap() as u8;
        let players_data = team_data["players"].as_object().unwrap();

        let mut players = Vec::new();
        for player_data in players_data.values() {
            let player_name = player_data["person"]["fullName"].as_str().unwrap().to_string();
            let position_abbr = player_data["position"]["abbreviation"].as_str().unwrap();
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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<TEAM>", indent_spaces(indent));

        tokens += &format!("\n{}<ID>{}</ID>", indent_spaces(indent + 1), self.id);

        tokens += &format!("\n{}<PLAYERS>", indent_spaces(indent + 1));
        for player in &self.players {
            tokens += &format!("\n{}", player.tokenize(indent + 2));
        }
        tokens += &format!("\n{}</PLAYERS>", indent_spaces(indent + 1));

        tokens += &format!("\n{}</TEAM>", indent_spaces(indent));

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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<WEATHER>", indent_spaces(indent));

        tokens += &format!("\n{}<CONDITION>{}</CONDITION>", indent_spaces(indent + 1), self.condition);
        tokens += &format!("\n{}<TEMPERATURE>{}</TEMPERATURE>", indent_spaces(indent + 1), self.temperature);
        tokens += &format!("\n{}<WIND_SPEED>{}</WIND_SPEED>", indent_spaces(indent + 1), self.wind_speed);

        tokens += &format!("\n{}</WEATHER>", indent_spaces(indent));

        tokens
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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<CONTEXT>", indent_spaces(indent));

        tokens += &format!("\n{}<GAME_PK>{}</GAME_PK>", indent_spaces(indent + 1), self.game_pk);
        tokens += "\n";
        tokens += &self.date.tokenize(indent + 1);
        tokens += &format!("\n{}<VENUE_NAME>{}</VENUE_NAME>", indent_spaces(indent + 1), self.venue_name);
        tokens += "\n";
        tokens += &self.weather.tokenize(indent + 1);
        tokens += "\n";
        tokens += &self.home_team.tokenize(indent + 1);
        tokens += "\n";
        tokens += &self.away_team.tokenize(indent + 1);

        tokens += &format!("\n{}</CONTEXT>", indent_spaces(indent));

        tokens
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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<MOVEMENT>", indent_spaces(indent));

        tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), self.runner);

        tokens += &format!("\n{}<START_BASE>", indent_spaces(indent + 1));
        tokens += match self.start_base {
            Some(base) => base.to_string(),
            None => "null".to_string(),
        }.as_str();
        tokens += "</START_BASE>";

        tokens += &format!("\n{}<END_BASE>", indent_spaces(indent + 1));
        tokens += match self.end_base {
            Some(base) => base.to_string(),
            None => "null".to_string(),
        }.as_str();
        tokens += "</END_BASE>";

        tokens += &format!("\n{}<IS_OUT>{}</IS_OUT>", indent_spaces(indent + 1), self.is_out);

        tokens += &format!("\n{}</MOVEMENT>", indent_spaces(indent));

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
            "Single" => Play::single_from_value(value).await,
            "Double" => Play::double_from_value(value).await,
            "Triple" => Play::triple_from_value(value).await,
            "Home Run" => Play::home_run_from_value(value).await,
            "Walk" => Play::walk_from_value(value).await,
            "Intent Walk" => Play::intent_walk_from_value(value).await,
            "Hit By Pitch" => Play::hit_by_pitch_from_value(value).await,
            "Fielders Choice" => Play::fielders_choice_from_value(value).await,
            "Sac Fly" => Play::sac_fly_from_value(value).await,
            "Sac Fly Double Play" => Play::sac_fly_double_play_from_value(value).await,
            "Sac Bunt" => Play::sac_bunt_from_value(value).await,
            "Field Error" => Play::field_error_from_value(value).await,
            _ => panic!("Unknown play type: {}", play_type),
        }
    }
}

impl Tokenize for Play {
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        match self {
            Play::Groundout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<GROUNDOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</GROUNDOUT>", indent_spaces(indent));
            },
            Play::BuntGroundout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<BUNT_GROUNDOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</BUNT_GROUNDOUT>", indent_spaces(indent));
            },
            Play::Strikeout { batter, pitcher, movements } => {
                tokens += &format!("{}<STRIKEOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</STRIKEOUT>", indent_spaces(indent));
            },
            Play::Lineout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<LINEOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</LINEOUT>", indent_spaces(indent));
            },
            Play::BuntLineout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<BUNT_LINEOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</BUNT_LINEOUT>", indent_spaces(indent));
            },
            Play::Flyout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<FLYOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</FLYOUT>", indent_spaces(indent));
            },
            Play::PopOut { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<POP_OUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</POP_OUT>", indent_spaces(indent));
            },
            Play::BuntPopOut { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<BUNT_POP_OUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</BUNT_POP_OUT>", indent_spaces(indent));
            },
            Play::Forceout { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<FORCEOUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</FORCEOUT>", indent_spaces(indent));
            },
            Play::FieldersChoiceOut { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!("{}<FIELDERS_CHOICE_OUT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));
                tokens += &format!("\n{}<SCORING_RUNNER>{}</SCORING_RUNNER>", indent_spaces(indent + 1), scoring_runner);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</FIELDERS_CHOICE_OUT>", indent_spaces(indent));
            },
            Play::DoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<DOUBLE_PLAY>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</DOUBLE_PLAY>", indent_spaces(indent));
            },
            Play::GroundedIntoDoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<GROUNDED_INTO_DOUBLE_PLAY>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</GROUNDED_INTO_DOUBLE_PLAY>", indent_spaces(indent));
            },
            Play::StrikeoutDoublePlay { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<STRIKEOUT_DOUBLE_PLAY>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</STRIKEOUT_DOUBLE_PLAY>", indent_spaces(indent));
            },
            Play::Pickoff { base, runner, fielders, movements } => {
                tokens += &format!("{}<PICKOFF>", indent_spaces(indent));

                tokens += &format!("\n{}<BASE>{}</BASE>", indent_spaces(indent + 1), base);
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</PICKOFF>", indent_spaces(indent));
            },
            Play::PickoffError { base, runner, fielders, movements } => {
                tokens += &format!("{}<PICKOFF_ERROR>", indent_spaces(indent));

                tokens += &format!("\n{}<BASE>{}</BASE>", indent_spaces(indent + 1), base);
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</PICKOFF_ERROR>", indent_spaces(indent));
            },
            Play::CaughtStealing { base, runner, fielders, movements } => {
                tokens += &format!("{}<CAUGHT_STEALING>", indent_spaces(indent));

                tokens += &format!("\n{}<BASE>{}</BASE>", indent_spaces(indent + 1), base);
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</CAUGHT_STEALING>", indent_spaces(indent));
            },
            Play::PickoffCaughtStealing { base, runner, fielders, movements } => {
                tokens += &format!("{}<PICKOFF_CAUGHT_STEALING>", indent_spaces(indent));

                tokens += &format!("\n{}<BASE>{}</BASE>", indent_spaces(indent + 1), base);
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</PICKOFF_CAUGHT_STEALING>", indent_spaces(indent));
            },
            Play::WildPitch { pitcher, runner, movements } => {
                tokens += &format!("{}<WILD_PITCH>", indent_spaces(indent));

                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</WILD_PITCH>", indent_spaces(indent));
            },
            Play::RunnerOut { runner, fielders, movements } => {
                tokens += &format!("{}<RUNNER_OUT>", indent_spaces(indent));

                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</RUNNER_OUT>", indent_spaces(indent));
            },
            Play::Single { batter, pitcher, movements } => {
                tokens += &format!("{}<SINGLE>", indent_spaces(indent));
                
                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</SINGLE>", indent_spaces(indent));
            },
            Play::Double { batter, pitcher, movements } => {
                tokens += &format!("{}<DOUBLE>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</DOUBLE>", indent_spaces(indent));
            },
            Play::Triple { batter, pitcher, movements } => {
                tokens += &format!("{}<TRIPLE>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</TRIPLE>", indent_spaces(indent));
            },
            Play::HomeRun { batter, pitcher, movements } => {
                tokens += &format!("{}<HOME_RUN>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</HOME_RUN>", indent_spaces(indent));
            },
            Play::Walk { batter, pitcher, movements } => {
                tokens += &format!("{}<WALK>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</WALK>", indent_spaces(indent));
            },
            Play::IntentWalk { batter, pitcher, movements } => {
                tokens += &format!("{}<INTENT_WALK>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</INTENT_WALK>", indent_spaces(indent));
            },
            Play::HitByPitch { batter, pitcher, movements } => {
                tokens += &format!("{}<HIT_BY_PITCH>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</HIT_BY_PITCH>", indent_spaces(indent));
            },
            Play::FieldersChoice { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<FIELDERS_CHOICE>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</FIELDERS_CHOICE>", indent_spaces(indent));
            },
            Play::CatcherInterference { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<CATCHER_INTERFERENCE>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</CATCHER_INTERFERENCE>", indent_spaces(indent));
            },
            Play::SacFly { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!("{}<SAC_FLY>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));
                tokens += &format!("\n{}<SCORING_RUNNER>{}</SCORING_RUNNER>", indent_spaces(indent + 1), scoring_runner);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</SAC_FLY>", indent_spaces(indent));
            },
            Play::SacFlyDoublePlay { batter, pitcher, fielders, scoring_runner, movements } => {
                tokens += &format!("{}<SAC_FLY_DOUBLE_PLAY>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));
                tokens += &format!("\n{}<SCORING_RUNNER>{}</SCORING_RUNNER>", indent_spaces(indent + 1), scoring_runner);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</SAC_FLY_DOUBLE_PLAY>", indent_spaces(indent));
            },
            Play::SacBunt { batter, pitcher, fielders, runner, movements } => {
                tokens += &format!("{}<SAC_BUNT>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));
                tokens += &format!("\n{}<RUNNER>{}</RUNNER>", indent_spaces(indent + 1), runner);

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</SAC_BUNT>", indent_spaces(indent));
            },
            Play::FieldError { batter, pitcher, fielders, movements } => {
                tokens += &format!("{}<FIELD_ERROR>", indent_spaces(indent));

                tokens += &format!("\n{}<BATTER>{}</BATTER>", indent_spaces(indent + 1), batter);
                tokens += &format!("\n{}<PITCHER>{}</PITCHER>", indent_spaces(indent + 1), pitcher);
                tokens += &format!("\n{}<FIELDERS>{}</FIELDERS>", indent_spaces(indent + 1), fielders.join(", "));

                tokens += &format!("\n{}<MOVEMENTS>", indent_spaces(indent + 1));
                for movement in movements {
                    tokens += &format!("\n{}", movement.tokenize(indent + 2));
                }
                tokens += &format!("\n{}</MOVEMENTS>", indent_spaces(indent + 1));

                tokens += &format!("\n{}</FIELD_ERROR>", indent_spaces(indent));
            },
        }

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
    fn tokenize(&self, indent: usize) -> String {
        let mut tokens = String::new();

        tokens += &format!("{}<GAME>", indent_spaces(indent));

        tokens += &format!("\n{}", self.context.tokenize(indent + 1));

        tokens += &format!("\n{}<PLAYS>", indent_spaces(indent + 1));
        for play in &self.plays {
            tokens += &format!("\n{}", play.tokenize(indent + 2));
        }
        tokens += &format!("\n{}</PLAYS>", indent_spaces(indent + 1));

        tokens += &format!("\n{}</GAME>", indent_spaces(indent));

        tokens
    }
}