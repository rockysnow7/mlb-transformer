use indicatif::{ProgressIterator, ProgressStyle};
use serde::{Serialize, Deserialize};
use std::io::Write;

pub trait Preprocess {
    /// Returns a JSON string representing the object.
    fn preprocess(&self) -> String;
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

async fn get_player_name_from_id(player_id: usize) -> Result<String, String> {
    let url = format!("https://statsapi.mlb.com/api/v1/people/{player_id}");
    let response = if let Ok(response) = reqwest::get(&url).await {
        response
    } else {
        return Err("Failed to get player data".to_string());
    };
    let player_data = if let Ok(player_data) = response.json::<serde_json::Value>().await {
        player_data
    } else {
        return Err("Failed to parse player data".to_string());
    };
    let player_name = player_data["people"][0]["fullName"].as_str().unwrap().to_string();

    Ok(player_name)
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
            "P" | "LHP" | "RHP" => Position::Pitcher,
            "C" => Position::Catcher,
            "1B" => Position::FirstBase,
            "2B" => Position::SecondBase,
            "3B" => Position::ThirdBase,
            "SS" => Position::Shortstop,
            "LF" => Position::LeftField,
            "CF" => Position::CenterField,
            "RF" => Position::RightField,
            "DH" | "EH" => Position::DesignatedHitter,
            "PH" => Position::PinchHitter,
            "PR" => Position::PinchRunner,
            "TWP" => Position::TwoWayPlayer,
            "OF" => Position::Outfield,
            "IF" => Position::Infield,
            "UT" | "UTIL" => Position::Utility,
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

impl Preprocess for Player {
    fn preprocess(&self) -> String {
        // format!("[{}] {}", self.position.to_string(), self.name)
        format!("{{ \"position\": \"{}\", \"name\": \"{}\" }}", self.position.to_string(), self.name)
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

impl Preprocess for Team {
    fn preprocess(&self) -> String {
        // let mut tokens = String::new();

        // tokens += &format!("[TEAM] {}\n", self.id);
        // for player in &self.players {
        //     tokens += &format!("{}\n", player.preprocess());
        // }

        // tokens

        format!(
            "{{ \"id\": {}, \"players\": [{}] }}",
            self.id,
            self.players.iter().map(|player| player.preprocess()).collect::<Vec<String>>().join(", "),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    condition: String,
    temperature: u8,
    wind_speed: u8,
}

impl Weather {
    pub fn from_value(value: &serde_json::Value) -> Result<Self, String> {
        let condition = value["condition"].as_str().unwrap().to_string();
        let temperature = value["temp"].as_str().unwrap().parse().unwrap();
        let wind_speed = value["wind"]
            .as_str();

        let wind_speed = if let Some(wind_speed) = wind_speed {
            wind_speed.to_string()
                .split(' ')
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .parse()
                .unwrap()
        } else {
            return Err("No wind speed".to_string());
        };

        Ok(Self {
            condition,
            temperature,
            wind_speed,
        })
    }
}

impl Preprocess for Weather {
    fn preprocess(&self) -> String {
        // format!("[WEATHER] {} {} {}", self.condition, self.temperature, self.wind_speed)
        format!(
            "{{ \"condition\": \"{}\", \"temperature\": {}, \"wind_speed\": {} }}",
            self.condition,
            self.temperature,
            self.wind_speed,
        )
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

impl Preprocess for GameContext {
    fn preprocess(&self) -> String {
        // format!(
        //     "{} [DATE] {} [VENUE] {} {}\n\n{}\n{}",
        //     self.game_pk,
        //     self.date.to_string(),
        //     self.venue_name,
        //     self.weather.preprocess(),
        //     self.home_team.preprocess(),
        //     self.away_team.preprocess(),
        // )
        format!(
            "{{ \"game_pk\": {}, \"date\": \"{}\", \"venue_name\": \"{}\", \"weather\": {}, \"home_team\": {}, \"away_team\": {} }}",
            self.game_pk,
            self.date.to_string(),
            self.venue_name,
            self.weather.preprocess(),
            self.home_team.preprocess(),
            self.away_team.preprocess(),
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

impl Preprocess for Movement {
    fn preprocess(&self) -> String {
        // let mut tokens = String::new();

        // tokens += &format!("{} ", self.runner);

        // let start_base_str = match self.start_base {
        //     Some(base) => base.to_string(),
        //     None => "home".to_string(),
        // };
        // let end_base_str = match self.end_base {
        //     Some(base) => base.to_string(),
        //     None => "home".to_string(),
        // };

        // tokens += &format!("{} -> {}", start_base_str, end_base_str);

        // if self.is_out {
        //     tokens += " [out]";
        // }

        // tokens

        format!(
            "{{ \"runner\": \"{}\", \"start_base\": {}, \"end_base\": {}, \"is_out\": {} }}",
            self.runner,
            self.start_base.map_or("\"home\"".to_string(), |base| format!("\"{base}\"")),
            self.end_base.map_or("\"home\"".to_string(), |base| format!("\"{base}\"")),
            self.is_out,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inning {
    pub number: u8,
    pub top: bool,
}

impl Inning {
    pub fn from_value(value: &serde_json::Value) -> Self {
        let number = value["inning"].as_u64().unwrap() as u8;
        let top = value["isTopInning"].as_bool().unwrap();

        Self { number, top }
    }
}

impl ToString for Inning {
    fn to_string(&self) -> String {
        format!("{} {}", self.number, if self.top { "top" } else { "bottom" })
    }
}

impl Preprocess for Inning {
    fn preprocess(&self) -> String {
        format!("{{ \"number\": {}, \"top\": {} }}", self.number, self.top)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Play {
    // outs
    Groundout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntGroundout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Strikeout {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Lineout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntLineout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Flyout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PopOut {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    BuntPopOut {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Forceout {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    FieldersChoiceOut {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    DoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    TriplePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    RunnerDoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    RunnerTriplePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    GroundedIntoDoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    StrikeoutDoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    Pickoff {
        inning: Inning,
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PickoffError {
        inning: Inning,
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    CaughtStealing {
        inning: Inning,
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    PickoffCaughtStealing {
        inning: Inning,
        base: u8,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    WildPitch {
        inning: Inning,
        pitcher: String,
        runner: String,
        movements: Vec<Movement>,
    },
    RunnerOut {
        inning: Inning,
        runner: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    FieldOut {
        inning: Inning,
        fielder: String,
        runner: String,
        movements: Vec<Movement>,
    },
    BatterOut {
        inning: Inning,
        batter: String,
        catcher: String,
        movements: Vec<Movement>,
    },
    Balk {
        inning: Inning,
        pitcher: String,
        movements: Vec<Movement>,
    },
    PassedBall {
        inning: Inning,
        pitcher: String,
        catcher: String,
        movements: Vec<Movement>,
    },
    Error {
        inning: Inning,
        pitcher: String,
        catcher: String,
        movements: Vec<Movement>,
    },
    // scores
    Single {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Double {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Triple {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    HomeRun {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    Walk {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    IntentWalk {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    HitByPitch {
        inning: Inning,
        batter: String,
        pitcher: String,
        movements: Vec<Movement>,
    },
    FieldersChoice {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    CatcherInterference {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    StolenBase {
        inning: Inning,
        base: u8,
        runner: String,
        movements: Vec<Movement>,
    },
    // other
    SacFly {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    SacFlyDoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        scoring_runner: String,
        movements: Vec<Movement>,
    },
    SacBunt {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        runner: String,
        movements: Vec<Movement>,
    },
    SacBuntDoublePlay {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        runner: String,
        movements: Vec<Movement>,
    },
    FieldError {
        inning: Inning,
        batter: String,
        pitcher: String,
        fielders: Vec<String>,
        movements: Vec<Movement>,
    },
    GameAdvisory {
        inning: Inning,
    },
    Ejection {
        inning: Inning,
        movements: Vec<Movement>,
    }
}

impl Play {
    // outs
    async fn groundout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Groundout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_groundout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntGroundout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn strikeout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn lineout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Lineout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_lineout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntLineout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn flyout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Flyout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn pop_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PopOut {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn bunt_pop_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BuntPopOut {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn forceout_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Forceout {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn fielders_choice_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let scoring_runner = match value["runners"][1]["details"]["runner"]["fullName"].as_str() {
            Some(runner) => runner.to_string(),
            None => return Err("No scoring runner".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldersChoiceOut {
            inning,
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::DoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn triple_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::TriplePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn runner_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::RunnerDoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn runner_triple_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::TriplePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn grounded_into_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::GroundedIntoDoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn strikeout_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::StrikeoutDoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn pickoff_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Pickoff {
            inning,
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn pickoff_error_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PickoffError {
            inning,
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn caught_stealing_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::CaughtStealing {
            inning,
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn pickoff_caught_stealing_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::PickoffCaughtStealing {
            inning,
            base,
            runner,
            fielders,
            movements,
        })
    }

    async fn wild_pitch_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            pitcher,
            runner,
            movements,
        })
    }

    async fn runner_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let fielder_ids = value["runners"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|runner| runner["credits"][0]["player"]["id"].as_u64())
            .map(|id| id as usize);
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::RunnerOut {
            inning,
            runner,
            fielders,
            movements,
        })
    }

    async fn field_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            fielder,
            runner,
            movements,
        })
    }

    async fn batter_out_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let batter = match value["matchup"]["batter"]["fullName"].as_str() {
            Some(batter) => batter.to_string(),
            None => return Err("No batter".to_string()),
        };
        let catcher_id = value["runners"][0]["credits"][0]["player"]["id"].as_u64().unwrap() as usize;
        let catcher = get_player_name_from_id(catcher_id).await?;
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::BatterOut {
            inning,
            batter,
            catcher,
            movements,
        })
    }

    async fn balk_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = vec![Movement::from_runner_and_value(
            runner.clone(),
            &value["runners"][0]["movement"],
        )];

        Ok(Play::Balk {
            inning,
            pitcher,
            movements,
        })
    }

    async fn passed_ball_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let pitcher = match value["runners"][0]["details"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let catcher = match value["runners"][0]["details"]["fielder"]["fullName"].as_str() {
            Some(catcher) => catcher.to_string(),
            None => return Err("No catcher".to_string()),
        };
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = vec![Movement::from_runner_and_value(
            runner.clone(),
            &value["runners"][0]["movement"],
        )];

        Ok(Play::PassedBall {
            inning,
            pitcher,
            catcher,
            movements,
        })
    }

    async fn error_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let pitcher = match value["matchup"]["pitcher"]["fullName"].as_str() {
            Some(pitcher) => pitcher.to_string(),
            None => return Err("No pitcher".to_string()),
        };
        let catcher = match value["matchup"]["catcher"]["fullName"].as_str() {
            Some(catcher) => catcher.to_string(),
            None => return Err("No catcher".to_string()),
        };
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Error {
            inning,
            pitcher,
            catcher,
            movements,
        })
    }

    // scores
    async fn single_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn double_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn triple_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn home_run_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn walk_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn intent_walk_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn hit_by_pitch_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
            inning,
            batter,
            pitcher,
            movements,
        })
    }

    async fn fielders_choice_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldersChoice {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn catcher_interference_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::CatcherInterference {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn stolen_base_from_value_and_base(value: &serde_json::Value, base: u8) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let runner = value["runners"][0]["details"]["runner"]["fullName"].as_str().unwrap().to_string();
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::StolenBase {
            inning,
            base,
            runner,
            movements,
        })
    }

    // other
    async fn sac_fly_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let scoring_runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacFly {
            inning,
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn sac_fly_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let scoring_runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacFlyDoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            scoring_runner,
            movements,
        })
    }

    async fn sac_bunt_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacBunt {
            inning,
            batter,
            pitcher,
            fielders,
            runner,
            movements,
        })
    }

    async fn sac_bunt_double_play_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }
        let runner = value["runners"][1]["details"]["runner"]["fullName"].as_str().unwrap().to_string();

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::SacBuntDoublePlay {
            inning,
            batter,
            pitcher,
            fielders,
            runner,
            movements,
        })
    }

    async fn field_error_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
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
        let mut fielders = Vec::new();
        for id in fielder_ids {
            fielders.push(get_player_name_from_id(id).await?);
        }

        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::FieldError {
            inning,
            batter,
            pitcher,
            fielders,
            movements,
        })
    }

    async fn game_advistory_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);

        Ok(Play::GameAdvisory {
            inning,
        })
    }

    async fn ejection_from_value(value: &serde_json::Value) -> Result<Self, String> {
        let inning = Inning::from_value(&value["about"]);
        let movements = value["runners"].as_array().unwrap().iter().map(|runner| Movement::from_runner_and_value(
            runner["details"]["runner"]["fullName"].as_str().unwrap().to_string(),
            &runner["movement"],
        )).collect();

        Ok(Play::Ejection {
            inning,
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
            "Runner Double Play" => Play::runner_double_play_from_value(value).await,
            "Runner Triple Play" => Play::runner_triple_play_from_value(value).await,
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
            "Batter Out" => Play::batter_out_from_value(value).await,
            "Balk" => Play::balk_from_value(value).await,
            "Passed Ball" => Play::passed_ball_from_value(value).await,
            "Error" => Play::error_from_value(value).await,
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
            "Stolen Base Home" => Play::stolen_base_from_value_and_base(value, 4).await,
            "Sac Fly" => Play::sac_fly_from_value(value).await,
            "Sac Fly Double Play" => Play::sac_fly_double_play_from_value(value).await,
            "Sac Bunt" => Play::sac_bunt_from_value(value).await,
            "Sac Bunt Double Play" => Play::sac_bunt_double_play_from_value(value).await,
            "Field Error" => Play::field_error_from_value(value).await,
            "Game Advisory" => Play::game_advistory_from_value(value).await,
            "Ejection" => Play::ejection_from_value(value).await,
            _ => panic!("Unknown play type: {}", play_type),
        }
    }
}

impl Preprocess for Play {
    fn preprocess(&self) -> String {
        match self {
            Play::Groundout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Groundout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::BuntGroundout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Bunt Groundout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Strikeout { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Strikeout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Lineout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Lineout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::BuntLineout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Bunt Lineout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Flyout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Flyout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::PopOut { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Pop Out\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::BuntPopOut { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Bunt Pop Out\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Forceout { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Forceout\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::FieldersChoiceOut { inning, batter, pitcher, fielders, scoring_runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Fielders Choice Out\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"scoring_runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    scoring_runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::DoublePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::TriplePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Triple Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::RunnerDoublePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Runner Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::RunnerTriplePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Runner Triple Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::GroundedIntoDoublePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Grounded Into Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::StrikeoutDoublePlay { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Strikeout Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Pickoff { inning, base, runner, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Pickoff\" }}\n{{ \"base\": {}, \"runner\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    base,
                    runner,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::PickoffError { inning, base, runner, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Pickoff Error\" }}\n{{ \"base\": {}, \"runner\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    base,
                    runner,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::CaughtStealing { inning, base, runner, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Caught Stealing\" }}\n{{ \"base\": {}, \"runner\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    base,
                    runner,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::PickoffCaughtStealing { inning, base, runner, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Pickoff Caught Stealing\" }}\n{{ \"base\": {}, \"runner\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    base,
                    runner,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::WildPitch { inning, pitcher, runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Wild Pitch\" }}\n{{ \"pitcher\": \"{}\", \"runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    pitcher,
                    runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::RunnerOut { inning, runner, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Runner Out\" }}\n{{ \"runner\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    runner,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::FieldOut { inning, fielder, runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Field Out\" }}\n{{ \"fielder\": \"{}\", \"runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    fielder,
                    runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::BatterOut { inning, batter, catcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Batter Out\" }}\n{{ \"batter\": \"{}\", \"catcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    catcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Balk { inning, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Balk\" }}\n{{ \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::PassedBall { inning, pitcher, catcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Passed Ball\" }}\n{{ \"pitcher\": \"{}\", \"catcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    pitcher,
                    catcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Error { inning, pitcher, catcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Error\" }}\n{{ \"pitcher\": \"{}\", \"catcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    pitcher,
                    catcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Single { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Single\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Double { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Double\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Triple { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Triple\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::HomeRun { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Home Run\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::Walk { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Walk\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::IntentWalk { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Intent Walk\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::HitByPitch { inning, batter, pitcher, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Hit By Pitch\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::FieldersChoice { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Fielders Choice\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::CatcherInterference { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Catcher Interference\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::StolenBase { inning, base, runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Stolen Base\" }}\n{{ \"base\": {}, \"runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    base,
                    runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::SacFly { inning, batter, pitcher, fielders, scoring_runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Sac Fly\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"scoring_runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    scoring_runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::SacFlyDoublePlay { inning, batter, pitcher, fielders, scoring_runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Sac Fly Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"scoring_runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    scoring_runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::SacBunt { inning, batter, pitcher, fielders, runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Sac Bunt\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::SacBuntDoublePlay { inning, batter, pitcher, fielders, runner, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Sac Bunt Double Play\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"runner\": \"{}\", \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    runner,
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::FieldError { inning, batter, pitcher, fielders, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Field Error\" }}\n{{ \"batter\": \"{}\", \"pitcher\": \"{}\", \"fielders\": [{}], \"movements\": [{}] }}",
                    inning.preprocess(),
                    batter,
                    pitcher,
                    fielders.iter().map(|fielder| format!("\"{fielder}\"")).collect::<Vec<String>>().join(", "),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
            Play::GameAdvisory { inning } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Game Advisory\" }}",
                    inning.preprocess(),
                )
            }
            Play::Ejection { inning, movements } => {
                format!(
                    "{{ \"inning\": {}, \"type\": \"Ejection\" }}\n{{ \"movements\": [{}] }}",
                    inning.preprocess(),
                    movements.iter().map(|movement| movement.preprocess()).collect::<Vec<String>>().join(", "),
                )
            }
        }
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
        let response = match reqwest::get(&url).await {
            Ok(response) => response,
            Err(_) => return Err("Failed to fetch game data".to_string()),
        };
        let game_data = response.json::<serde_json::Value>().await.unwrap();

        let game_status = game_data["gameData"]["status"]["detailedState"].as_str();
        if let Some("Final") = game_status {} else {
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
        let boxscore_response = match reqwest::get(&boxscore_data_url).await {
            Ok(response) => response,
            Err(_) => return Err("Failed to fetch boxscore data".to_string()),
        };
        let boxscore_data = boxscore_response.json::<serde_json::Value>().await.unwrap();
        let context = GameContext::from_game_boxscore_data_and_date_and_weather_and_game_pk(
            &boxscore_data,
            game_date,
            weather?,
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

    pub async fn get_all_by_team_in_season(team_id: u8, season: u16, skip_game_pks: Vec<usize>) -> Result<(), String> {
        let url = format!("https://statsapi.mlb.com/api/v1/schedule?sportId=1&teamId={}&season={}", team_id, season);
        let response = match reqwest::get(&url).await {
            Ok(response) => response,
            Err(_) => return Err("Failed to fetch team data".to_string()),
        };
        let schedule = response.json::<serde_json::Value>().await.unwrap();
        let dates = schedule["dates"].as_array().unwrap();

        let progress_style = ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} | elapsed: {elapsed_precise}, eta: {eta_precise}").unwrap();
        for date in dates.iter().progress_with_style(progress_style) {
            let games_data = date["games"].as_array().unwrap();
            for game_data in games_data {
                let game_pk = game_data["gamePk"].as_u64().unwrap() as usize;
                if skip_game_pks.contains(&game_pk) {
                    log(format!("[Game::get_all_by_team_in_season] Skipping game {}", game_pk));
                    continue;
                }

                match Game::from_game_pk(game_pk).await {
                    Ok(game) => game.save(game_pk),
                    Err(e) => log(format!("[Game::get_all_by_team_in_season] Error: {}", e)),
                };
            }
        }

        Ok(())
    }
}

impl Preprocess for Game {
    fn preprocess(&self) -> String {
        // let mut tokens = String::new();

        // tokens += &format!("[GAME] {}\n[GAME_START]\n", self.context.preprocess());
        // for play in &self.plays {
        //     tokens += &format!("{}\n", play.preprocess());
        // }
        // tokens += "[GAME_END]\n";

        // tokens

        // format!(
        //     "{{ \"context\": {}, \"plays\": [{}] }}",
        //     self.context.preprocess(),
        //     self.plays.iter().map(|play| play.preprocess()).collect::<Vec<String>>().join(", "),
        // )
        format!(
            "{}\n{}\n",
            self.context.preprocess(),
            self.plays.iter().map(|play| play.preprocess()).collect::<Vec<String>>().join("\n"),
        )
    }
}
