from game import parse_game
from glob import glob


def test_parse_all() -> None:
    paths = glob("tokenized_data/**/**/*.txt")
    for path in paths:
        with open(path) as f:
            try:
                parse_game(f.read())
            except:
                assert False, f"Failed to parse {path}"
    assert True