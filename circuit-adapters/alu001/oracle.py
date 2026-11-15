"""Sign-labelled ALU operations, using Python integer arithmetic and bit logic."""

OPERATIONS = ('add', 'sub', 'and', 'nand', 'or', 'nor', 'xor', 'xnor',
              'a_imply_b', 'a_nimply_b', 'right_shift')


def expected(operation, a, b):
    if operation == 'add':
        return (a + b) & 255
    if operation == 'sub':
        return (a - b) & 255
    if operation == 'and':
        return a & b
    if operation == 'nand':
        return ~(a & b) & 255
    if operation == 'or':
        return a | b
    if operation == 'nor':
        return ~(a | b) & 255
    if operation == 'xor':
        return a ^ b
    if operation == 'xnor':
        return ~(a ^ b) & 255
    if operation == 'a_imply_b':
        return (~a | b) & 255
    if operation == 'a_nimply_b':
        return (a & ~b) & 255
    if operation == 'right_shift':
        assert b == 0, 'Right-shift contract only covers b=0'
        return a >> 1
    raise ValueError(operation)


def generate():
    pairs = [(0, 0), (255, 0), (0, 255), (255, 255), (127, 1),
             (1, 127), (128, 128), (85, 170), (170, 85), (85, 85), (170, 170)]
    for bit in range(8):
        value = 1 << bit
        pairs.extend([(value, 0), (0, value), (value, value), (value - 1, 1)])
    state = 0xA1C001
    for _ in range(32):
        state = (state * 1664525 + 1013904223) & 0xffffffff
        a = state >> 24
        state = (state * 1664525 + 1013904223) & 0xffffffff
        pairs.append((a, state >> 24))
    # Interleave operations for each pair to exercise mode transitions.
    cases = [(operation, a, b) for a, b in pairs for operation in OPERATIONS[:10]]
    cases += [('right_shift', a, 0) for a in range(256)]
    cases += [(operation, 0, 0) for operation in reversed(OPERATIONS)]
    return {
        'schema_version': 1,
        'adapter': 'alu001',
        'oracle': 'Independent Python unsigned 8-bit arithmetic/bitwise logic for ten operations, plus exhaustive a>>1 with b=0. Operation is a one-hot mask in source sign order. Overflow/borrow discarded; unrestricted two-operand pairs and combined selectors are not exhaustive.',
        'vectors': [
            {'inputs': {'a': a, 'b': b, 'operation': 1 << OPERATIONS.index(operation)},
             'expected': {'out': expected(operation, a, b)}}
            for operation, a, b in cases
        ],
    }
