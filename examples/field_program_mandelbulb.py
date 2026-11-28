"""Portable power-8 Mandelbulb distance estimator built with Field.expr.

Compiles a bounded infix expression into Nucleation's sandboxed bytecode VM so
the resulting Field is serializable (FieldGraph v2 JSON) and evaluates natively
across all bindings with zero host callbacks.
"""

from nucleation import Field, FieldDistanceKind, FieldExprBuilder


def mandelbulb(power: float = 8.0, bailout: float = 4.0, iterations: int = 12) -> Field:
    source = f"""
        let z = p;
        let dr = 1.0;
        let r = 0.0;
        for i in 0..{iterations} {{
            r = length(z);
            if r > {bailout} {{ break; }}
            let theta = acos(clamp(z.z / max(r, 1e-6), -1.0, 1.0)) * {power};
            let phi = atan2(z.y, z.x) * {power};
            dr = pow(max(r, 1e-6), {power - 1.0}) * {power} * dr + 1.0;
            let zr = pow(max(r, 1e-6), {power});
            z = vec3(sin(theta) * cos(phi), sin(phi) * sin(theta), cos(theta)) * zr + p;
        }}
        let r_safe = max(r, 1e-6);
        0.5 * log(r_safe) * r_safe / max(dr, 1e-6)
    """
    builder = FieldExprBuilder.create()
    builder.set_bounds(-1.3, -1.3, -1.3, 1.3, 1.3, 1.3)
    builder.set_distance_kind(FieldDistanceKind.Estimate)
    return builder.compile(source)
