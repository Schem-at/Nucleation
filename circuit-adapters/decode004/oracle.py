"""Original four identical programmed rows: match 1010, not one-hot decoding.

Expectations come from the pinned schematic's repeater/torch programming,
independent of any simulator. Every ordered pair of four-bit input words is
exercised, in addition to ascending, descending and Gray-code traversals.
"""


def generate():
    ascending = list(range(16))
    gray = [n ^ (n >> 1) for n in range(16)]
    words = ascending + ascending[::-1] + gray + gray[::-1]
    # Explicit source/destination endpoints cover all 256 ordered transitions,
    # including repeats. The actual lever-bit write order remains LSB first.
    for source in range(16):
        for destination in range(16):
            words.extend((source, destination))
    # Repeatedly enter/leave the programmed pattern from every input word.
    for word in range(16):
        words.extend((10, word))
    return {
        "schema_version": 1,
        "adapter": "decode004",
        "oracle": (
            "Independent fixed programmed-row Boolean equation: each output is "
            "!(bit0 | !bit1 | bit2 | !bit3), so matches=15 iff data=10, otherwise 0; "
            "input_echo=data. 608 endpoints cover the complete 16-word truth table, "
            "all 256 ordered transitions, ascending/descending/Gray orders and repeated match transitions."
        ),
        "vectors": [
            {"inputs": {"data": word},
             "expected": {"matches": 15 if word == 10 else 0, "input_echo": word}}
            for word in words
        ],
    }
