"""Sequential stored-bit model, independent of simulator observations.

Data is stable for a complete endpoint before each requested button pulse.
capture=True requests one real stone-button press; false requests no press.
"""


def generate():
    vectors = []
    stored = False

    def operation(data, capture=False):
        nonlocal stored
        if capture:
            stored = data
        vectors.append({"inputs": {"data": data, "capture": capture},
                        "expected": {"stored": stored}})

    # Establish a known state by a real write, not by sampling the initial sim.
    operation(False, True)
    # Both polarities, repeated writes and repeated alternation. Each write is
    # preceded by a no-clock stage; each pulse has 64 ticks to release naturally.
    for value in (False, True, True, False, False, True, False, True):
        operation(value)
        operation(value, True)
        operation(value)
        operation(value, True)  # repeated write of the same stored value
        for data in (not value, not value, value, not value, value, value):
            operation(data)   # hold even while the live data input toggles
        operation(value, True)
        operation(not value)
        operation(value)
    # Explicitly exercise every held-value/live-data combination with long
    # duplicate holds, and a final repeated capture of each polarity.
    for value in (False, True):
        operation(value)
        operation(value, True)
        for data in (False, True, False, True):
            operation(data)
            operation(data)
        operation(value)
        operation(value, True)
        operation(value, True)
    return {
        "schema_version": 1,
        "adapter": "lff006",
        "oracle": (
            "Independent sequential stored bit: capture=true performs one actual button pulse "
            "and stores stable data; capture=false preserves the previous bit. "
            "Both polarities, hold across input transitions, duplicate holds, and repeated writes; "
            "no edge-latency or fresh vanilla-parity claim."
        ),
        "vectors": vectors,
    }
