from game import parse_game, Game, PlayType
from glob import glob


def runs_at_end(game: Game) -> dict[str, int]:
    """Return the total number of runs scored by each team at the end of the game."""

    scores = {
        game.context.home_team.id: 0,
        game.context.away_team.id: 0,
    }

    home_team_player_names = [player.name for player in game.context.home_team.players]
    away_team_player_names = [player.name for player in game.context.away_team.players]

    for play in game.plays:
        if play.contents.movements is None:
            continue

        for movement in play.contents.movements:
            if movement.end_base == 4 and not movement.is_out:
                if movement.start_base == 4 and play.play_type != PlayType.HOME_RUN:
                    continue

                if movement.runner in home_team_player_names:
                    scores[game.context.home_team.id] += 1
                    print(f"{movement.runner} scored a run for Team {game.context.home_team.id} ({movement.start_base} -> {movement.end_base})")
                elif movement.runner in away_team_player_names:
                    scores[game.context.away_team.id] += 1
                    print(f"{movement.runner} scored a run for Team {game.context.away_team.id} ({movement.start_base} -> {movement.end_base})")
                print(f"\t\t\t\t\t\t\t(home {scores[game.context.home_team.id]}, away {scores[game.context.away_team.id]})")

    return scores


if __name__ == "__main__":
    paths = glob("tokenized_data/**/**/*.txt")
    game_path = paths[1]

    with open(game_path) as f:
        game = parse_game(f.read())

    print(f"path: {game_path}")
    print(f"url: https://statsapi.mlb.com/api/v1/game/{game.context.game_pk}/boxscore")
    print()
    print(runs_at_end(game))