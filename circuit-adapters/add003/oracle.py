"""Unsigned full-adder expectations computed without consulting the simulator."""


def generate():
    pairs = [(0, 0), (255, 0), (0, 255), (255, 255), (127, 1),
             (128, 128), (85, 170), (170, 85), (85, 85), (170, 170)]
    for bit in range(8):
        value = 1 << bit
        pairs.extend([(value, 0), (0, value), (value, value),
                      (value - 1, 1), (255 ^ value, value), (0, 0)])
    state = 0xADD003
    for _ in range(128):
        state = (state * 1664525 + 1013904223) & 0xffffffff
        a = state >> 24
        state = (state * 1664525 + 1013904223) & 0xffffffff
        pairs.append((a, state >> 24))
    return {
        'schema_version': 1,
        'adapter': 'add003',
        'oracle': 'Independent unsigned arithmetic: total=a+b+cin; sum=total&255; cout=total>=256. Carry boundaries, walking bits, transitions, and 128 deterministic pseudorandom operand pairs; not exhaustive.',
        'vectors': [
            {'inputs': {'a': a, 'b': b, 'cin': cin},
             'expected': {'sum': (a + b + cin) & 255, 'cout': a + b + cin >= 256}}
            for a, b in pairs for cin in (False, True)
        ],
    }
