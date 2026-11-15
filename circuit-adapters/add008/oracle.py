"""Full-adder truth table and every ordered transition between its eight inputs."""


def generate():
    states = [(bool(i & 1), bool(i & 2), bool(i & 4)) for i in range(8)]
    sequence = states + [value for source in states for target in states
                         for value in (source, target)]
    return {
        'schema_version': 1,
        'adapter': 'add008',
        'oracle': 'Independent three-Boolean integer sum: sum=bool(total&1); cout=total>=2. All eight input combinations and all 64 directed endpoint transitions, including unchanged inputs.',
        'vectors': [
            {'inputs': {'a': a, 'b': b, 'cin': cin},
             'expected': {'sum': bool((a + b + cin) & 1), 'cout': a + b + cin >= 2}}
            for a, b, cin in sequence
        ],
    }
