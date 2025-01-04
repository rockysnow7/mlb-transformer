# mlb-transformer

this repo contains code to gather MLB play-by-play data, preprocess it, and train a transformer model on it. this is a work in progress.

to collect raw data, run `cargo run get {year}`, where `{year}` is the year you want to collect data for. this will create a directory `data/{year}` with each game saved in the subdirectory `data/{year}/{home_team_id}`. for example, `cargo run get 2021` will create `data/2021/108`, `data/2021/109`, etc., each containing the play-by-play data for all games played by those teams in 2021. please do not run this command too frequently, as it will put a strain on the MLB servers.

to preprocess the raw data, run `cargo run tokenize`. this will create a directory `tokenized_data` with each game saved in the subdirectory `tokenized_data/{year}/{home_team_id}`.

the raw and tokenized data for 2020-2024 (inclusive) is available in the `data` and `tokenized_data` directories respectively.

a tokenizer has been trained on the tokenized data and saved in `training/tokenizer.json`. a notebook `training/MLB Train.ipynb` is provided to train the model on the tokenized data. a trained model is available on [huggingface](https://huggingface.co/finnnnnnnnnnnn/mlb-v1.1).

IMPORTANT: you should not let the model decode its own output. instead, use the following function:

```python
def decode_tokens(tokenizer: PreTrainedTokenizer, tokens: list[int]) -> str:
    decoded = tokenizer.decode(tokens, skip_special_tokens=False)

    tokens = decoded.split()
    filtered = [token for token in tokens if token not in SPECIAL_TOKENS]
    joined = " ".join(filtered)
    joined = joined.replace(" - ", "-")

    return joined
```