"""Independent four-bit functional oracle; never reads simulator outputs."""
import json

PROGRAM = (15, 15, 15, 15)  # independently read original programming crosspoints

def encode(selected):
    word = 0
    for bit, programmed in enumerate(PROGRAM):
        if selected & (1 << bit):
            word |= programmed
    return word


def generate():
    # Includes each of all256 ordered old/new input pairs, plus initial zero.
    values = [0] + [v for old in range(16) for new in range(16) for v in (old, new)]
    return {
        "schema_version": 1,
        "adapter": "encode003",
        "oracle": "All16 programming crosspoints contain north-facing wall torches; each input has a west-facing inversion torch, so each selected row encodes1111; outputs are a wired OR. All16 inputs and all256 ordered settled transitions;513 operations. Expected values are independent of mc-tick.",
        "vectors": [{"inputs": {"selected": value},
                     "expected": {"code": encode(value)}} for value in values],
    }


if __name__ == "__main__":
    print(json.dumps(generate(), indent=2))
