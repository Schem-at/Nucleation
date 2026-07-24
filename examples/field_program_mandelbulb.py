"""Portable power-8 Mandelbulb distance estimator.

This builds versioned, bounded FieldProgram bytecode. The resulting Sdf is
serializable and can be evaluated by every Nucleation binding without a host
callback.
"""
from nucleation import (
    FieldProgramBinaryOp as B,
    FieldProgramBuilder,
    FieldProgramDistanceKind as DistanceKind,
    FieldProgramUnaryOp as U,
    FieldProgramValueType as ValueType,
    Sdf,
)


def mandelbulb(power: float = 8.0, bailout: float = 4.0, iterations: int = 12) -> Sdf:
    p = FieldProgramBuilder.create()
    z = p.add_slot(ValueType.Vec3)
    dr = p.add_slot(ValueType.Scalar)
    radius = p.add_slot(ValueType.Scalar)
    distance = p.add_slot(ValueType.Scalar)
    theta = p.add_slot(ValueType.Scalar)
    phi = p.add_slot(ValueType.Scalar)
    zr = p.add_slot(ValueType.Scalar)
    radius_safe = p.add_slot(ValueType.Scalar)

    p.set_output(distance)
    p.set_bounds(-1.3, -1.3, -1.3, 1.3, 1.3, 1.3)
    p.set_distance_kind(DistanceKind.Estimate)

    p.push_pos()
    p.store_local(z)
    p.push_const_scalar(1.0)
    p.store_local(dr)

    p.begin_repeat(iterations)

    p.load_local(z)
    p.unary_op(U.Length)
    p.store_local(radius)

    p.load_local(radius)
    p.push_const_scalar(bailout)
    p.binary_op(B.Gt)
    p.break_if()

    # theta = acos(clamp(z.z / max(r, eps), -1, 1)) * power
    p.load_local(z)
    p.unary_op(U.VecZ)
    p.load_local(radius)
    p.push_const_scalar(1.0e-6)
    p.binary_op(B.Max)
    p.binary_op(B.Div)
    p.push_const_scalar(-1.0)
    p.push_const_scalar(1.0)
    p.clamp()
    p.unary_op(U.Acos)
    p.push_const_scalar(power)
    p.binary_op(B.Mul)
    p.store_local(theta)

    # phi = atan2(z.y, z.x) * power
    p.load_local(z)
    p.unary_op(U.VecY)
    p.load_local(z)
    p.unary_op(U.VecX)
    p.binary_op(B.Atan2)
    p.push_const_scalar(power)
    p.binary_op(B.Mul)
    p.store_local(phi)

    # dr = r^(power - 1) * power * dr + 1
    p.load_local(radius)
    p.push_const_scalar(power - 1.0)
    p.binary_op(B.Pow)
    p.push_const_scalar(power)
    p.binary_op(B.Mul)
    p.load_local(dr)
    p.binary_op(B.Mul)
    p.push_const_scalar(1.0)
    p.binary_op(B.Add)
    p.store_local(dr)

    p.load_local(radius)
    p.push_const_scalar(power)
    p.binary_op(B.Pow)
    p.store_local(zr)

    # z = zr * vec3(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta)) + pos
    p.load_local(theta)
    p.unary_op(U.Sin)
    p.load_local(phi)
    p.unary_op(U.Cos)
    p.binary_op(B.Mul)
    p.load_local(phi)
    p.unary_op(U.Sin)
    p.load_local(theta)
    p.unary_op(U.Sin)
    p.binary_op(B.Mul)
    p.load_local(theta)
    p.unary_op(U.Cos)
    p.make_vec3()
    p.load_local(zr)
    p.binary_op(B.Scale)
    p.push_pos()
    p.binary_op(B.Add)
    p.store_local(z)

    p.end_repeat()

    # 0.5 * log(max(r, eps)) * max(r, eps) / dr
    p.load_local(radius)
    p.push_const_scalar(1.0e-6)
    p.binary_op(B.Max)
    p.store_local(radius_safe)
    p.push_const_scalar(0.5)
    p.load_local(radius_safe)
    p.unary_op(U.Log)
    p.binary_op(B.Mul)
    p.load_local(radius_safe)
    p.binary_op(B.Mul)
    p.load_local(dr)
    p.binary_op(B.Div)
    p.store_local(distance)

    program = p.build()
    # Explicit interchange works without making JSON the authoring API.
    restored = type(program).from_json_string(program.to_json())
    return Sdf.from_program(restored)


if __name__ == "__main__":
    bulb = mandelbulb()
    assert bulb.eval_at(0.0, 0.0, 0.0) <= 0.0
    assert bulb.eval_at(5.0, 5.0, 5.0) > 3.0
    assert '"version":1' in bulb.to_json()
    # It is an ordinary graph node: transforms and booleans remain available.
    composed = bulb.scale(16.0).smooth_union(Sdf.sphere(3.0).translate(0.0, -19.0, 0.0), 1.5)
    assert composed.bounds() is not None
    print("portable Mandelbulb OK")
