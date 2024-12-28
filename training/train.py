from tokenizers import Tokenizer
from train_tokenizer import SPECIAL_TOKENS


tokenizer = Tokenizer.from_file("training/tokenizer.json")


def decode_tokens(tokens: list[int]) -> str:
    decoded = tokenizer.decode(tokens, skip_special_tokens=False)

    tokens = decoded.split()
    filtered = [token for token in tokens if token not in SPECIAL_TOKENS]
    joined = " ".join(filtered)
    joined = joined.replace(" - ", "-")

    return joined


with open("tokenized_data/2023/112/716449.txt") as f:
    text = f.read()

print(text)
print()

encoded = tokenizer.encode(text)

decoded = decode_tokens(encoded.ids)
print(decoded)