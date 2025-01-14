from __future__ import annotations
from dataclasses import dataclass
from enum import Enum


@dataclass
class Movement:
    "Information about a movement of a runner from one base to another."

    runner: str
    "The name of the runner moving."
    start_base: int | None
    "The base the runner started at."
    end_base: int | None
    "The base the runner ended at."
    is_out: bool
    "Whether the runner was out at the end of the movement."

    def pretty(self) -> str:
        return f"{self.runner} {self.start_base} -> {self.end_base}{' [out]' if self.is_out else ''}"


class PlayType(Enum):
    "An enum representing the type of play."

    # Outs
    GROUNDOUT = "GROUNDOUT"
    BUNT_GROUNDOUT = "BUNT_GROUNDOUT"
    STRIKEOUT = "STRIKEOUT"
    LINEOUT = "LINEOUT"
    BUNT_LINEOUT = "BUNT_LINEOUT"
    FLYOUT = "FLYOUT"
    POP_OUT = "POP_OUT"
    BUNT_POP_OUT = "BUNT_POP_OUT"
    FORCEOUT = "FORCEOUT"
    FIELDERS_CHOICE_OUT = "FIELDERS_CHOICE_OUT"
    DOUBLE_PLAY = "DOUBLE_PLAY"
    TRIPLE_PLAY = "TRIPLE_PLAY"
    RUNNER_DOUBLE_PLAY = "RUNNER_DOUBLE_PLAY"
    RUNNER_TRIPLE_PLAY = "RUNNER_TRIPLE_PLAY"
    GROUNDED_INTO_DOUBLE_PLAY = "GROUNDED_INTO_DOUBLE_PLAY"
    STRIKEOUT_DOUBLE_PLAY = "STRIKEOUT_DOUBLE_PLAY"
    PICKOFF = "PICKOFF"
    PICKOFF_ERROR = "PICKOFF_ERROR"
    CAUGHT_STEALING = "CAUGHT_STEALING"
    PICKOFF_CAUGHT_STEALING = "PICKOFF_CAUGHT_STEALING"
    WILD_PITCH = "WILD_PITCH"
    RUNNER_OUT = "RUNNER_OUT"
    FIELD_OUT = "FIELD_OUT"
    BALK = "BALK"
    PASSED_BALL = "PASSED_BALL"
    ERROR = "ERROR"

    # Scores
    SINGLE = "SINGLE"
    DOUBLE = "DOUBLE"
    TRIPLE = "TRIPLE"
    HOME_RUN = "HOME_RUN"
    WALK = "WALK"
    INTENT_WALK = "INTENT_WALK"
    HIT_BY_PITCH = "HIT_BY_PITCH"
    FIELDERS_CHOICE = "FIELDERS_CHOICE"
    CATCHER_INTERFERENCE = "CATCHER_INTERFERENCE"
    STOLEN_BASE = "STOLEN_BASE"

    # Other
    SAC_FLY = "SAC_FLY"
    SAC_FLY_DOUBLE_PLAY = "SAC_FLY_DOUBLE_PLAY"
    SAC_BUNT = "SAC_BUNT"
    SAC_BUNT_DOUBLE_PLAY = "SAC_BUNT_DOUBLE_PLAY"
    FIELD_ERROR = "FIELD_ERROR"
    GAME_ADVISORY = "GAME_ADVISORY"

    @staticmethod
    def from_text(text: str) -> PlayType:
        return PlayType(text.replace(" ", "_").upper())


@dataclass
class PlayContents:
    "Information about a play."

    batter: str | None = None
    "The name of the batter in the play."
    pitcher: str | None = None
    "The name of the pitcher in the play."
    fielders: list[str] | None = None
    "The names of the fielders involved in the play."
    catcher: str | None = None
    "The name of the catcher in the play."
    runner: str | None = None
    "The name of the runner in the play."
    scoring_runner: str | None = None
    "The name of the runner who scored in the play."
    base: int | None = None
    "The number of the base involved in the play."
    movements: list[Movement] | None = None
    "A list of all movements that occurred in the play."

    def players(self) -> list[str]:
        players = []
        if self.batter:
            players.append(self.batter)
        if self.pitcher:
            players.append(self.pitcher)
        if self.fielders:
            players.extend(self.fielders)
        if self.catcher:
            players.append(self.catcher)
        if self.runner:
            players.append(self.runner)
        if self.scoring_runner:
            players.append(self.scoring_runner)
        return players


@dataclass
class Play:
    play_type: PlayType
    "The type of play."
    contents: PlayContents
    "Information about the play."


class Position(Enum):
    "An enum representing a position on the baseball field."

    PITCHER = "PITCHER"
    CATCHER = "CATCHER"
    FIRST_BASE = "FIRST_BASE"
    SECOND_BASE = "SECOND_BASE"
    THIRD_BASE = "THIRD_BASE"
    SHORTSTOP = "SHORTSTOP"
    LEFT_FIELD = "LEFT_FIELD"
    CENTER_FIELD = "CENTER_FIELD"
    RIGHT_FIELD = "RIGHT_FIELD"
    DESIGNATED_HITTER = "DESIGNATED_HITTER"
    PINCH_HITTER = "PINCH_HITTER"
    PINCH_RUNNER = "PINCH_RUNNER"
    TWO_WAY_PLAYER = "TWO_WAY_PLAYER"
    OUTFIELD = "OUTFIELD"
    INFIELD = "INFIELD"
    UTILITY = "UTILITY"
    RELIEF_PITCHER = "RELIEF_PITCHER"
    STARTING_PITCHER = "STARTING_PITCHER"


@dataclass
class Player:
    name: str
    "The name of the player."
    position: Position
    "The position of the player on the field."


@dataclass
class Team:
    id: int
    "The unique identifier for the team in the MLB API."
    players: list[Player]
    "A list of all players on the team."


@dataclass
class Weather:
    "Information about the weather."

    condition: str
    "A description of the weather condition."
    temperature: int
    "The temperature in degrees Fahrenheit."
    wind_speed: int
    "The wind speed in miles per hour."


@dataclass
class GameContext:
    game_pk: int
    "The unique identifier for the game in the MLB API."
    date: str
    "The date on which the game was played."
    venue: str
    "The name of the venue in which the game was played."
    weather: Weather
    "The weather conditions at the time of the game."
    home_team: Team
    away_team: Team


@dataclass
class Game:
    "A baseball game."

    context: GameContext
    plays: list[Play]
    "A list of all plays in the game."
