# game data format

preprocessed play-by-play data is stored in JSONL files.

## overview

the first line of a JSONL file is a JSON object containing the context of the game.
all subsequent lines are JSON objects representing plays.

a play line is either an introduction line, an information line, or a movement line.

an introduction line contains the following fields:

- `inning`: an inning object.
- `type`: the play type, a string.

information lines and movement lines are described further below.

Game Advisories do not require an information line or a movement line.

Ejections do not require an information line, but do require a movement line.

All other play types require an information line and a movement line.

## context line

the context object contains the following fields:

- `game_pk`: the game pk, an integer.
- `date`: the date of the game, a string in the format `YYYY-MM-DD`.
- `venue_name`: the name of the venue, a string.
- `weather`: the weather at the start of the game, a weather object.
- `home_team`: a team object for the home team.
- `away_team`: a team object for the away team.

### weather

the weather object contains the following fields:

- `condition`: the weather condition, a string.
- `temperature`: the temperature, an integer.
- `wind_speed`: the wind speed, an integer.

### team

a team object contains the following fields:

- `id`: the team id, an integer.
- `players`: a list of player objects.

### player

a player object contains the following fields:

- `position`: the position of the player, a string.
- `name`: the name of the player, a string.

## information lines

an information object contains all the information required for the play type introduced on the previous line.

the following table lists the information required for each play type (in order, from left to right):

| Play Type | Base | Batter | Pitcher | Catcher | Fielders | Runner | Scoring Runner |
|-|-|-|-|-|-|-|-|
| Groundout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Bunt Groundout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Strikeout | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Lineout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Bunt Lineout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Flyout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Pop Out | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Bunt Pop Out | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Forceout | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Fielders Choice Out | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ |
| Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Triple Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Runner Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Runner Triple Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Grounded Into Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Strikeout Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Pickoff | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Pickoff Error | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Caught Stealing | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Pickoff Caught Stealing | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Wild Pitch | ✗ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ |
| Runner Out | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Field Out | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ |
| Batter Out | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Balk | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Passed Ball | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ |
| Error | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ |
| Single | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Double | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Triple | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Home Run | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Walk | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Intent Walk | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Hit By Pitch | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| Fielders Choice | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Catcher Interference | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Stolen Base | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Sac Fly | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ |
| Sac Fly Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ |
| Sac Bunt | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ |
| Sac Bunt Double Play | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ |
| Field Error | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| Game Advisory | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Ejection | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

the keys and types for each of these pieces of information are listed below:

- Base: `base`, an integer.
- Batter: `batter`, a string.
- Pitcher: `pitcher`, a string.
- Catcher: `catcher`, a string.
- Fielders: `fielders`, a list of strings.
- Runner: `runner`, a string.
- Scoring Runner: `scoring_runner`, a string.

### inning

an inning object contains the following fields:

- `number`: the inning number, an integer.
- `top`: a boolean indicating if the inning is in the top half.

### fielders

`fielders` is a list of player names (strings).

## movement lines

a movement object contains one field, `movements`, a list of movement objects.

### movements

a movement object represents a runner's movement from one base to another, possibly being out.

a movement object contains the following fields:

- `runner`: the name of the runner, a string.
- `start_base`: the starting base, a string (one of `home`, `1`, `2`, `3`, or `4`).
- `end_base`: the ending base, a string (one of `home`, `1`, `2`, `3`, or `4`).
- `is_out`: a boolean indicating if the runner is out.
