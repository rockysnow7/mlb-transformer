# mlb-transformer

this repo contains code to gather and preprocess MLB play-by-play data. it is part of a larger project to train a transformer model to predict the outcome of baseball games.

## gathering data

to collect raw data, run `cargo run get {year}`, where `{year}` is the year you want to collect data for.
this will create a directory `data/{year}` with each game saved in the subdirectory `data/{year}/{home_team_id}`.
for example, `cargo run get 2021` will create `data/2021/108`, `data/2021/109`, etc., each containing the
play-by-play data for all games played by those teams in 2021.

(please do not run this command too frequently, as it will put a strain on the MLB servers.)

to preprocess the raw data, run `cargo run preprocess`. this will create a directory `preprocessed_data` with each game saved in the subdirectory `preprocessed_data/{year}/{home_team_id}`.

the preprocessed data format is described in `FORMAT.md`.

## huggingface

a preprocessed dataset can be found on [huggingface](https://huggingface.co/datasets/finnnnnnnnnnnn/mlb-play-by-plays). this is in the process of being updated.
