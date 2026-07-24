import at.schem.nucleation.FieldProgram;
import at.schem.nucleation.FieldProgramBinaryOp;
import at.schem.nucleation.FieldProgramDistanceKind;
import at.schem.nucleation.FieldProgramDsl;
import at.schem.nucleation.FieldProgramUnaryOp;
import at.schem.nucleation.FieldProgramValueType;
import at.schem.nucleation.SdfExpr;

/** Compile-time Java interop smoke for the handwritten exception/int facade. */
public final class ProgramJavaSmoke {
    private ProgramJavaSmoke() {}

    public static void compileSmoke() {
        FieldProgramDsl builder = FieldProgramDsl.create();
        int distance = builder.addSlot(FieldProgramValueType.Scalar);
        FieldProgram program = builder
            .output(distance)
            .bounds(-4.0f, -4.0f, -4.0f, 4.0f, 4.0f, 4.0f)
            .distanceKind(FieldProgramDistanceKind.Exact)
            .pushPosition()
            .unary(FieldProgramUnaryOp.Length)
            .pushConst(2.0f)
            .binary(FieldProgramBinaryOp.Sub)
            .store(distance)
            .build();

        SdfExpr field = SdfExpr.fromProgram(program);
        if (!(field.evalAt(0.0f, 0.0f, 0.0f) < 0.0f)) {
            throw new AssertionError("portable program did not evaluate inside the sphere");
        }

        SdfExpr iq = SdfExpr.squarePyramid(2.0f, 4.0f)
            .elongate(1.0f, 0.0f, 1.0f)
            .twist(0.1f)
            .bend(0.05f)
            .xorWith(SdfExpr.sphere(1.0f));
        if (!Float.isFinite(iq.evalAt(4.0f, 0.0f, 0.0f))) {
            throw new AssertionError("IQ façade evaluation was non-finite");
        }
        if (!(SdfExpr.infiniteCone(45.0f).evalAt(0.0f, -1.0f, 0.0f) > 0.0f)) {
            throw new AssertionError("infinite cone orientation was incorrect");
        }
    }
}
