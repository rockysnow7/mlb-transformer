from __future__ import annotations
from dataclasses import dataclass
from enum import Enum

import re
import sys


@dataclass
class Movement:
    runner: str
    start_base: int | None
    end_base: int | None
    is_out: bool

    @staticmethod
    def from_tokens(tokens: list[str]) -> Movement:
        name_tokens = []
        while tokens[1] != "->":
            name_tokens.append(tokens.pop(0))
        runner = " ".join(name_tokens)

        start_base_token = tokens.pop(0)
        if start_base_token == "home":
            start_base = 4
        else:
            start_base = int(start_base_token)

        tokens.pop(0)

        end_base_token = tokens.pop(0)
        if end_base_token == "home":
            end_base = 4
        else:
            end_base = int(end_base_token)

        if tokens[0] == "[out]":
            is_out = True
            tokens.pop(0)
        else:
            is_out = False

        return Movement(runner, start_base, end_base, is_out)


class PlayType(Enum):
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


ALL_PLAY_CONTENT_TOKENS = [
    "[BATTER]",
    "[PITCHER]",
    "[FIELDERS]",
    "[CATCHER]",
    "[RUNNER]",
    "[SCORING_RUNNER]",
    "[BASE]",
    "[MOVEMENTS]",
]


@dataclass
class PlayContents:
    batter: str | None = None
    pitcher: str | None = None
    fielders: list[str] | None = None
    catcher: str | None = None
    runner: str | None = None
    scoring_runner: str | None = None
    base: int | None = None
    movements: list[Movement] | None = None

    @staticmethod
    def from_tokens(tokens: list[str]) -> PlayContents:
        context_tokens = []
        while tokens[0] != "[MOVEMENTS]":
            context_tokens.append(tokens.pop(0))

        contents = PlayContents()
        while context_tokens:
            token = context_tokens.pop(0)
            match token:
                case "[BATTER]":
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.batter = " ".join(name_tokens)
                case "[PITCHER]":
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.pitcher = " ".join(name_tokens)
                case "[FIELDERS]":  # all "fielders" fields have length 1 currently, so we don't need to handle multiple names
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.fielders = [" ".join(name_tokens)]
                case "[CATCHER]":
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.catcher = " ".join(name_tokens)
                case "[RUNNER]":
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.runner = " ".join(name_tokens)
                case "[SCORING_RUNNER]":
                    name_tokens = []
                    while context_tokens and context_tokens[0] not in ALL_PLAY_CONTENT_TOKENS and context_tokens[0] != "[MOVEMENTS]":
                        name_tokens.append(context_tokens.pop(0))
                    contents.scoring_runner = " ".join(name_tokens)
                case "[BASE]":
                    base_token = context_tokens.pop(0)
                    if base_token == "home":
                        contents.base = 4
                    else:
                        contents.base = int(base_token)

        movements_token = tokens.pop(0)
        if movements_token != "[MOVEMENTS]":
            raise ValueError(f"Expected token [MOVEMENTS], got '{movements_token}'")

        movements = []
        while tokens[0] not in ["[PLAY]", "[GAME_END]"]:
            movement = Movement.from_tokens(tokens)
            movements.append(movement)
        contents.movements = movements

        return contents


@dataclass
class Play:
    play_type: PlayType
    contents: PlayContents

    @staticmethod
    def from_tokens(tokens: list[str]) -> Play:
        play_token = tokens.pop(0)
        if play_token != "[PLAY]":
            raise ValueError(f"Expected token [PLAY], got '{play_token}'")

        play_type_tokens = []
        while tokens[0] not in ALL_PLAY_CONTENT_TOKENS and tokens[0] != "[GAME_END]":
            play_type_tokens.append(tokens.pop(0))
        play_type = PlayType.from_text(" ".join(play_type_tokens))

        if play_type == PlayType.GAME_ADVISORY:
            contents = PlayContents()
        else:
            contents = PlayContents.from_tokens(tokens)

        return Play(play_type, contents)


class Position(Enum):
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


ALL_POSITIONS = [position.value for position in Position]
ALL_POSITION_TOKENS = [f"[{position}]" for position in ALL_POSITIONS]


@dataclass
class Player:
    name: str
    position: Position

    @staticmethod
    def from_tokens(tokens: list[str]) -> Player:
        position_token = tokens.pop(0)
        if position_token not in ALL_POSITION_TOKENS:
            raise ValueError(f"Expected position token, got '{position_token}'")
        position = Position(position_token[1:-1])

        name_tokens = []
        while tokens[0] not in ALL_POSITION_TOKENS and tokens[0] != "[TEAM]" and tokens[0] != "[GAME_START]":
            name_tokens.append(tokens.pop(0))
        name = " ".join(name_tokens)

        return Player(name, position)


@dataclass
class Team:
    id: int
    players: list[Player]

    @staticmethod
    def from_tokens(tokens: list[str]) -> Team:
        team_token = tokens.pop(0)
        if team_token != "[TEAM]":
            raise ValueError(f"Expected token [TEAM], got '{team_token}'")
        team_id = int(tokens.pop(0))

        players = []
        while tokens[0] != "[TEAM]" and tokens[0] != "[GAME_START]":
            player = Player.from_tokens(tokens)
            players.append(player)

        return Team(team_id, players)


@dataclass
class Weather:
    condition: str
    temperature: int
    wind_speed: int

    @staticmethod
    def from_tokens(tokens: list[str]) -> Weather:
        weather_token = tokens.pop(0)
        if weather_token != "[WEATHER]":
            raise ValueError(f"Expected token [WEATHER], got '{weather_token}'")

        condition_tokens = []
        while not re.match(r"\d+", tokens[0]):
            condition_tokens.append(tokens.pop(0))
        condition = " ".join(condition_tokens)

        temperature = int(tokens.pop(0))
        wind_speed = int(tokens.pop(0))

        return Weather(condition, temperature, wind_speed)


@dataclass
class GameContext:
    game_pk: int
    date: str
    venue: str
    weather: Weather
    home_team: Team
    away_team: Team

    @staticmethod
    def from_tokens(tokens: list[str]) -> GameContext:
        game_token = tokens.pop(0)
        if game_token != "[GAME]":
            raise ValueError(f"Expected token [GAME], got '{game_token}'")
        game_pk = int(tokens.pop(0))

        date_token = tokens.pop(0)
        if date_token != "[DATE]":
            raise ValueError(f"Expected token [DATE], got '{date_token}'")
        date = tokens.pop(0)

        venue_token = tokens.pop(0)
        if venue_token != "[VENUE]":
            raise ValueError(f"Expected token [VENUE], got '{venue_token}'")
        
        venue_tokens = []
        while tokens[0] != "[WEATHER]":
            venue_tokens.append(tokens.pop(0))
        venue = " ".join(venue_tokens)

        weather = Weather.from_tokens(tokens)

        home_team = Team.from_tokens(tokens)
        away_team = Team.from_tokens(tokens)

        return GameContext(game_pk, date, venue, weather, home_team, away_team)


@dataclass
class Game:
    context: GameContext
    plays: list[Play]

    @staticmethod
    def from_tokens(tokens: list[str]) -> Game:
        context = GameContext.from_tokens(tokens)

        game_start_token = tokens.pop(0)
        if game_start_token != "[GAME_START]":
            raise ValueError(f"Expected token [GAME_START], got '{game_start_token}'")

        plays = []
        while tokens[0] != "[GAME_END]":
            play = Play.from_tokens(tokens)
            plays.append(play)

        return Game(context, plays)


def parse_game(text: str) -> Game:
    tokens = re.split(r"[\s,]+", text)

    return Game.from_tokens(tokens)


if __name__ == "__main__":
    path = sys.argv[1]

    with open(path) as f:
        text = f.read()

    try:
        parse_game(text)
    except Exception as e:
        print(f"Error parsing game: {e}")
        sys.exit(1)