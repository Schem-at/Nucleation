"""Independent four-bit functional oracle; never reads simulator outputs."""
import json


def generate():
    # Includes each of all256 ordered old/new input pairs, plus initial zero.
    values = [0] + [v for old in range(16) for new in range(16) for v in (old, new)]
    return {
        "schema_version": 1,
        "adapter": "othercomb001",
        "oracle": "Outputs input minus one, modulo16. All16 inputs and all256 ordered settled transitions;513 operations. Expected values are independent of mc-tick.",
        "vectors": [{"inputs": {"value": value},
                     "expected": {"result": (value - 1) & 15}} for value in values],
    }


if __name__ == "__main__":
    print(json.dumps(generate(), indent=2))
