//! Sandboxed, portable infix expression language for custom SDFs and fields.
//!
//! Compiles a clean, readable math/shader-like expression directly into a
//! statically validated, provably terminating register/stack bytecode VM that
//! runs over plain `f32`, forward-mode dual numbers ([`Dual3`]) for analytic
//! gradients, and can read any bound external [`Field`] inputs!
//!
//! ## Language summary
//!
//! - Built-in variables: `x`, `y`, `z` (`Scalar`), `p` (`Vec3([x, y, z])`)
//! - External field bindings: any bound name evaluates that [`Field`] at `(x, y, z)` (or call `name(q)` at a transformed `Vec3`!)
//! - Statements:
//!   - `let name = expr;`
//!   - `name = expr;`
//!   - `for i in 0..N { ... }` (statically bounded `1 <= N <= 4096`)
//!   - `if cond { break; }`
//!   - Final expression is the scalar output!
//! - Expressions:
//!   - `+`, `-`, `*`, `/`, `%`, `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, `||`, `!`
//!   - `if cond { a } else { b }` (branchless `Select` over `Scalar` or `Vec3`)
//!   - `vec3(x, y, z)`, swizzles `.x`, `.y`, `.z`
//!   - Built-ins:
//!     - Math: `abs`, `sqrt`, `ln`, `log`, `exp`, `sin`, `cos`, `tan`, `acos`, `asin`, `atan2`, `floor`, `ceil`, `fract`, `sign`, `min`, `max`, `pow`, `clamp`, `mix`, `lerp`, `smoothstep`, `step`, `smin`, `smax`, `saturate`
//!     - Vec3: `length`, `dot`, `cross`, `normalize`, `abs(v)`, `min(v, s)`, `max(v, s)`
//!     - Noise: `value_noise(p, seed)`, `perlin(p, seed)`, `fbm(p, freq, seed, octaves)`, `ridged(p, freq, seed, octaves)`, `worley(p, freq, seed)`
//!     - Primitives: `sphere(p, r)`, `box(p, b)` (where `b` is `Vec3`), `torus(p, r_maj, r_min)`, `cylinder(p, r, h)`, `capsule(p, a, b, r)`, `plane(p, n, off)`

use super::noise::{fbm3_params, perlin3, value_noise3, worley3, CellMode};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

pub const MAX_EXPR_SOURCE_BYTES: usize = 64 * 1024;
pub const MAX_EXPR_SLOTS: usize = 128;
pub const MAX_EXPR_INSTRUCTIONS: usize = 4096;
pub const MAX_EXPR_REPEAT_DEPTH: usize = 8;
pub const MAX_EXPR_REPEAT_ITERATIONS: u32 = 4096;
pub const MAX_EXPR_DYNAMIC_STEPS: u64 = 200_000;

/// Forward-mode dual number carrying value `v` and partial derivatives `d = [dv/dx, dv/dy, dv/dz]`.
#[derive(Debug, Clone, Copy)]
pub struct Dual3 {
    pub v: f32,
    pub d: [f32; 3],
}

impl Dual3 {
    #[inline]
    pub fn constant(v: f32) -> Self {
        Self {
            v,
            d: [0.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn var(v: f32, axis: usize) -> Self {
        let mut d = [0.0; 3];
        d[axis] = 1.0;
        Self { v, d }
    }
}

/// Numeric scalar abstraction executed by both the compiled DAG and expression VM.
pub trait ScalarNum: Copy {
    fn constant(v: f32) -> Self;
    fn primal(self) -> f32;
    fn add(self, o: Self) -> Self;
    fn sub(self, o: Self) -> Self;
    fn mul(self, o: Self) -> Self;
    fn div(self, o: Self) -> Self;
    fn rem(self, o: Self) -> Self;
    fn neg(self) -> Self;
    fn abs(self) -> Self;
    fn sqrt(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn fract(self) -> Self;
    fn sign(self) -> Self;
    fn min(self, o: Self) -> Self;
    fn max(self, o: Self) -> Self;
    fn pow(self, o: Self) -> Self;
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan2(self, o: Self) -> Self;
    fn clamp_scalar(self, lo: Self, hi: Self) -> Self {
        self.max(lo).min(hi)
    }
    fn smin(self, o: Self, k: Self) -> Self {
        if k.primal() <= 0.0 {
            return self.min(o);
        }
        let h = Self::constant(0.5)
            .add(Self::constant(0.5).mul(o.sub(self)).div(k))
            .clamp_scalar(Self::constant(0.0), Self::constant(1.0));
        let mixed = o.add(self.sub(o).mul(h));
        mixed.sub(k.mul(h).mul(Self::constant(1.0).sub(h)))
    }
    fn smax(self, o: Self, k: Self) -> Self {
        self.neg().smin(o.neg(), k).neg()
    }
}

impl ScalarNum for f32 {
    #[inline]
    fn constant(v: f32) -> Self {
        v
    }
    #[inline]
    fn primal(self) -> f32 {
        self
    }
    #[inline]
    fn add(self, o: Self) -> Self {
        self + o
    }
    #[inline]
    fn sub(self, o: Self) -> Self {
        self - o
    }
    #[inline]
    fn mul(self, o: Self) -> Self {
        self * o
    }
    #[inline]
    fn div(self, o: Self) -> Self {
        self / o
    }
    #[inline]
    fn rem(self, o: Self) -> Self {
        self.rem_euclid(o)
    }
    #[inline]
    fn neg(self) -> Self {
        -self
    }
    #[inline]
    fn abs(self) -> Self {
        f32::abs(self)
    }
    #[inline]
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
    #[inline]
    fn floor(self) -> Self {
        f32::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        f32::ceil(self)
    }
    #[inline]
    fn fract(self) -> Self {
        self - self.floor()
    }
    #[inline]
    fn sign(self) -> Self {
        if self > 0.0 {
            1.0
        } else if self < 0.0 {
            -1.0
        } else {
            0.0
        }
    }
    #[inline]
    fn min(self, o: Self) -> Self {
        f32::min(self, o)
    }
    #[inline]
    fn max(self, o: Self) -> Self {
        f32::max(self, o)
    }
    #[inline]
    fn pow(self, o: Self) -> Self {
        f32::powf(self, o)
    }
    #[inline]
    fn ln(self) -> Self {
        f32::ln(self)
    }
    #[inline]
    fn exp(self) -> Self {
        f32::exp(self)
    }
    #[inline]
    fn sin(self) -> Self {
        f32::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        f32::cos(self)
    }
    #[inline]
    fn tan(self) -> Self {
        f32::tan(self)
    }
    #[inline]
    fn asin(self) -> Self {
        f32::asin(self)
    }
    #[inline]
    fn acos(self) -> Self {
        f32::acos(self)
    }
    #[inline]
    fn atan2(self, o: Self) -> Self {
        f32::atan2(self, o)
    }
}

impl ScalarNum for Dual3 {
    #[inline]
    fn constant(v: f32) -> Self {
        Dual3::constant(v)
    }
    #[inline]
    fn primal(self) -> f32 {
        self.v
    }
    #[inline]
    fn add(self, o: Self) -> Self {
        Dual3 {
            v: self.v + o.v,
            d: [self.d[0] + o.d[0], self.d[1] + o.d[1], self.d[2] + o.d[2]],
        }
    }
    #[inline]
    fn sub(self, o: Self) -> Self {
        Dual3 {
            v: self.v - o.v,
            d: [self.d[0] - o.d[0], self.d[1] - o.d[1], self.d[2] - o.d[2]],
        }
    }
    #[inline]
    fn mul(self, o: Self) -> Self {
        Dual3 {
            v: self.v * o.v,
            d: [
                self.d[0] * o.v + self.v * o.d[0],
                self.d[1] * o.v + self.v * o.d[1],
                self.d[2] * o.v + self.v * o.d[2],
            ],
        }
    }
    #[inline]
    fn div(self, o: Self) -> Self {
        let inv = 1.0 / o.v;
        let inv2 = inv * inv;
        Dual3 {
            v: self.v * inv,
            d: [
                (self.d[0] * o.v - self.v * o.d[0]) * inv2,
                (self.d[1] * o.v - self.v * o.d[1]) * inv2,
                (self.d[2] * o.v - self.v * o.d[2]) * inv2,
            ],
        }
    }
    #[inline]
    fn rem(self, o: Self) -> Self {
        // x mod y = x - y * floor(x / y); derivative is dx - floor(x/y)*dy almost everywhere.
        let q = (self.v / o.v).floor();
        Dual3 {
            v: self.v.rem_euclid(o.v),
            d: [
                self.d[0] - q * o.d[0],
                self.d[1] - q * o.d[1],
                self.d[2] - q * o.d[2],
            ],
        }
    }
    #[inline]
    fn neg(self) -> Self {
        Dual3 {
            v: -self.v,
            d: [-self.d[0], -self.d[1], -self.d[2]],
        }
    }
    #[inline]
    fn abs(self) -> Self {
        let s = if self.v > 0.0 {
            1.0
        } else if self.v < 0.0 {
            -1.0
        } else {
            f32::NAN
        };
        Dual3 {
            v: self.v.abs(),
            d: [self.d[0] * s, self.d[1] * s, self.d[2] * s],
        }
    }
    #[inline]
    fn sqrt(self) -> Self {
        let v = self.v.sqrt();
        let denom = 2.0 * v;
        Dual3 {
            v,
            d: [self.d[0] / denom, self.d[1] / denom, self.d[2] / denom],
        }
    }
    #[inline]
    fn floor(self) -> Self {
        Dual3::constant(self.v.floor())
    }
    #[inline]
    fn ceil(self) -> Self {
        Dual3::constant(self.v.ceil())
    }
    #[inline]
    fn fract(self) -> Self {
        Dual3 {
            v: self.v - self.v.floor(),
            d: self.d,
        }
    }
    #[inline]
    fn sign(self) -> Self {
        Dual3::constant(self.v.sign())
    }
    #[inline]
    fn min(self, o: Self) -> Self {
        if self.v <= o.v {
            self
        } else {
            o
        }
    }
    #[inline]
    fn max(self, o: Self) -> Self {
        if self.v >= o.v {
            self
        } else {
            o
        }
    }
    #[inline]
    fn pow(self, o: Self) -> Self {
        let v = self.v.powf(o.v);
        let coeff = o.v * self.v.powf(o.v - 1.0);
        let d = if o.d == [0.0, 0.0, 0.0] {
            [coeff * self.d[0], coeff * self.d[1], coeff * self.d[2]]
        } else {
            let l = self.v.ln();
            [
                coeff * self.d[0] + v * l * o.d[0],
                coeff * self.d[1] + v * l * o.d[1],
                coeff * self.d[2] + v * l * o.d[2],
            ]
        };
        Dual3 { v, d }
    }
    #[inline]
    fn ln(self) -> Self {
        Dual3 {
            v: self.v.ln(),
            d: [self.d[0] / self.v, self.d[1] / self.v, self.d[2] / self.v],
        }
    }
    #[inline]
    fn exp(self) -> Self {
        let e = self.v.exp();
        Dual3 {
            v: e,
            d: [e * self.d[0], e * self.d[1], e * self.d[2]],
        }
    }
    #[inline]
    fn sin(self) -> Self {
        let c = self.v.cos();
        Dual3 {
            v: self.v.sin(),
            d: [self.d[0] * c, self.d[1] * c, self.d[2] * c],
        }
    }
    #[inline]
    fn cos(self) -> Self {
        let s = -self.v.sin();
        Dual3 {
            v: self.v.cos(),
            d: [self.d[0] * s, self.d[1] * s, self.d[2] * s],
        }
    }
    #[inline]
    fn tan(self) -> Self {
        let c = self.v.cos();
        let sec2 = 1.0 / (c * c);
        Dual3 {
            v: self.v.tan(),
            d: [self.d[0] * sec2, self.d[1] * sec2, self.d[2] * sec2],
        }
    }
    #[inline]
    fn asin(self) -> Self {
        let denom = (1.0 - self.v * self.v).sqrt();
        Dual3 {
            v: self.v.asin(),
            d: [self.d[0] / denom, self.d[1] / denom, self.d[2] / denom],
        }
    }
    #[inline]
    fn acos(self) -> Self {
        let denom = (1.0 - self.v * self.v).sqrt();
        Dual3 {
            v: self.v.acos(),
            d: [-self.d[0] / denom, -self.d[1] / denom, -self.d[2] / denom],
        }
    }
    #[inline]
    fn atan2(self, o: Self) -> Self {
        let denom = o.v * o.v + self.v * self.v;
        Dual3 {
            v: self.v.atan2(o.v),
            d: [
                (o.v * self.d[0] - self.v * o.d[0]) / denom,
                (o.v * self.d[1] - self.v * o.d[1]) / denom,
                (o.v * self.d[2] - self.v * o.d[2]) / denom,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExprType {
    Scalar,
    Vec3,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryInstr {
    NegScalar,
    NegVec3,
    NotBool,
    AbsScalar,
    AbsVec3,
    Sqrt,
    Floor,
    Ceil,
    Fract,
    Sign,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Ln,
    Exp,
    Saturate,
    VecX,
    VecY,
    VecZ,
    Length,
    Normalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryInstr {
    AddScalar,
    SubScalar,
    MulScalar,
    DivScalar,
    RemScalar,
    PowScalar,
    MinScalar,
    MaxScalar,
    Atan2,
    Step,
    AddVec3,
    SubVec3,
    MulVec3,
    DivVec3,
    ScaleVec3,
    MinVec3,
    MaxVec3,
    Dot,
    Cross,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TernaryInstr {
    ClampScalar,
    MixScalar,
    MixVec3,
    Smoothstep,
    Smin,
    Smax,
    SelectScalar,
    SelectVec3,
}

#[derive(Debug, Clone)]
pub enum ExprOp {
    PushScalar(f32),
    PushBool(bool),
    PushPos,
    Load(u16),
    Store(u16),
    Pop,
    MakeVec3,
    Unary(UnaryInstr),
    Binary(BinaryInstr),
    Ternary(TernaryInstr),
    /// Evaluates external bound field `index` at the `Vec3` popped from stack.
    EvalInput(u16),
    /// Built-in noise & primitive intrinsics.
    ValueNoise,
    Perlin,
    Fbm { ridged: bool },
    Worley,
    PrimSphere,
    PrimBox,
    PrimTorus,
    PrimCylinder,
    PrimCapsule,
    PrimPlane,
    Repeat {
        count: u32,
        iter_slot: Option<u16>,
        body: Vec<ExprOp>,
    },
    BreakIf,
}

/// Compiled, validated expression program.
///
/// The original `source` and `input_names` are preserved for lossless JSON
/// serialization — deserializing re-parses and re-validates the source, so the
/// wire format is compact, human-readable, and impossible to corrupt.
#[derive(Debug, Clone)]
pub struct ExprProgram {
    pub source: String,
    pub input_names: Vec<String>,
    slots: Vec<ExprType>,
    ops: Vec<ExprOp>,
    output_slot: u16,
    has_non_diff: bool,
}

impl ExprProgram {
    pub fn compile(source: &str, input_names: &[String]) -> Result<Self, String> {
        if source.len() > MAX_EXPR_SOURCE_BYTES {
            return Err(format!(
                "expression source exceeds {MAX_EXPR_SOURCE_BYTES} bytes"
            ));
        }
        let tokens = tokenize(source)?;
        let mut parser = Parser::new(&tokens, input_names);
        let prog = parser.parse_program(source)?;
        prog.validate_budgets()?;
        Ok(prog)
    }

    pub fn is_differentiable(&self) -> bool {
        !self.has_non_diff
    }

    fn validate_budgets(&self) -> Result<(), String> {
        let mut count = 0usize;
        let mut steps = 0u64;
        fn walk(
            ops: &[ExprOp],
            depth: usize,
            mult: u64,
            count: &mut usize,
            steps: &mut u64,
        ) -> Result<(), String> {
            for op in ops {
                *count += 1;
                if *count > MAX_EXPR_INSTRUCTIONS {
                    return Err("expression exceeds maximum static instruction count".into());
                }
                *steps = steps.saturating_add(mult);
                if *steps > MAX_EXPR_DYNAMIC_STEPS {
                    return Err("expression exceeds dynamic step budget".into());
                }
                if let ExprOp::Repeat { count: n, body, .. } = op {
                    if *n == 0 || *n > MAX_EXPR_REPEAT_ITERATIONS {
                        return Err("loop iteration count out of range".into());
                    }
                    if depth + 1 > MAX_EXPR_REPEAT_DEPTH {
                        return Err("loop nesting depth exceeded".into());
                    }
                    let next_mult = mult.saturating_mul(u64::from(*n));
                    *steps = steps.saturating_add(next_mult);
                    if *steps > MAX_EXPR_DYNAMIC_STEPS {
                        return Err("expression exceeds dynamic step budget".into());
                    }
                    walk(body, depth + 1, next_mult, count, steps)?;
                }
            }
            Ok(())
        }
        walk(&self.ops, 0, 1, &mut count, &mut steps)
    }

    /// Evaluate the expression at `(x, y, z)` using `eval_input(input_idx, qx, qy, qz)`
    /// to resolve bound external fields.
    pub fn eval_f32(&self, x: f32, y: f32, z: f32, eval_input: &dyn Fn(usize, f32, f32, f32) -> f32) -> f32 {
        let run = |scratch: &mut Scratch<f32>| {
            scratch.reset(&self.slots);
            let pos = [x, y, z];
            exec_block(&self.ops, pos, scratch, &|idx, q| {
                eval_input(idx, q[0], q[1], q[2])
            });
            match scratch.slots[self.output_slot as usize] {
                Val::Scalar(v) => v,
                _ => f32::NAN,
            }
        };
        F32_SCRATCH.with(|cell| {
            if let Ok(mut guard) = cell.try_borrow_mut() {
                run(&mut guard)
            } else {
                let mut local = Scratch {
                    slots: Vec::new(),
                    stack: Vec::new(),
                };
                run(&mut local)
            }
        })
    }

    /// Evaluate value and forward-mode gradient `[df/dx, df/dy, df/dz]` at `(x, y, z)`.
    /// Returns `None` if the expression uses non-differentiable ops or hits a singularity.
    pub fn eval_dual(
        &self,
        x: f32,
        y: f32,
        z: f32,
        eval_input_dual: &dyn Fn(usize, [Dual3; 3]) -> Dual3,
    ) -> Option<(f32, [f32; 3])> {
        if self.has_non_diff {
            return None;
        }
        let run = |scratch: &mut Scratch<Dual3>| {
            scratch.reset(&self.slots);
            let pos = [Dual3::var(x, 0), Dual3::var(y, 1), Dual3::var(z, 2)];
            exec_block(&self.ops, pos, scratch, eval_input_dual);
            match scratch.slots[self.output_slot as usize] {
                Val::Scalar(d) if d.v.is_finite() && d.d.iter().all(|c| c.is_finite()) => {
                    Some((d.v, d.d))
                }
                _ => None,
            }
        };
        DUAL_SCRATCH.with(|cell| {
            if let Ok(mut guard) = cell.try_borrow_mut() {
                run(&mut guard)
            } else {
                let mut local = Scratch {
                    slots: Vec::new(),
                    stack: Vec::new(),
                };
                run(&mut local)
            }
        })
    }
}

#[derive(Clone, Copy)]
enum Val<S> {
    Scalar(S),
    Vec3([S; 3]),
    Bool(bool),
}

struct Scratch<S> {
    slots: Vec<Val<S>>,
    stack: Vec<Val<S>>,
}

impl<S: ScalarNum> Scratch<S> {
    fn reset(&mut self, slot_types: &[ExprType]) {
        self.slots.clear();
        self.stack.clear();
        for ty in slot_types {
            self.slots.push(match ty {
                ExprType::Scalar => Val::Scalar(S::constant(0.0)),
                ExprType::Vec3 => Val::Vec3([S::constant(0.0); 3]),
                ExprType::Bool => Val::Bool(false),
            });
        }
    }
}

thread_local! {
    static F32_SCRATCH: RefCell<Scratch<f32>> = const {
        RefCell::new(Scratch { slots: Vec::new(), stack: Vec::new() })
    };
    static DUAL_SCRATCH: RefCell<Scratch<Dual3>> = const {
        RefCell::new(Scratch { slots: Vec::new(), stack: Vec::new() })
    };
}

#[inline]
fn as_s<S: ScalarNum>(v: Val<S>) -> S {
    match v {
        Val::Scalar(s) => s,
        _ => S::constant(0.0),
    }
}

#[inline]
fn as_v<S: ScalarNum>(v: Val<S>) -> [S; 3] {
    match v {
        Val::Vec3(a) => a,
        _ => [S::constant(0.0); 3],
    }
}

#[inline]
fn as_b<S: ScalarNum>(v: Val<S>) -> bool {
    match v {
        Val::Bool(b) => b,
        _ => false,
    }
}

fn exec_block<S: ScalarNum>(
    ops: &[ExprOp],
    pos: [S; 3],
    st: &mut Scratch<S>,
    eval_input: &dyn Fn(usize, [S; 3]) -> S,
) -> bool {
    for op in ops {
        match op {
            ExprOp::PushScalar(v) => st.stack.push(Val::Scalar(S::constant(*v))),
            ExprOp::PushBool(b) => st.stack.push(Val::Bool(*b)),
            ExprOp::PushPos => st.stack.push(Val::Vec3(pos)),
            ExprOp::Load(slot) => st.stack.push(st.slots[*slot as usize]),
            ExprOp::Store(slot) => {
                if let Some(v) = st.stack.pop() {
                    st.slots[*slot as usize] = v;
                }
            }
            ExprOp::Pop => {
                st.stack.pop();
            }
            ExprOp::MakeVec3 => {
                let z = as_s(st.stack.pop().unwrap());
                let y = as_s(st.stack.pop().unwrap());
                let x = as_s(st.stack.pop().unwrap());
                st.stack.push(Val::Vec3([x, y, z]));
            }
            ExprOp::Unary(u) => {
                let a = st.stack.pop().unwrap();
                let res = match u {
                    UnaryInstr::NegScalar => Val::Scalar(as_s(a).neg()),
                    UnaryInstr::NegVec3 => {
                        let v = as_v(a);
                        Val::Vec3([v[0].neg(), v[1].neg(), v[2].neg()])
                    }
                    UnaryInstr::NotBool => Val::Bool(!as_b(a)),
                    UnaryInstr::AbsScalar => Val::Scalar(as_s(a).abs()),
                    UnaryInstr::AbsVec3 => {
                        let v = as_v(a);
                        Val::Vec3([v[0].abs(), v[1].abs(), v[2].abs()])
                    }
                    UnaryInstr::Sqrt => Val::Scalar(as_s(a).sqrt()),
                    UnaryInstr::Floor => Val::Scalar(as_s(a).floor()),
                    UnaryInstr::Ceil => Val::Scalar(as_s(a).ceil()),
                    UnaryInstr::Fract => Val::Scalar(as_s(a).fract()),
                    UnaryInstr::Sign => Val::Scalar(as_s(a).sign()),
                    UnaryInstr::Sin => Val::Scalar(as_s(a).sin()),
                    UnaryInstr::Cos => Val::Scalar(as_s(a).cos()),
                    UnaryInstr::Tan => Val::Scalar(as_s(a).tan()),
                    UnaryInstr::Asin => Val::Scalar(as_s(a).asin()),
                    UnaryInstr::Acos => Val::Scalar(as_s(a).acos()),
                    UnaryInstr::Ln => Val::Scalar(as_s(a).ln()),
                    UnaryInstr::Exp => Val::Scalar(as_s(a).exp()),
                    UnaryInstr::Saturate => {
                        Val::Scalar(as_s(a).clamp_scalar(S::constant(0.0), S::constant(1.0)))
                    }
                    UnaryInstr::VecX => Val::Scalar(as_v(a)[0]),
                    UnaryInstr::VecY => Val::Scalar(as_v(a)[1]),
                    UnaryInstr::VecZ => Val::Scalar(as_v(a)[2]),
                    UnaryInstr::Length => {
                        let v = as_v(a);
                        Val::Scalar(
                            v[0].mul(v[0])
                                .add(v[1].mul(v[1]))
                                .add(v[2].mul(v[2]))
                                .sqrt(),
                        )
                    }
                    UnaryInstr::Normalize => {
                        let v = as_v(a);
                        let l = v[0]
                            .mul(v[0])
                            .add(v[1].mul(v[1]))
                            .add(v[2].mul(v[2]))
                            .sqrt();
                        Val::Vec3([v[0].div(l), v[1].div(l), v[2].div(l)])
                    }
                };
                st.stack.push(res);
            }
            ExprOp::Binary(b) => {
                let rhs = st.stack.pop().unwrap();
                let lhs = st.stack.pop().unwrap();
                let res = match b {
                    BinaryInstr::AddScalar => Val::Scalar(as_s(lhs).add(as_s(rhs))),
                    BinaryInstr::SubScalar => Val::Scalar(as_s(lhs).sub(as_s(rhs))),
                    BinaryInstr::MulScalar => Val::Scalar(as_s(lhs).mul(as_s(rhs))),
                    BinaryInstr::DivScalar => Val::Scalar(as_s(lhs).div(as_s(rhs))),
                    BinaryInstr::RemScalar => Val::Scalar(as_s(lhs).rem(as_s(rhs))),
                    BinaryInstr::PowScalar => Val::Scalar(as_s(lhs).pow(as_s(rhs))),
                    BinaryInstr::MinScalar => Val::Scalar(as_s(lhs).min(as_s(rhs))),
                    BinaryInstr::MaxScalar => Val::Scalar(as_s(lhs).max(as_s(rhs))),
                    BinaryInstr::Atan2 => Val::Scalar(as_s(lhs).atan2(as_s(rhs))),
                    BinaryInstr::Step => {
                        let edge = as_s(lhs).primal();
                        let val = as_s(rhs).primal();
                        Val::Scalar(S::constant(if val < edge { 0.0 } else { 1.0 }))
                    }
                    BinaryInstr::AddVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].add(c[0]), a[1].add(c[1]), a[2].add(c[2])])
                    }
                    BinaryInstr::SubVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].sub(c[0]), a[1].sub(c[1]), a[2].sub(c[2])])
                    }
                    BinaryInstr::MulVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].mul(c[0]), a[1].mul(c[1]), a[2].mul(c[2])])
                    }
                    BinaryInstr::DivVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].div(c[0]), a[1].div(c[1]), a[2].div(c[2])])
                    }
                    BinaryInstr::ScaleVec3 => {
                        let a = as_v(lhs);
                        let s = as_s(rhs);
                        Val::Vec3([a[0].mul(s), a[1].mul(s), a[2].mul(s)])
                    }
                    BinaryInstr::MinVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].min(c[0]), a[1].min(c[1]), a[2].min(c[2])])
                    }
                    BinaryInstr::MaxVec3 => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([a[0].max(c[0]), a[1].max(c[1]), a[2].max(c[2])])
                    }
                    BinaryInstr::Dot => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Scalar(
                            a[0].mul(c[0])
                                .add(a[1].mul(c[1]))
                                .add(a[2].mul(c[2])),
                        )
                    }
                    BinaryInstr::Cross => {
                        let a = as_v(lhs);
                        let c = as_v(rhs);
                        Val::Vec3([
                            a[1].mul(c[2]).sub(a[2].mul(c[1])),
                            a[2].mul(c[0]).sub(a[0].mul(c[2])),
                            a[0].mul(c[1]).sub(a[1].mul(c[0])),
                        ])
                    }
                    BinaryInstr::Lt => Val::Bool(as_s(lhs).primal() < as_s(rhs).primal()),
                    BinaryInstr::Le => Val::Bool(as_s(lhs).primal() <= as_s(rhs).primal()),
                    BinaryInstr::Gt => Val::Bool(as_s(lhs).primal() > as_s(rhs).primal()),
                    BinaryInstr::Ge => Val::Bool(as_s(lhs).primal() >= as_s(rhs).primal()),
                    BinaryInstr::Eq => Val::Bool(as_s(lhs).primal() == as_s(rhs).primal()),
                    BinaryInstr::Ne => Val::Bool(as_s(lhs).primal() != as_s(rhs).primal()),
                    BinaryInstr::And => Val::Bool(as_b(lhs) && as_b(rhs)),
                    BinaryInstr::Or => Val::Bool(as_b(lhs) || as_b(rhs)),
                };
                st.stack.push(res);
            }
            ExprOp::Ternary(t) => {
                let c = st.stack.pop().unwrap();
                let b = st.stack.pop().unwrap();
                let a = st.stack.pop().unwrap();
                let res = match t {
                    TernaryInstr::ClampScalar => {
                        Val::Scalar(as_s(a).clamp_scalar(as_s(b), as_s(c)))
                    }
                    TernaryInstr::MixScalar => {
                        let va = as_s(a);
                        let vb = as_s(b);
                        let vt = as_s(c);
                        Val::Scalar(va.add(vb.sub(va).mul(vt)))
                    }
                    TernaryInstr::MixVec3 => {
                        let va = as_v(a);
                        let vb = as_v(b);
                        let vt = as_s(c);
                        Val::Vec3([
                            va[0].add(vb[0].sub(va[0]).mul(vt)),
                            va[1].add(vb[1].sub(va[1]).mul(vt)),
                            va[2].add(vb[2].sub(va[2]).mul(vt)),
                        ])
                    }
                    TernaryInstr::Smoothstep => {
                        let e0 = as_s(a);
                        let e1 = as_s(b);
                        let x = as_s(c);
                        let u = x
                            .sub(e0)
                            .div(e1.sub(e0))
                            .clamp_scalar(S::constant(0.0), S::constant(1.0));
                        let three = S::constant(3.0);
                        let two = S::constant(2.0);
                        Val::Scalar(u.mul(u).mul(three.sub(two.mul(u))))
                    }
                    TernaryInstr::Smin => Val::Scalar(as_s(a).smin(as_s(b), as_s(c))),
                    TernaryInstr::Smax => Val::Scalar(as_s(a).smax(as_s(b), as_s(c))),
                    TernaryInstr::SelectScalar => {
                        Val::Scalar(if as_b(a) { as_s(b) } else { as_s(c) })
                    }
                    TernaryInstr::SelectVec3 => {
                        Val::Vec3(if as_b(a) { as_v(b) } else { as_v(c) })
                    }
                };
                st.stack.push(res);
            }
            ExprOp::EvalInput(idx) => {
                let q = as_v(st.stack.pop().unwrap());
                st.stack.push(Val::Scalar(eval_input(*idx as usize, q)));
            }
            ExprOp::ValueNoise => {
                let seed = as_s(st.stack.pop().unwrap()).primal() as i32;
                let q = as_v(st.stack.pop().unwrap());
                let v = value_noise3(q[0].primal(), q[1].primal(), q[2].primal(), seed);
                st.stack.push(Val::Scalar(S::constant(v)));
            }
            ExprOp::Perlin => {
                let seed = as_s(st.stack.pop().unwrap()).primal() as i32;
                let q = as_v(st.stack.pop().unwrap());
                let v = perlin3(q[0].primal(), q[1].primal(), q[2].primal(), seed);
                st.stack.push(Val::Scalar(S::constant(v)));
            }
            ExprOp::Fbm { ridged } => {
                let octaves = as_s(st.stack.pop().unwrap()).primal().clamp(1.0, 8.0) as u32;
                let seed = as_s(st.stack.pop().unwrap()).primal() as i32;
                let freq = as_s(st.stack.pop().unwrap()).primal();
                let q = as_v(st.stack.pop().unwrap());
                let v = fbm3_params(
                    q[0].primal(),
                    q[1].primal(),
                    q[2].primal(),
                    seed,
                    freq,
                    octaves,
                    2.0,
                    0.5,
                    true,
                    *ridged,
                );
                st.stack.push(Val::Scalar(S::constant(v)));
            }
            ExprOp::Worley => {
                let seed = as_s(st.stack.pop().unwrap()).primal() as i32;
                let freq = as_s(st.stack.pop().unwrap()).primal();
                let q = as_v(st.stack.pop().unwrap());
                let v = worley3(
                    q[0].primal(),
                    q[1].primal(),
                    q[2].primal(),
                    freq,
                    seed,
                    1.0,
                    CellMode::F1,
                );
                st.stack.push(Val::Scalar(S::constant(v)));
            }
            ExprOp::PrimSphere => {
                let r = as_s(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let len = q[0]
                    .mul(q[0])
                    .add(q[1].mul(q[1]))
                    .add(q[2].mul(q[2]))
                    .sqrt();
                st.stack.push(Val::Scalar(len.sub(r)));
            }
            ExprOp::PrimBox => {
                let b = as_v(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let zero = S::constant(0.0);
                let qx = q[0].abs().sub(b[0]);
                let qy = q[1].abs().sub(b[1]);
                let qz = q[2].abs().sub(b[2]);
                let ox = qx.max(zero);
                let oy = qy.max(zero);
                let oz = qz.max(zero);
                let out = ox.mul(ox).add(oy.mul(oy)).add(oz.mul(oz)).sqrt();
                let inside = qx.max(qy.max(qz)).min(zero);
                st.stack.push(Val::Scalar(out.add(inside)));
            }
            ExprOp::PrimTorus => {
                let r_min = as_s(st.stack.pop().unwrap());
                let r_maj = as_s(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let pxz = q[0].mul(q[0]).add(q[2].mul(q[2])).sqrt().sub(r_maj);
                let d = pxz.mul(pxz).add(q[1].mul(q[1])).sqrt().sub(r_min);
                st.stack.push(Val::Scalar(d));
            }
            ExprOp::PrimCylinder => {
                let h = as_s(st.stack.pop().unwrap());
                let r = as_s(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let zero = S::constant(0.0);
                let dx = q[0].mul(q[0]).add(q[2].mul(q[2])).sqrt().sub(r);
                let dy = q[1].abs().sub(h);
                let ox = dx.max(zero);
                let oy = dy.max(zero);
                let d = dx.max(dy).min(zero).add(ox.mul(ox).add(oy.mul(oy)).sqrt());
                st.stack.push(Val::Scalar(d));
            }
            ExprOp::PrimCapsule => {
                let r = as_s(st.stack.pop().unwrap());
                let b = as_v(st.stack.pop().unwrap());
                let a = as_v(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let pa = [q[0].sub(a[0]), q[1].sub(a[1]), q[2].sub(a[2])];
                let ba = [b[0].sub(a[0]), b[1].sub(a[1]), b[2].sub(a[2])];
                let ba2 = ba[0].mul(ba[0]).add(ba[1].mul(ba[1])).add(ba[2].mul(ba[2]));
                let h = if ba2.primal() > 1e-12 {
                    pa[0]
                        .mul(ba[0])
                        .add(pa[1].mul(ba[1]))
                        .add(pa[2].mul(ba[2]))
                        .div(ba2)
                        .clamp_scalar(S::constant(0.0), S::constant(1.0))
                } else {
                    S::constant(0.0)
                };
                let dx = pa[0].sub(ba[0].mul(h));
                let dy = pa[1].sub(ba[1].mul(h));
                let dz = pa[2].sub(ba[2].mul(h));
                let d = dx.mul(dx).add(dy.mul(dy)).add(dz.mul(dz)).sqrt().sub(r);
                st.stack.push(Val::Scalar(d));
            }
            ExprOp::PrimPlane => {
                let off = as_s(st.stack.pop().unwrap());
                let n = as_v(st.stack.pop().unwrap());
                let q = as_v(st.stack.pop().unwrap());
                let d = q[0]
                    .mul(n[0])
                    .add(q[1].mul(n[1]))
                    .add(q[2].mul(n[2]))
                    .add(off);
                st.stack.push(Val::Scalar(d));
            }
            ExprOp::Repeat {
                count,
                iter_slot,
                body,
            } => {
                for iter in 0..*count {
                    if let Some(slot) = iter_slot {
                        st.slots[*slot as usize] = Val::Scalar(S::constant(iter as f32));
                    }
                    if exec_block(body, pos, st, eval_input) {
                        break;
                    }
                }
            }
            ExprOp::BreakIf => {
                if as_b(st.stack.pop().unwrap()) {
                    return true;
                }
            }
        }
    }
    false
}

// ── Tokenizer ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f32),
    Ident(String),
    Let,
    For,
    In,
    If,
    Else,
    Break,
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    Ne,
    AndAnd,
    OrOr,
    Bang,
    Eq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Dot,
    DotDot,
    Eof,
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let bytes = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        // Line comment `//` or `#`
        if (c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/') || c == b'#' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit()
            || (c == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
        {
            let start = i;
            let mut saw_dot = false;
            while i < bytes.len() {
                if bytes[i].is_ascii_digit() {
                    i += 1;
                } else if bytes[i] == b'.'
                    && !saw_dot
                    && !(i + 1 < bytes.len() && bytes[i + 1] == b'.')
                {
                    saw_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
                i += 1;
                if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                    i += 1;
                }
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let num: f32 = src[start..i]
                .parse()
                .map_err(|_| format!("invalid number literal '{}'", &src[start..i]))?;
            if !num.is_finite() {
                return Err("number literal must be finite".into());
            }
            out.push(Tok::Num(num));
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &src[start..i];
            out.push(match word {
                "let" => Tok::Let,
                "for" => Tok::For,
                "in" => Tok::In,
                "if" => Tok::If,
                "else" => Tok::Else,
                "break" => Tok::Break,
                "true" => Tok::True,
                "false" => Tok::False,
                "pi" | "PI" => Tok::Num(std::f32::consts::PI),
                "tau" | "TAU" => Tok::Num(std::f32::consts::TAU),
                _ => Tok::Ident(word.to_string()),
            });
            continue;
        }
        let next = bytes.get(i + 1).copied();
        match (c, next) {
            (b'.', Some(b'.')) => {
                out.push(Tok::DotDot);
                i += 2;
            }
            (b'<', Some(b'=')) => {
                out.push(Tok::Le);
                i += 2;
            }
            (b'>', Some(b'=')) => {
                out.push(Tok::Ge);
                i += 2;
            }
            (b'=', Some(b'=')) => {
                out.push(Tok::EqEq);
                i += 2;
            }
            (b'!', Some(b'=')) => {
                out.push(Tok::Ne);
                i += 2;
            }
            (b'&', Some(b'&')) => {
                out.push(Tok::AndAnd);
                i += 2;
            }
            (b'|', Some(b'|')) => {
                out.push(Tok::OrOr);
                i += 2;
            }
            (b'+', _) => {
                out.push(Tok::Plus);
                i += 1;
            }
            (b'-', _) => {
                out.push(Tok::Minus);
                i += 1;
            }
            (b'*', _) => {
                out.push(Tok::Star);
                i += 1;
            }
            (b'/', _) => {
                out.push(Tok::Slash);
                i += 1;
            }
            (b'%', _) => {
                out.push(Tok::Percent);
                i += 1;
            }
            (b'<', _) => {
                out.push(Tok::Lt);
                i += 1;
            }
            (b'>', _) => {
                out.push(Tok::Gt);
                i += 1;
            }
            (b'!', _) => {
                out.push(Tok::Bang);
                i += 1;
            }
            (b'=', _) => {
                out.push(Tok::Eq);
                i += 1;
            }
            (b'(', _) => {
                out.push(Tok::LParen);
                i += 1;
            }
            (b')', _) => {
                out.push(Tok::RParen);
                i += 1;
            }
            (b'{', _) => {
                out.push(Tok::LBrace);
                i += 1;
            }
            (b'}', _) => {
                out.push(Tok::RBrace);
                i += 1;
            }
            (b',', _) => {
                out.push(Tok::Comma);
                i += 1;
            }
            (b';', _) => {
                out.push(Tok::Semi);
                i += 1;
            }
            (b'.', _) => {
                out.push(Tok::Dot);
                i += 1;
            }
            _ => return Err(format!("unexpected character '{}'", c as char)),
        }
    }
    out.push(Tok::Eof);
    Ok(out)
}

// ── Parser & Type-Checked Codegen ────────────────────────────────────────────

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
    input_names: Vec<String>,
    inputs: HashMap<String, u16>,
    slots: Vec<ExprType>,
    scopes: Vec<HashMap<String, (u16, ExprType)>>,
    loop_depth: usize,
    has_non_diff: bool,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Tok], input_names: &[String]) -> Self {
        let mut inputs = HashMap::new();
        for (i, name) in input_names.iter().enumerate() {
            inputs.insert(name.clone(), i as u16);
        }
        Self {
            tokens,
            pos: 0,
            input_names: input_names.to_vec(),
            inputs,
            slots: Vec::new(),
            scopes: vec![HashMap::new()],
            loop_depth: 0,
            has_non_diff: false,
        }
    }

    fn peek(&self) -> &Tok {
        self.tokens.get(self.pos).unwrap_or(&Tok::Eof)
    }

    fn peek_at(&self, offset: usize) -> &Tok {
        self.tokens.get(self.pos + offset).unwrap_or(&Tok::Eof)
    }

    fn bump(&mut self) -> Tok {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Tok) -> Result<(), String> {
        if self.peek() == expected {
            self.bump();
            Ok(())
        } else {
            Err(format!("expected {expected:?}, found {:?}", self.peek()))
        }
    }

    fn alloc_slot(&mut self, ty: ExprType) -> Result<u16, String> {
        if self.slots.len() >= MAX_EXPR_SLOTS {
            return Err(format!("expression exceeds {MAX_EXPR_SLOTS} local slots"));
        }
        let id = self.slots.len() as u16;
        self.slots.push(ty);
        Ok(id)
    }

    fn lookup_var(&self, name: &str) -> Option<(u16, ExprType)> {
        for scope in self.scopes.iter().rev() {
            if let Some(&v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn parse_program(&mut self, source: &str) -> Result<ExprProgram, String> {
        let mut ops = Vec::new();
        let out_ty = self.parse_block_contents(&mut ops, true)?;
        if out_ty != ExprType::Scalar {
            return Err(format!(
                "expression must evaluate to Scalar, got {out_ty:?}"
            ));
        }
        let output_slot = self.alloc_slot(ExprType::Scalar)?;
        ops.push(ExprOp::Store(output_slot));
        if *self.peek() != Tok::Eof {
            return Err(format!("unexpected trailing token {:?}", self.peek()));
        }
        Ok(ExprProgram {
            source: source.to_string(),
            input_names: self.input_names.clone(),
            slots: self.slots.clone(),
            ops,
            output_slot,
            has_non_diff: self.has_non_diff,
        })
    }

    /// Parses statements inside a block. If `expect_value` is true, expects a
    /// final expression leaving one value on the stack and returns its type.
    fn parse_block_contents(
        &mut self,
        ops: &mut Vec<ExprOp>,
        expect_value: bool,
    ) -> Result<ExprType, String> {
        let mut final_ty: Option<ExprType> = None;
        while !matches!(self.peek(), Tok::RBrace | Tok::Eof) {
            if final_ty.is_some() {
                return Err("missing ';' between statements".into());
            }
            match self.peek() {
                Tok::Let => {
                    self.bump();
                    let name = match self.bump() {
                        Tok::Ident(s) => s,
                        t => return Err(format!("expected identifier after 'let', got {t:?}")),
                    };
                    self.expect(&Tok::Eq)?;
                    let ty = self.parse_expr(ops)?;
                    self.expect(&Tok::Semi)?;
                    let slot = self.alloc_slot(ty)?;
                    ops.push(ExprOp::Store(slot));
                    self.scopes.last_mut().unwrap().insert(name, (slot, ty));
                }
                Tok::For => {
                    self.parse_for_loop(ops)?;
                }
                Tok::Break => {
                    if self.loop_depth == 0 {
                        return Err("'break' used outside a loop".into());
                    }
                    self.bump();
                    self.expect(&Tok::Semi)?;
                    ops.push(ExprOp::PushBool(true));
                    ops.push(ExprOp::BreakIf);
                }
                Tok::Ident(name) if *self.peek_at(1) == Tok::Eq => {
                    let name = name.clone();
                    let (slot, expected_ty) = self
                        .lookup_var(&name)
                        .ok_or_else(|| format!("assignment to undeclared variable '{name}'"))?;
                    self.bump(); // ident
                    self.bump(); // =
                    let ty = self.parse_expr(ops)?;
                    if ty != expected_ty {
                        return Err(format!(
                            "cannot assign {ty:?} to variable '{name}' of type {expected_ty:?}"
                        ));
                    }
                    self.expect(&Tok::Semi)?;
                    ops.push(ExprOp::Store(slot));
                }
                Tok::If if self.is_if_break_stmt() => {
                    self.parse_if_break(ops)?;
                }
                _ => {
                    let ty = self.parse_expr(ops)?;
                    if *self.peek() == Tok::Semi {
                        self.bump();
                        ops.push(ExprOp::Pop);
                    } else if expect_value {
                        final_ty = Some(ty);
                    } else {
                        return Err("statement inside loop body must end with ';'".into());
                    }
                }
            }
        }
        if expect_value {
            final_ty.ok_or_else(|| "block must end with a value expression".into())
        } else {
            Ok(ExprType::Scalar)
        }
    }

    fn is_if_break_stmt(&self) -> bool {
        // Look ahead for `if <cond> { break ; }` without `else`
        let mut i = self.pos + 1;
        let mut paren = 0usize;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Tok::LParen => paren += 1,
                Tok::RParen => paren = paren.saturating_sub(1),
                Tok::LBrace if paren == 0 => {
                    return matches!(
                        self.tokens.get(i + 1),
                        Some(Tok::Break)
                    );
                }
                Tok::Eof => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn parse_if_break(&mut self, ops: &mut Vec<ExprOp>) -> Result<(), String> {
        if self.loop_depth == 0 {
            return Err("'if ... { break; }' used outside a loop".into());
        }
        self.expect(&Tok::If)?;
        let cond_ty = self.parse_expr(ops)?;
        if cond_ty != ExprType::Bool {
            return Err(format!("'if' condition must be Bool, got {cond_ty:?}"));
        }
        self.expect(&Tok::LBrace)?;
        self.expect(&Tok::Break)?;
        if *self.peek() == Tok::Semi {
            self.bump();
        }
        self.expect(&Tok::RBrace)?;
        ops.push(ExprOp::BreakIf);
        Ok(())
    }

    fn parse_for_loop(&mut self, ops: &mut Vec<ExprOp>) -> Result<(), String> {
        self.expect(&Tok::For)?;
        let iter_name = match self.bump() {
            Tok::Ident(s) => s,
            t => return Err(format!("expected loop variable name, got {t:?}")),
        };
        self.expect(&Tok::In)?;
        let start = match self.bump() {
            Tok::Num(v) if v == 0.0 => 0u32,
            t => return Err(format!("loop range must start at 0 (0..N), got {t:?}")),
        };
        self.expect(&Tok::DotDot)?;
        let end = match self.bump() {
            Tok::Num(v)
                if v.fract() == 0.0 && v >= 1.0 && v <= MAX_EXPR_REPEAT_ITERATIONS as f32 =>
            {
                v as u32
            }
            t => {
                return Err(format!(
                    "loop upper bound must be integer in 1..={MAX_EXPR_REPEAT_ITERATIONS}, got {t:?}"
                ))
            }
        };
        let count = end - start;
        self.expect(&Tok::LBrace)?;
        let iter_slot = if iter_name == "_" {
            None
        } else {
            Some(self.alloc_slot(ExprType::Scalar)?)
        };
        self.scopes.push(HashMap::new());
        if let Some(slot) = iter_slot {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(iter_name, (slot, ExprType::Scalar));
        }
        self.loop_depth += 1;
        let mut body = Vec::new();
        self.parse_block_contents(&mut body, false)?;
        self.loop_depth -= 1;
        self.scopes.pop();
        self.expect(&Tok::RBrace)?;
        ops.push(ExprOp::Repeat {
            count,
            iter_slot,
            body,
        });
        Ok(())
    }

    fn parse_expr(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        if *self.peek() == Tok::If {
            return self.parse_if_expr(ops);
        }
        self.parse_or(ops)
    }

    fn parse_if_expr(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        self.expect(&Tok::If)?;
        let cond_ty = self.parse_expr(ops)?;
        if cond_ty != ExprType::Bool {
            return Err(format!("if condition must be Bool, got {cond_ty:?}"));
        }
        self.expect(&Tok::LBrace)?;
        self.scopes.push(HashMap::new());
        let then_ty = self.parse_block_contents(ops, true)?;
        self.scopes.pop();
        self.expect(&Tok::RBrace)?;
        self.expect(&Tok::Else)?;
        self.expect(&Tok::LBrace)?;
        self.scopes.push(HashMap::new());
        let else_ty = self.parse_block_contents(ops, true)?;
        self.scopes.pop();
        self.expect(&Tok::RBrace)?;
        if then_ty != else_ty {
            return Err(format!(
                "if/else branches must have matching types ({then_ty:?} vs {else_ty:?})"
            ));
        }
        match then_ty {
            ExprType::Scalar => ops.push(ExprOp::Ternary(TernaryInstr::SelectScalar)),
            ExprType::Vec3 => ops.push(ExprOp::Ternary(TernaryInstr::SelectVec3)),
            ExprType::Bool => return Err("if/else cannot return Bool".into()),
        }
        Ok(then_ty)
    }

    fn parse_or(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let mut lhs = self.parse_and(ops)?;
        while *self.peek() == Tok::OrOr {
            self.bump();
            let rhs = self.parse_and(ops)?;
            if lhs != ExprType::Bool || rhs != ExprType::Bool {
                return Err("'||' requires Bool operands".into());
            }
            ops.push(ExprOp::Binary(BinaryInstr::Or));
            lhs = ExprType::Bool;
        }
        Ok(lhs)
    }

    fn parse_and(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let mut lhs = self.parse_cmp(ops)?;
        while *self.peek() == Tok::AndAnd {
            self.bump();
            let rhs = self.parse_cmp(ops)?;
            if lhs != ExprType::Bool || rhs != ExprType::Bool {
                return Err("'&&' requires Bool operands".into());
            }
            ops.push(ExprOp::Binary(BinaryInstr::And));
            lhs = ExprType::Bool;
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let lhs = self.parse_add(ops)?;
        let bin = match self.peek() {
            Tok::Lt => BinaryInstr::Lt,
            Tok::Le => BinaryInstr::Le,
            Tok::Gt => BinaryInstr::Gt,
            Tok::Ge => BinaryInstr::Ge,
            Tok::EqEq => BinaryInstr::Eq,
            Tok::Ne => BinaryInstr::Ne,
            _ => return Ok(lhs),
        };
        self.bump();
        let rhs = self.parse_add(ops)?;
        if lhs != ExprType::Scalar || rhs != ExprType::Scalar {
            return Err("comparison operators require Scalar operands".into());
        }
        ops.push(ExprOp::Binary(bin));
        Ok(ExprType::Bool)
    }

    fn parse_add(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let mut lhs = self.parse_mul(ops)?;
        loop {
            let is_add = match self.peek() {
                Tok::Plus => true,
                Tok::Minus => false,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_mul(ops)?;
            match (lhs, rhs) {
                (ExprType::Scalar, ExprType::Scalar) => {
                    ops.push(ExprOp::Binary(if is_add {
                        BinaryInstr::AddScalar
                    } else {
                        BinaryInstr::SubScalar
                    }));
                }
                (ExprType::Vec3, ExprType::Vec3) => {
                    ops.push(ExprOp::Binary(if is_add {
                        BinaryInstr::AddVec3
                    } else {
                        BinaryInstr::SubVec3
                    }));
                }
                _ => return Err(format!("cannot add/sub {lhs:?} and {rhs:?}")),
            }
            lhs = rhs;
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let mut lhs = self.parse_unary(ops)?;
        loop {
            let op = match self.peek() {
                Tok::Star => 0,
                Tok::Slash => 1,
                Tok::Percent => 2,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_unary(ops)?;
            lhs = match (op, lhs, rhs) {
                (0, ExprType::Scalar, ExprType::Scalar) => {
                    ops.push(ExprOp::Binary(BinaryInstr::MulScalar));
                    ExprType::Scalar
                }
                (0, ExprType::Vec3, ExprType::Vec3) => {
                    ops.push(ExprOp::Binary(BinaryInstr::MulVec3));
                    ExprType::Vec3
                }
                (0, ExprType::Vec3, ExprType::Scalar) => {
                    ops.push(ExprOp::Binary(BinaryInstr::ScaleVec3));
                    ExprType::Vec3
                }
                (0, ExprType::Scalar, ExprType::Vec3) => {
                    // Swap scalar * vec3 via temp slots
                    let vs = self.alloc_slot(ExprType::Vec3)?;
                    let ss = self.alloc_slot(ExprType::Scalar)?;
                    ops.push(ExprOp::Store(vs));
                    ops.push(ExprOp::Store(ss));
                    ops.push(ExprOp::Load(vs));
                    ops.push(ExprOp::Load(ss));
                    ops.push(ExprOp::Binary(BinaryInstr::ScaleVec3));
                    ExprType::Vec3
                }
                (1, ExprType::Scalar, ExprType::Scalar) => {
                    ops.push(ExprOp::Binary(BinaryInstr::DivScalar));
                    ExprType::Scalar
                }
                (1, ExprType::Vec3, ExprType::Vec3) => {
                    ops.push(ExprOp::Binary(BinaryInstr::DivVec3));
                    ExprType::Vec3
                }
                (1, ExprType::Vec3, ExprType::Scalar) => {
                    ops.push(ExprOp::PushScalar(1.0));
                    // compute 1.0 / s via swap
                    let ss = self.alloc_slot(ExprType::Scalar)?;
                    ops.pop(); // remove PushScalar(1.0)
                    ops.push(ExprOp::Store(ss));
                    ops.push(ExprOp::PushScalar(1.0));
                    ops.push(ExprOp::Load(ss));
                    ops.push(ExprOp::Binary(BinaryInstr::DivScalar));
                    ops.push(ExprOp::Binary(BinaryInstr::ScaleVec3));
                    ExprType::Vec3
                }
                (2, ExprType::Scalar, ExprType::Scalar) => {
                    ops.push(ExprOp::Binary(BinaryInstr::RemScalar));
                    ExprType::Scalar
                }
                _ => return Err(format!("invalid mul/div/rem operands {lhs:?} and {rhs:?}")),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        match self.peek() {
            Tok::Minus => {
                self.bump();
                let ty = self.parse_unary(ops)?;
                match ty {
                    ExprType::Scalar => ops.push(ExprOp::Unary(UnaryInstr::NegScalar)),
                    ExprType::Vec3 => ops.push(ExprOp::Unary(UnaryInstr::NegVec3)),
                    ExprType::Bool => return Err("cannot negate Bool".into()),
                }
                Ok(ty)
            }
            Tok::Bang => {
                self.bump();
                let ty = self.parse_unary(ops)?;
                if ty != ExprType::Bool {
                    return Err("'!' requires Bool".into());
                }
                ops.push(ExprOp::Unary(UnaryInstr::NotBool));
                Ok(ExprType::Bool)
            }
            _ => self.parse_postfix(ops),
        }
    }

    fn parse_postfix(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        let mut ty = self.parse_primary(ops)?;
        while *self.peek() == Tok::Dot {
            self.bump();
            let field = match self.bump() {
                Tok::Ident(s) => s,
                t => return Err(format!("expected component name after '.', got {t:?}")),
            };
            if ty != ExprType::Vec3 {
                return Err(format!("cannot access '.{field}' on {ty:?}"));
            }
            match field.as_str() {
                "x" => ops.push(ExprOp::Unary(UnaryInstr::VecX)),
                "y" => ops.push(ExprOp::Unary(UnaryInstr::VecY)),
                "z" => ops.push(ExprOp::Unary(UnaryInstr::VecZ)),
                _ => return Err(format!("unknown vec3 component '.{field}'")),
            }
            ty = ExprType::Scalar;
        }
        Ok(ty)
    }

    fn parse_primary(&mut self, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        match self.bump() {
            Tok::Num(v) => {
                ops.push(ExprOp::PushScalar(v));
                Ok(ExprType::Scalar)
            }
            Tok::True => {
                ops.push(ExprOp::PushBool(true));
                Ok(ExprType::Bool)
            }
            Tok::False => {
                ops.push(ExprOp::PushBool(false));
                Ok(ExprType::Bool)
            }
            Tok::LParen => {
                let ty = self.parse_expr(ops)?;
                self.expect(&Tok::RParen)?;
                Ok(ty)
            }
            Tok::Ident(name) => {
                if *self.peek() == Tok::LParen {
                    self.parse_call(&name, ops)
                } else if let Some((slot, ty)) = self.lookup_var(&name) {
                    ops.push(ExprOp::Load(slot));
                    Ok(ty)
                } else if let Some(&idx) = self.inputs.get(&name) {
                    ops.push(ExprOp::PushPos);
                    ops.push(ExprOp::EvalInput(idx));
                    Ok(ExprType::Scalar)
                } else {
                    match name.as_str() {
                        "p" => {
                            ops.push(ExprOp::PushPos);
                            Ok(ExprType::Vec3)
                        }
                        "x" => {
                            ops.push(ExprOp::PushPos);
                            ops.push(ExprOp::Unary(UnaryInstr::VecX));
                            Ok(ExprType::Scalar)
                        }
                        "y" => {
                            ops.push(ExprOp::PushPos);
                            ops.push(ExprOp::Unary(UnaryInstr::VecY));
                            Ok(ExprType::Scalar)
                        }
                        "z" => {
                            ops.push(ExprOp::PushPos);
                            ops.push(ExprOp::Unary(UnaryInstr::VecZ));
                            Ok(ExprType::Scalar)
                        }
                        _ => Err(format!("unknown identifier '{name}'")),
                    }
                }
            }
            t => Err(format!("unexpected token {t:?} in expression")),
        }
    }

    fn parse_args(&mut self, ops: &mut Vec<ExprOp>) -> Result<Vec<ExprType>, String> {
        self.expect(&Tok::LParen)?;
        let mut types = Vec::new();
        if *self.peek() != Tok::RParen {
            loop {
                types.push(self.parse_expr(ops)?);
                if *self.peek() == Tok::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(types)
    }

    fn parse_call(&mut self, name: &str, ops: &mut Vec<ExprOp>) -> Result<ExprType, String> {
        // Calling a bound external field at a transformed Vec3 position: `my_field(q)`
        if let Some(&idx) = self.inputs.get(name) {
            let args = self.parse_args(ops)?;
            if args != [ExprType::Vec3] {
                return Err(format!(
                    "bound field '{name}(q)' expects 1 Vec3 argument, got {args:?}"
                ));
            }
            ops.push(ExprOp::EvalInput(idx));
            return Ok(ExprType::Scalar);
        }

        let args = self.parse_args(ops)?;
        use ExprType::*;
        match (name, args.as_slice()) {
            ("vec3", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::MakeVec3);
                Ok(Vec3)
            }
            ("abs", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::AbsScalar));
                Ok(Scalar)
            }
            ("abs", [Vec3]) => {
                ops.push(ExprOp::Unary(UnaryInstr::AbsVec3));
                Ok(Vec3)
            }
            ("sqrt", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Sqrt));
                Ok(Scalar)
            }
            ("floor", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Floor));
                Ok(Scalar)
            }
            ("ceil", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Ceil));
                Ok(Scalar)
            }
            ("fract", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Fract));
                Ok(Scalar)
            }
            ("sign", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Sign));
                Ok(Scalar)
            }
            ("sin", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Sin));
                Ok(Scalar)
            }
            ("cos", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Cos));
                Ok(Scalar)
            }
            ("tan", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Tan));
                Ok(Scalar)
            }
            ("asin", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Asin));
                Ok(Scalar)
            }
            ("acos", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Acos));
                Ok(Scalar)
            }
            ("ln" | "log", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Ln));
                Ok(Scalar)
            }
            ("exp", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Exp));
                Ok(Scalar)
            }
            ("saturate", [Scalar]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Saturate));
                Ok(Scalar)
            }
            ("length", [Vec3]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Length));
                Ok(Scalar)
            }
            ("normalize", [Vec3]) => {
                ops.push(ExprOp::Unary(UnaryInstr::Normalize));
                Ok(Vec3)
            }
            ("min", [Scalar, Scalar]) => {
                ops.push(ExprOp::Binary(BinaryInstr::MinScalar));
                Ok(Scalar)
            }
            ("min", [Vec3, Vec3]) => {
                ops.push(ExprOp::Binary(BinaryInstr::MinVec3));
                Ok(Vec3)
            }
            ("max", [Scalar, Scalar]) => {
                ops.push(ExprOp::Binary(BinaryInstr::MaxScalar));
                Ok(Scalar)
            }
            ("max", [Vec3, Vec3]) => {
                ops.push(ExprOp::Binary(BinaryInstr::MaxVec3));
                Ok(Vec3)
            }
            ("pow", [Scalar, Scalar]) => {
                ops.push(ExprOp::Binary(BinaryInstr::PowScalar));
                Ok(Scalar)
            }
            ("atan2", [Scalar, Scalar]) => {
                ops.push(ExprOp::Binary(BinaryInstr::Atan2));
                Ok(Scalar)
            }
            ("step", [Scalar, Scalar]) => {
                ops.push(ExprOp::Binary(BinaryInstr::Step));
                Ok(Scalar)
            }
            ("dot", [Vec3, Vec3]) => {
                ops.push(ExprOp::Binary(BinaryInstr::Dot));
                Ok(Scalar)
            }
            ("cross", [Vec3, Vec3]) => {
                ops.push(ExprOp::Binary(BinaryInstr::Cross));
                Ok(Vec3)
            }
            ("clamp", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::ClampScalar));
                Ok(Scalar)
            }
            ("mix" | "lerp", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::MixScalar));
                Ok(Scalar)
            }
            ("mix" | "lerp", [Vec3, Vec3, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::MixVec3));
                Ok(Vec3)
            }
            ("smoothstep", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::Smoothstep));
                Ok(Scalar)
            }
            ("smin", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::Smin));
                Ok(Scalar)
            }
            ("smax", [Scalar, Scalar, Scalar]) => {
                ops.push(ExprOp::Ternary(TernaryInstr::Smax));
                Ok(Scalar)
            }
            ("value_noise", [Vec3, Scalar]) => {
                self.has_non_diff = true;
                ops.push(ExprOp::ValueNoise);
                Ok(Scalar)
            }
            ("perlin", [Vec3, Scalar]) => {
                self.has_non_diff = true;
                ops.push(ExprOp::Perlin);
                Ok(Scalar)
            }
            ("fbm", [Vec3, Scalar, Scalar, Scalar]) => {
                self.has_non_diff = true;
                ops.push(ExprOp::Fbm { ridged: false });
                Ok(Scalar)
            }
            ("ridged", [Vec3, Scalar, Scalar, Scalar]) => {
                self.has_non_diff = true;
                ops.push(ExprOp::Fbm { ridged: true });
                Ok(Scalar)
            }
            ("worley", [Vec3, Scalar, Scalar]) => {
                self.has_non_diff = true;
                ops.push(ExprOp::Worley);
                Ok(Scalar)
            }
            ("sphere", [Vec3, Scalar]) => {
                ops.push(ExprOp::PrimSphere);
                Ok(Scalar)
            }
            ("box", [Vec3, Vec3]) => {
                ops.push(ExprOp::PrimBox);
                Ok(Scalar)
            }
            ("torus", [Vec3, Scalar, Scalar]) => {
                ops.push(ExprOp::PrimTorus);
                Ok(Scalar)
            }
            ("cylinder", [Vec3, Scalar, Scalar]) => {
                ops.push(ExprOp::PrimCylinder);
                Ok(Scalar)
            }
            ("capsule", [Vec3, Vec3, Vec3, Scalar]) => {
                ops.push(ExprOp::PrimCapsule);
                Ok(Scalar)
            }
            ("plane", [Vec3, Vec3, Scalar]) => {
                ops.push(ExprOp::PrimPlane);
                Ok(Scalar)
            }
            _ => Err(format!(
                "unknown function '{name}' or mismatched argument types {args:?}"
            )),
        }
    }
}
