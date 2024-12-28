from glob import glob
from tokenizers import Tokenizer, models, pre_tokenizers, decoders, trainers, processors, normalizers
from tqdm import tqdm

import re


SPECIAL_TOKENS = [
    "[UNK]",
    "[PAD]",
    "[CLS]",
    "[SEP]",
    "[MASK]",
]


def get_game_tokens_from_game(game: str) -> list[str]:
    game = re.sub(r"\s+", " ", game)
    game = re.sub(r",", "", game)

    all_tokens = game.split()
    interesting_tokens = []

    while all_tokens:
        token = all_tokens.pop(0)

        if token.startswith("[") and token.endswith("]"):
            interesting_tokens.append(token)

        if token == "[PLAY]":
            parts = []
            while (part := all_tokens.pop(0)) and not part.startswith("["):
                parts.append(part)
            play = " ".join(parts)

            interesting_tokens.append(play)

        elif token == "[MOVEMENTS]":
            while all_tokens[1] != "->":
                all_tokens.pop(0)

            start_base = all_tokens.pop(0)
            all_tokens.pop(0)
            end_base = all_tokens.pop(0)
            token = f"{start_base} -> {end_base}"

            interesting_tokens.append(token)

    return interesting_tokens

def get_game_tokens() -> list[str]:
    tokens = set()
    all_game_paths = glob("tokenized_data/**/**/*.txt")
    for path in tqdm(all_game_paths):
        with open(path) as f:
            game = f.read()

        game_tokens = get_game_tokens_from_game(game)
        tokens.update(game_tokens)

    return sorted(tokens)


def train_tokenizer():
    game_tokens = get_game_tokens()
    special_tokens = SPECIAL_TOKENS + game_tokens

    tokenizer = Tokenizer(models.WordPiece(unk_token="[UNK]"))
    tokenizer.normalizer = normalizers.Sequence([
        normalizers.NFKC(),
    ])
    tokenizer.pre_tokenizer = pre_tokenizers.Sequence([
        pre_tokenizers.Punctuation(),
        pre_tokenizers.Whitespace(),
    ])
    tokenizer.decoder = decoders.WordPiece()
    trainer = trainers.WordPieceTrainer(
        vocab_size=10000,
        special_tokens=special_tokens,
    )

    all_game_paths = glob("tokenized_data/**/**/*.txt")
    tokenizer.train(files=all_game_paths, trainer=trainer)

    tokenizer.post_processor = processors.TemplateProcessing(
        single="[CLS] $A [SEP]",
        pair="[CLS] $A [SEP] $B:1 [SEP]:1",
        special_tokens=[
            ("[CLS]", tokenizer.token_to_id("[CLS]")),
            ("[SEP]", tokenizer.token_to_id("[SEP]")),
        ],
    )

    # Ensure game tokens are added to the tokenizer's vocabulary
    for token in game_tokens:
        tokenizer.add_special_tokens([token])

    tokenizer.save("training/tokenizer.json")


if __name__ == "__main__":
    train_tokenizer()