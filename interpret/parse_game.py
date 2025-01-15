from game import Movement, PlayType, PlayContents, Play, Position, Player, Team, Weather, GameContext, Game
from glob import glob
from tqdm import tqdm

import re
import sys


ALL_POSITION_TOKENS = (
    "[PITCHER]",
    "[CATCHER]",
    "[FIRST_BASE]",
    "[SECOND_BASE]",
    "[THIRD_BASE]",
    "[SHORTSTOP]",
    "[LEFT_FIELD]",
    "[CENTER_FIELD]",
    "[RIGHT_FIELD]",
    "[DESIGNATED_HITTER]",
    "[PINCH_HITTER]",
    "[PINCH_RUNNER]",
    "[TWO_WAY_PLAYER]",
    "[OUTFIELD]",
    "[INFIELD]",
    "[UTILITY]",
    "[RELIEF_PITCHER]",
    "[STARTING_PITCHER]",
)
ALL_POSITION_TOKENS_PATTERN = "|".join(ALL_POSITION_TOKENS).replace("[", r"\[").replace("]", r"\]")


class Parser:
    def __match(self, text: str, pattern: str) -> tuple[str, str]:
        """Returns the first token that matches the pattern, and the remaining text."""

        try:
            token = re.match(f"^{pattern}", text).group(0)

            return token.strip(), text[len(token):].strip()
        except AttributeError:
            error_text = text
            if len(text) > 20:
                error_text = text[:20] + "..."
            raise ValueError(f"Expected '{pattern}' but got '{error_text}'")

    def __skip(self, text: str, pattern: str) -> str:
        """Skips a pattern in the text and returns the remaining text."""

        _, remaining_text = self.__match(text, pattern)

        return remaining_text
    
    def __match_until(self, text: str, pattern: str) -> tuple[str, str]:
        """Returns the text until the pattern is matched, and the remaining text."""

        try:
            total_pattern = f"^.+?(?={pattern})"
            token = re.match(total_pattern, text, re.DOTALL).group(0)

            return token.strip(), text[len(token):].strip()
        except AttributeError:
            error_text = text
            if len(text) > 20:
                error_text = text[:20] + "..."
            raise ValueError(f"Expected eventual '{pattern}' but got '{error_text}'")

    def __parse_player(self, text: str) -> tuple[Player, str]:
        """Parses a single player from a team section and returns a Player object and the remaining text."""

        player = Player(None, None)

        position_token, text = self.__match(text, ALL_POSITION_TOKENS_PATTERN)
        position_name = position_token[1:-1]
        position = Position(position_name)
        player.position = position

        name, text = self.__match_until(text, r"\[")
        player.name = name

        return player, text

    def __parse_team(self, text: str) -> tuple[Team, str]:
        """Parses a team section of the game and returns a Team object and the remaining text."""

        team = Team(None, None)

        text = self.__skip(text, r"\[TEAM\]")
        id_, text = self.__match(text, r"\d+")
        team.id = int(id_)

        players = []
        while not text.startswith("[TEAM]") and not text.startswith("[GAME_START]"):
            player, text = self.__parse_player(text)
            players.append(player)

        team.players = players

        return team, text

    def __parse_context(self, text: str) -> tuple[GameContext, str]:
        """Parses the context section of the game and returns a GameContext object and the remaining text."""

        context = GameContext(
            game_pk=None,
            date=None,
            venue=None,
            weather=None,
            home_team=None,
            away_team=None,
        )

        text = self.__skip(text, r"\[GAME\]")
        game_pk, text = self.__match(text, r"\d+")
        context.game_pk = int(game_pk)

        text = self.__skip(text, r"\[DATE\]")
        date, text = self.__match(text, r"\d{4}-\d{2}-\d{2}")
        context.date = date

        text = self.__skip(text, r"\[VENUE\]")
        venue, text = self.__match_until(text, r"\[WEATHER\]")
        context.venue = venue

        text = self.__skip(text, r"\[WEATHER\]")
        condition, text = self.__match_until(text, r"\d+")
        temperature, text = self.__match(text, r"\d+")
        wind_speed, text = self.__match(text, r"\d+")
        context.weather = Weather(condition, int(temperature), int(wind_speed))

        home_team, text = self.__parse_team(text)
        context.home_team = home_team

        away_team, text = self.__parse_team(text)
        context.away_team = away_team

        return context, text
    
    def __parse_play_contents_player(self, text: str, expected_position_tag: str) -> tuple[str, str]:
        """Parses a player from a play contents section with an expected position tag and returns the player name and the remaining text."""

        text = self.__skip(text, expected_position_tag)
        name, text = self.__match_until(text, r"\[")

        return name, text

    def __parse_play_contents_fielders(self, text: str) -> tuple[list[str], str]:
        """Parses the fielders from a play contents section and returns a list of fielder names and the remaining text."""

        text = self.__skip(text, r"\[FIELDERS\]")
        if text.startswith("["):
            return [], text

        fielders, text = self.__match_until(text, r"\[")
        fielders = fielders.split(", ")
        fielders = [fielder for fielder in fielders if fielder.strip()]

        return fielders, text

    def __parse_movement(self, text: str) -> Movement:
        """Parses a movement from a single movement text and returns a Movement object."""

        movement = Movement(None, None, None, None)

        movement.runner, text = self.__match_until(text, r"home|\d")
        start_base, text = self.__match(text, r"home|\d")
        if start_base == "home":
            movement.start_base = 4
        else:
            movement.start_base = int(start_base)

        text = self.__skip(text, r"->")

        end_base, text = self.__match(text, r"home|\d")
        if end_base == "home":
            movement.end_base = 4
        else:
            movement.end_base = int(end_base)

        movement.is_out = text == "[out]"

        return movement

    def __parse_play_contents_movements(self, text: str) -> tuple[list[Movement], str]:
        """Parses the movements from a play contents section and returns a list of Movement objects and the remaining text."""

        text = self.__skip(text, r"\[MOVEMENTS\]")
        if text.startswith("["):
            return [], text

        movements, text = self.__match_until(text, r"\[[A-Z]")
        movements_text = movements.split(", ")

        movements = [self.__parse_movement(movement_text) for movement_text in movements_text]

        return movements, text

    def __parse_play_contents_base(self, text: str) -> tuple[int, str]:
        """Parses a base from a play contents section and returns the base number and the remaining text."""

        text = self.__skip(text, r"\[BASE\]")
        base_text, text = self.__match(text, r"home|\d")
        if base_text == "home":
            base = 4
        else:
            base = int(base_text)

        return base, text

    def __parse_play(self, text: str) -> tuple[Play, str]:
        """Parses a single play from the game and returns a Play object and the remaining text."""

        play = Play(None, None)

        text = self.__skip(text, r"\[PLAY\]")
        play_type_token, text = self.__match_until(text, r"\[")
        play.play_type = PlayType.from_text(play_type_token)

        play_contents = PlayContents()
        match play.play_type:
            case PlayType.GROUNDOUT \
            | PlayType.BUNT_GROUNDOUT \
            | PlayType.LINEOUT \
            | PlayType.BUNT_LINEOUT \
            | PlayType.FLYOUT \
            | PlayType.POP_OUT \
            | PlayType.BUNT_POP_OUT \
            | PlayType.FORCEOUT \
            | PlayType.DOUBLE_PLAY \
            | PlayType.TRIPLE_PLAY \
            | PlayType.RUNNER_DOUBLE_PLAY \
            | PlayType.RUNNER_TRIPLE_PLAY \
            | PlayType.GROUNDED_INTO_DOUBLE_PLAY \
            | PlayType.STRIKEOUT_DOUBLE_PLAY \
            | PlayType.FIELDERS_CHOICE \
            | PlayType.CATCHER_INTERFERENCE \
            | PlayType.FIELD_ERROR:
                play_contents.batter, text = self.__parse_play_contents_player(text, r"\[BATTER\]")
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.fielders, text = self.__parse_play_contents_fielders(text)
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.STRIKEOUT \
            | PlayType.SINGLE \
            | PlayType.DOUBLE \
            | PlayType.TRIPLE \
            | PlayType.HOME_RUN \
            | PlayType.WALK \
            | PlayType.INTENT_WALK \
            | PlayType.HIT_BY_PITCH:
                play_contents.batter, text = self.__parse_play_contents_player(text, r"\[BATTER\]")
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.PICKOFF \
            | PlayType.PICKOFF_ERROR \
            | PlayType.CAUGHT_STEALING \
            | PlayType.PICKOFF_CAUGHT_STEALING:
                play_contents.base, text = self.__parse_play_contents_base(text)
                play_contents.runner, text = self.__parse_play_contents_player(text, r"\[RUNNER\]")
                play_contents.fielders, text = self.__parse_play_contents_fielders(text)
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.WILD_PITCH:
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.runner, text = self.__parse_play_contents_player(text, r"\[RUNNER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.RUNNER_OUT \
            | PlayType.FIELD_OUT:
                play_contents.runner, text = self.__parse_play_contents_player(text, r"\[RUNNER\]")
                play_contents.fielders, text = self.__parse_play_contents_fielders(text)
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.BALK:
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.PASSED_BALL \
            | PlayType.ERROR:
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.catcher, text = self.__parse_play_contents_player(text, r"\[CATCHER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.STOLEN_BASE:
                play_contents.base, text = self.__parse_play_contents_base(text)
                play_contents.runner, text = self.__parse_play_contents_player(text, r"\[RUNNER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.SAC_FLY \
            | PlayType.SAC_FLY_DOUBLE_PLAY \
            | PlayType.FIELDERS_CHOICE_OUT:
                play_contents.batter, text = self.__parse_play_contents_player(text, r"\[BATTER\]")
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.fielders, text = self.__parse_play_contents_fielders(text)
                play_contents.scoring_runner, text = self.__parse_play_contents_player(text, r"\[SCORING_RUNNER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.SAC_BUNT \
            | PlayType.SAC_BUNT_DOUBLE_PLAY:
                play_contents.batter, text = self.__parse_play_contents_player(text, r"\[BATTER\]")
                play_contents.pitcher, text = self.__parse_play_contents_player(text, r"\[PITCHER\]")
                play_contents.fielders, text = self.__parse_play_contents_fielders(text)
                play_contents.runner, text = self.__parse_play_contents_player(text, r"\[RUNNER\]")
                play_contents.movements, text = self.__parse_play_contents_movements(text)
            case PlayType.GAME_ADVISORY:
                pass
            case _:
                raise ValueError(f"Play type {play.play_type} not supported")

        play.contents = play_contents

        return play, text

    def __parse_plays(self, text: str) -> tuple[list[Play], str]:
        """Parses the plays section of the game and returns a list of Play objects and the remaining text."""

        text = self.__skip(text, r"\[GAME_START\]")

        plays = []
        while not text.startswith("[GAME_END]"):
            play, text = self.__parse_play(text)
            plays.append(play)

        text = self.__skip(text, r"\[GAME_END\]")

        return plays, text

    def parse(self, text: str) -> Game:
        """Parses a game from the text and returns a Game object."""

        game = Game(None, None)
        game.context, text = self.__parse_context(text)
        game.plays, text = self.__parse_plays(text)

        if text:
            raise ValueError(f"Expected end of game but got {text}.")

        return game


def test_parse(parser: Parser, path: str) -> bool:
    with open(path) as file:
        text = file.read()

    try:
        parser.parse(text)
        return True
    except Exception as e:
        print(f"Error in {path}: {e}")
        return False


if __name__ == "__main__":
    parser = Parser()

    match sys.argv[1]:
        case "all":
            paths = glob("tokenized_data/**/**/*.txt")

            for path in tqdm(paths):
                test_parse(parser, path)
        case _:
            if test_parse(parser, sys.argv[1]):
                print("Parsed game successfully.")
