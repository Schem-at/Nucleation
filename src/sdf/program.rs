//! Portable, sandboxed SDF field program: a serde-serializable, deterministic
//! stack-based typed bytecode VM.
//!
//! A [`Program`] is a flat list of typed [`Instr`]uctions operating on an
//! explicit value stack (scalar / vec3 / bool) plus a fixed set of typed
//! local slots, closed over by a statically bounded [`Instr::Repeat`] block
//! (with [`Instr::BreakIf`] for early exit) instead of arbitrary jumps — this
//! keeps every program provably terminating without a runtime step counter.
//! Every [`Program`] in existence has already passed [`validate`]: the only
//! entry points are [`Program::from_json`] and [`ProgramBuilder::build`],
//! both of which run it, so malformed or resource-exhausting programs never
//! reach the evaluator. Evaluation is deterministic pure `f32` arithmetic
//! (IEEE 754 semantics — degenerate inputs like `0/0` produce `NaN`, never a
//! panic). [`Program::analytic_gradient`] runs the same interpreter over a
//! forward-mode dual number to get an exact gradient where the program is
//! differentiable at that point, and returns `None` at singularities (e.g.
//! `length` of the zero vector) so callers can fall back to
//! [`super::numerical_normal`].

use super::Aabb;
use serde::{Deserialize, Serialize};

/// Maximum number of typed local slots a program may declare.
pub const MAX_SLOTS: usize = 64;
/// Maximum number of `Instr` nodes in a program's syntax tree (counting
/// every instruction nested inside `Repeat` bodies once, not per-iteration).
pub const MAX_STATIC_INSTRUCTIONS: usize = 2048;
/// Maximum nesting depth of `Repeat` blocks.
pub const MAX_REPEAT_DEPTH: usize = 8;
/// Maximum static iteration count of a single `Repeat` node.
pub const MAX_REPEAT_ITERATIONS: u32 = 4096;
/// Upper bound on worst-case total instruction executions across nested
/// repeats (`sum(instructions_in_block * product(enclosing repeat counts))`),
/// checked at validation time so evaluation is always provably bounded.
pub const MAX_DYNAMIC_STEPS: u64 = 200_000;

/// The type of a value flowing through the program's stack or held in a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueType {
    Scalar,
    Vec3,
    Bool,
}

/// A finite literal pushed by [`Instr::PushConst`]. Untagged: a JSON number,
/// a 3-element array, or a boolean unambiguously identify the variant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Const {
    Scalar(f32),
    Vec3([f32; 3]),
    Bool(bool),
}

impl Const {
    fn value_type(self) -> ValueType {
        match self {
            Const::Scalar(_) => ValueType::Scalar,
            Const::Vec3(_) => ValueType::Vec3,
            Const::Bool(_) => ValueType::Bool,
        }
    }

    fn is_finite(self) -> bool {
        match self {
            Const::Scalar(v) => v.is_finite(),
            Const::Vec3(v) => v.iter().all(|c| c.is_finite()),
            Const::Bool(_) => true,
        }
    }
}

/// Unary scalar/vec3 operations. All pop one value and push one value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnaryOp {
    Neg,
    Abs,
    Sqrt,
    Log,
    Sin,
    Cos,
    Acos,
    VecX,
    VecY,
    VecZ,
    Length,
    Normalize,
}

impl UnaryOp {
    fn signature(self) -> (ValueType, ValueType) {
        use UnaryOp::*;
        match self {
            Neg | Abs | Sqrt | Log | Sin | Cos | Acos => (ValueType::Scalar, ValueType::Scalar),
            VecX | VecY | VecZ | Length => (ValueType::Vec3, ValueType::Scalar),
            Normalize => (ValueType::Vec3, ValueType::Vec3),
        }
    }
}

/// Binary operations. `Add`/`Sub` are polymorphic over `Scalar` or `Vec3`
/// (both operands must agree); every other op has a fixed signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Pow,
    Atan2,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Dot,
    Cross,
    /// `vec3 * scalar`, componentwise.
    Scale,
}

impl BinaryOp {
    /// `None` for the polymorphic `Add`/`Sub`; `Some((lhs, rhs, result))` otherwise.
    fn fixed_signature(self) -> Option<(ValueType, ValueType, ValueType)> {
        use BinaryOp::*;
        use ValueType::*;
        match self {
            Add | Sub => None,
            Mul | Div | Min | Max | Pow | Atan2 => Some((Scalar, Scalar, Scalar)),
            Lt | Le | Gt | Ge | Eq => Some((Scalar, Scalar, Bool)),
            Dot => Some((Vec3, Vec3, Scalar)),
            Cross => Some((Vec3, Vec3, Vec3)),
            Scale => Some((Vec3, Scalar, Vec3)),
        }
    }
}

/// One instruction. Serialized as `{"instr": "...", ...}` JSON, mirroring
/// [`super::SdfNode`]'s tagged representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "instr",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Instr {
    PushConst {
        value: Const,
    },
    /// Pushes the `Vec3` position the program is being evaluated at.
    PushPos,
    LoadLocal {
        slot: u16,
    },
    StoreLocal {
        slot: u16,
    },
    /// Discards the top of the stack (any type).
    Pop,
    Unary {
        op: UnaryOp,
    },
    Binary {
        op: BinaryOp,
    },
    /// Pops `(x, lo, hi)`, pushes `x` clamped to `[lo, hi]`.
    Clamp,
    /// Pops `(a, b, cond)`, pushes `a` if `cond` else `b`. `a`/`b` must share
    /// a type (`Scalar` or `Vec3`).
    Select,
    /// Pops `(x, y, z)`, pushes `Vec3([x, y, z])`.
    MakeVec3,
    /// Runs `body` up to `count` times, stopping early if a `BreakIf` inside
    /// it fires. Statically bounded: `count` is fixed at construction time,
    /// never a runtime value.
    Repeat {
        count: u32,
        body: Vec<Instr>,
    },
    /// Pops a `Bool`; if true, stops the nearest enclosing `Repeat` after
    /// this iteration. Invalid outside a `Repeat` body.
    BreakIf,
}

/// What kind of distance a program's output represents — mirrors the
/// authoring intent behind [`super::SdfNode`] primitives (most are `Exact`;
/// `Ellipsoid` is a bound; smooth/noise ops are estimates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DistanceKind {
    /// The exact Euclidean signed distance to the surface.
    Exact,
    /// A conservative lower bound (safe for sphere tracing / inside tests,
    /// never overestimates).
    LowerBound,
    /// Neither guaranteed exact nor a bound (e.g. a fractal DE).
    Estimate,
    /// An implicit function whose sign is meaningful but whose magnitude is
    /// not a distance at all.
    Implicit,
}

impl Default for DistanceKind {
    fn default() -> Self {
        DistanceKind::Estimate
    }
}

/// Explicit, finite axis-aligned bounds a program author asserts for their
/// field — never inferred, since an arbitrary program has no closed-form AABB.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// The plain, fully public data shape behind [`Program`]. Constructing a
/// [`Program`] from this always runs [`validate`] — this type by itself
/// carries no safety guarantee, [`Program`] does.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramData {
    pub slots: Vec<ValueType>,
    pub instructions: Vec<Instr>,
    pub output_slot: u16,
    pub bounds: ProgramBounds,
    #[serde(default)]
    pub distance_kind: DistanceKind,
}

/// Every way [`validate`] can reject a [`ProgramData`], and every way
/// [`ProgramBuilder`] can be misused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramError {
    TooManySlots,
    UnknownSlot(u16),
    InvalidOutputSlot,
    NonFiniteConstant,
    InvalidBounds,
    StackUnderflow,
    TypeMismatch,
    StackNotEmptyAtEnd,
    BreakOutsideRepeat,
    NoOpenRepeat,
    UnclosedRepeat,
    RepeatCountOutOfRange,
    DepthExceeded,
    TooManyInstructions,
    DynamicStepBudgetExceeded,
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::TooManySlots => write!(f, "program declares too many local slots"),
            ProgramError::UnknownSlot(slot) => write!(f, "reference to undeclared slot {slot}"),
            ProgramError::InvalidOutputSlot => {
                write!(f, "output slot must be a declared Scalar slot")
            }
            ProgramError::NonFiniteConstant => write!(f, "constant is not finite"),
            ProgramError::InvalidBounds => write!(f, "bounds must be finite with min <= max"),
            ProgramError::StackUnderflow => {
                write!(f, "instruction pops more values than available")
            }
            ProgramError::TypeMismatch => write!(f, "instruction operand type mismatch"),
            ProgramError::StackNotEmptyAtEnd => {
                write!(
                    f,
                    "block left values on the stack instead of consuming them"
                )
            }
            ProgramError::BreakOutsideRepeat => write!(f, "breakIf used outside a repeat block"),
            ProgramError::NoOpenRepeat => write!(f, "endRepeat with no matching beginRepeat"),
            ProgramError::UnclosedRepeat => write!(f, "program built with an unclosed repeat"),
            ProgramError::RepeatCountOutOfRange => {
                write!(f, "repeat count is zero or exceeds the per-node limit")
            }
            ProgramError::DepthExceeded => write!(f, "repeat nesting exceeds the depth limit"),
            ProgramError::TooManyInstructions => write!(f, "program exceeds the instruction limit"),
            ProgramError::DynamicStepBudgetExceeded => write!(
                f,
                "worst-case dynamic instruction count exceeds the iteration budget"
            ),
        }
    }
}

impl std::error::Error for ProgramError {}

impl From<ProgramError> for String {
    fn from(e: ProgramError) -> Self {
        e.to_string()
    }
}

fn pop_expect(stack: &mut Vec<ValueType>, expected: ValueType) -> Result<(), ProgramError> {
    match stack.pop() {
        Some(ty) if ty == expected => Ok(()),
        Some(_) => Err(ProgramError::TypeMismatch),
        None => Err(ProgramError::StackUnderflow),
    }
}

fn pop_any(stack: &mut Vec<ValueType>) -> Result<ValueType, ProgramError> {
    stack.pop().ok_or(ProgramError::StackUnderflow)
}

struct ValidateCtx<'a> {
    slots: &'a [ValueType],
    instr_count: usize,
    dynamic_steps: u64,
}

impl<'a> ValidateCtx<'a> {
    fn slot_type(&self, slot: u16) -> Result<ValueType, ProgramError> {
        self.slots
            .get(slot as usize)
            .copied()
            .ok_or(ProgramError::UnknownSlot(slot))
    }

    fn validate_block(
        &mut self,
        instrs: &[Instr],
        depth: usize,
        multiplier: u64,
    ) -> Result<(), ProgramError> {
        let mut stack: Vec<ValueType> = Vec::new();
        for instr in instrs {
            self.instr_count += 1;
            if self.instr_count > MAX_STATIC_INSTRUCTIONS {
                return Err(ProgramError::TooManyInstructions);
            }
            self.dynamic_steps = self.dynamic_steps.saturating_add(multiplier);
            if self.dynamic_steps > MAX_DYNAMIC_STEPS {
                return Err(ProgramError::DynamicStepBudgetExceeded);
            }

            match instr {
                Instr::PushConst { value } => {
                    if !value.is_finite() {
                        return Err(ProgramError::NonFiniteConstant);
                    }
                    stack.push(value.value_type());
                }
                Instr::PushPos => stack.push(ValueType::Vec3),
                Instr::LoadLocal { slot } => {
                    let ty = self.slot_type(*slot)?;
                    stack.push(ty);
                }
                Instr::StoreLocal { slot } => {
                    let ty = self.slot_type(*slot)?;
                    pop_expect(&mut stack, ty)?;
                }
                Instr::Pop => {
                    pop_any(&mut stack)?;
                }
                Instr::Unary { op } => {
                    let (in_ty, out_ty) = op.signature();
                    pop_expect(&mut stack, in_ty)?;
                    stack.push(out_ty);
                }
                Instr::Binary { op } => match op.fixed_signature() {
                    Some((lhs, rhs, result)) => {
                        pop_expect(&mut stack, rhs)?;
                        pop_expect(&mut stack, lhs)?;
                        stack.push(result);
                    }
                    None => {
                        let b_ty = pop_any(&mut stack)?;
                        let a_ty = pop_any(&mut stack)?;
                        if a_ty != b_ty || a_ty == ValueType::Bool {
                            return Err(ProgramError::TypeMismatch);
                        }
                        stack.push(a_ty);
                    }
                },
                Instr::Clamp => {
                    pop_expect(&mut stack, ValueType::Scalar)?; // hi
                    pop_expect(&mut stack, ValueType::Scalar)?; // lo
                    pop_expect(&mut stack, ValueType::Scalar)?; // x
                    stack.push(ValueType::Scalar);
                }
                Instr::Select => {
                    pop_expect(&mut stack, ValueType::Bool)?; // cond
                    let b_ty = pop_any(&mut stack)?;
                    let a_ty = pop_any(&mut stack)?;
                    if a_ty != b_ty || a_ty == ValueType::Bool {
                        return Err(ProgramError::TypeMismatch);
                    }
                    stack.push(a_ty);
                }
                Instr::MakeVec3 => {
                    pop_expect(&mut stack, ValueType::Scalar)?; // z
                    pop_expect(&mut stack, ValueType::Scalar)?; // y
                    pop_expect(&mut stack, ValueType::Scalar)?; // x
                    stack.push(ValueType::Vec3);
                }
                Instr::BreakIf => {
                    if depth == 0 {
                        return Err(ProgramError::BreakOutsideRepeat);
                    }
                    pop_expect(&mut stack, ValueType::Bool)?;
                }
                Instr::Repeat { count, body } => {
                    if *count == 0 || *count > MAX_REPEAT_ITERATIONS {
                        return Err(ProgramError::RepeatCountOutOfRange);
                    }
                    if depth + 1 > MAX_REPEAT_DEPTH {
                        return Err(ProgramError::DepthExceeded);
                    }
                    let next_multiplier = multiplier.saturating_mul(u64::from(*count));
                    self.validate_block(body, depth + 1, next_multiplier)?;
                }
            }
        }
        if !stack.is_empty() {
            return Err(ProgramError::StackNotEmptyAtEnd);
        }
        Ok(())
    }
}

/// Validate a [`ProgramData`]: every instruction's operand types and stack
/// effect, slot references, the output slot, constant finiteness, bounds,
/// and the instruction/depth/iteration budgets. `Ok(())` means the program
/// is safe to evaluate an unbounded number of times without panicking,
/// looping forever, or exceeding [`MAX_DYNAMIC_STEPS`] worst-case steps.
pub fn validate(data: &ProgramData) -> Result<(), ProgramError> {
    if data.slots.len() > MAX_SLOTS {
        return Err(ProgramError::TooManySlots);
    }
    if data.slots.get(data.output_slot as usize) != Some(&ValueType::Scalar) {
        return Err(ProgramError::InvalidOutputSlot);
    }
    if !data.bounds.min.iter().all(|v| v.is_finite())
        || !data.bounds.max.iter().all(|v| v.is_finite())
    {
        return Err(ProgramError::InvalidBounds);
    }
    for axis in 0..3 {
        if data.bounds.min[axis] > data.bounds.max[axis] {
            return Err(ProgramError::InvalidBounds);
        }
    }

    let mut ctx = ValidateCtx {
        slots: &data.slots,
        instr_count: 0,
        dynamic_steps: 0,
    };
    ctx.validate_block(&data.instructions, 0, 1)
}

// ── Evaluation ───────────────────────────────────────────────────────────

/// A numeric field type the generic interpreter can run over: plain `f32`
/// for evaluation, or [`Dual`] for forward-mode gradient propagation. Every
/// operation is total (never panics); non-differentiable points propagate
/// `NaN` through `Dual`'s derivative component exactly like IEEE 754
/// propagates `NaN` through `f32` values, so [`Program::analytic_gradient`]
/// can detect them uniformly by checking the final result is finite.
trait Field: Copy {
    fn constant(v: f32) -> Self;
    fn primal(self) -> f32;
    fn add(self, o: Self) -> Self;
    fn sub(self, o: Self) -> Self;
    fn mul(self, o: Self) -> Self;
    fn div(self, o: Self) -> Self;
    fn neg(self) -> Self;
    fn abs(self) -> Self;
    fn sqrt(self) -> Self;
    fn min(self, o: Self) -> Self;
    fn max(self, o: Self) -> Self;
    fn pow(self, o: Self) -> Self;
    fn ln(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn acos(self) -> Self;
    fn atan2(self, o: Self) -> Self;
}

impl Field for f32 {
    fn constant(v: f32) -> Self {
        v
    }
    fn primal(self) -> f32 {
        self
    }
    fn add(self, o: Self) -> Self {
        self + o
    }
    fn sub(self, o: Self) -> Self {
        self - o
    }
    fn mul(self, o: Self) -> Self {
        self * o
    }
    fn div(self, o: Self) -> Self {
        self / o
    }
    fn neg(self) -> Self {
        -self
    }
    fn abs(self) -> Self {
        f32::abs(self)
    }
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
    fn min(self, o: Self) -> Self {
        f32::min(self, o)
    }
    fn max(self, o: Self) -> Self {
        f32::max(self, o)
    }
    fn pow(self, o: Self) -> Self {
        f32::powf(self, o)
    }
    fn ln(self) -> Self {
        f32::ln(self)
    }
    fn sin(self) -> Self {
        f32::sin(self)
    }
    fn cos(self) -> Self {
        f32::cos(self)
    }
    fn acos(self) -> Self {
        f32::acos(self)
    }
    fn atan2(self, o: Self) -> Self {
        f32::atan2(self, o)
    }
}

/// `v` plus its partial derivatives with respect to the program's `(x, y,
/// z)` position input.
#[derive(Debug, Clone, Copy)]
struct Dual {
    v: f32,
    d: [f32; 3],
}

impl Dual {
    fn seed(v: f32, axis: usize) -> Self {
        let mut d = [0.0_f32; 3];
        d[axis] = 1.0;
        Dual { v, d }
    }
}

impl Field for Dual {
    fn constant(v: f32) -> Self {
        Dual {
            v,
            d: [0.0, 0.0, 0.0],
        }
    }
    fn primal(self) -> f32 {
        self.v
    }
    fn add(self, o: Self) -> Self {
        Dual {
            v: self.v + o.v,
            d: [self.d[0] + o.d[0], self.d[1] + o.d[1], self.d[2] + o.d[2]],
        }
    }
    fn sub(self, o: Self) -> Self {
        Dual {
            v: self.v - o.v,
            d: [self.d[0] - o.d[0], self.d[1] - o.d[1], self.d[2] - o.d[2]],
        }
    }
    fn mul(self, o: Self) -> Self {
        Dual {
            v: self.v * o.v,
            d: [
                self.d[0] * o.v + self.v * o.d[0],
                self.d[1] * o.v + self.v * o.d[1],
                self.d[2] * o.v + self.v * o.d[2],
            ],
        }
    }
    fn div(self, o: Self) -> Self {
        let inv = 1.0 / o.v;
        let v = self.v * inv;
        Dual {
            v,
            d: [
                (self.d[0] * o.v - self.v * o.d[0]) * inv * inv,
                (self.d[1] * o.v - self.v * o.d[1]) * inv * inv,
                (self.d[2] * o.v - self.v * o.d[2]) * inv * inv,
            ],
        }
    }
    fn neg(self) -> Self {
        Dual {
            v: -self.v,
            d: [-self.d[0], -self.d[1], -self.d[2]],
        }
    }
    fn abs(self) -> Self {
        let sign = if self.v > 0.0 {
            1.0
        } else if self.v < 0.0 {
            -1.0
        } else {
            f32::NAN // |x| is non-differentiable at x == 0
        };
        Dual {
            v: self.v.abs(),
            d: [self.d[0] * sign, self.d[1] * sign, self.d[2] * sign],
        }
    }
    fn sqrt(self) -> Self {
        let v = self.v.sqrt();
        let denom = 2.0 * v;
        Dual {
            v,
            d: [self.d[0] / denom, self.d[1] / denom, self.d[2] / denom],
        }
    }
    fn min(self, o: Self) -> Self {
        if self.v <= o.v {
            self
        } else {
            o
        }
    }
    fn max(self, o: Self) -> Self {
        if self.v >= o.v {
            self
        } else {
            o
        }
    }
    fn pow(self, o: Self) -> Self {
        let v = self.v.powf(o.v);
        // Common case: a constant exponent (o.d == 0) must not pick up a
        // spurious ln(self.v) term when self.v <= 0 — 0 * NaN is NaN, not 0.
        let d = if o.d == [0.0, 0.0, 0.0] {
            let coeff = o.v * self.v.powf(o.v - 1.0);
            [coeff * self.d[0], coeff * self.d[1], coeff * self.d[2]]
        } else {
            let coeff = o.v * self.v.powf(o.v - 1.0);
            let ln_term = self.v.ln();
            [
                coeff * self.d[0] + v * ln_term * o.d[0],
                coeff * self.d[1] + v * ln_term * o.d[1],
                coeff * self.d[2] + v * ln_term * o.d[2],
            ]
        };
        Dual { v, d }
    }
    fn ln(self) -> Self {
        Dual {
            v: self.v.ln(),
            d: [self.d[0] / self.v, self.d[1] / self.v, self.d[2] / self.v],
        }
    }
    fn sin(self) -> Self {
        let c = self.v.cos();
        Dual {
            v: self.v.sin(),
            d: [self.d[0] * c, self.d[1] * c, self.d[2] * c],
        }
    }
    fn cos(self) -> Self {
        let s = -self.v.sin();
        Dual {
            v: self.v.cos(),
            d: [self.d[0] * s, self.d[1] * s, self.d[2] * s],
        }
    }
    fn acos(self) -> Self {
        let denom = (1.0 - self.v * self.v).sqrt();
        Dual {
            v: self.v.acos(),
            d: [-self.d[0] / denom, -self.d[1] / denom, -self.d[2] / denom],
        }
    }
    fn atan2(self, o: Self) -> Self {
        // self = y, o = x
        let denom = o.v * o.v + self.v * self.v;
        Dual {
            v: self.v.atan2(o.v),
            d: [
                (o.v * self.d[0] - self.v * o.d[0]) / denom,
                (o.v * self.d[1] - self.v * o.d[1]) / denom,
                (o.v * self.d[2] - self.v * o.d[2]) / denom,
            ],
        }
    }
}

#[derive(Clone, Copy)]
enum RtValue<S> {
    Scalar(S),
    Vec3([S; 3]),
    Bool(bool),
}

fn rt_scalar<S: Field>(v: RtValue<S>) -> S {
    match v {
        RtValue::Scalar(s) => s,
        _ => S::constant(0.0),
    }
}

fn rt_vec3<S: Field>(v: RtValue<S>) -> [S; 3] {
    match v {
        RtValue::Vec3(a) => a,
        _ => [S::constant(0.0), S::constant(0.0), S::constant(0.0)],
    }
}

fn default_slot_value<S: Field>(ty: ValueType) -> RtValue<S> {
    match ty {
        ValueType::Scalar => RtValue::Scalar(S::constant(0.0)),
        ValueType::Vec3 => RtValue::Vec3([S::constant(0.0), S::constant(0.0), S::constant(0.0)]),
        ValueType::Bool => RtValue::Bool(false),
    }
}

fn const_to_rt<S: Field>(c: Const) -> RtValue<S> {
    match c {
        Const::Scalar(v) => RtValue::Scalar(S::constant(v)),
        Const::Vec3(v) => RtValue::Vec3([S::constant(v[0]), S::constant(v[1]), S::constant(v[2])]),
        Const::Bool(v) => RtValue::Bool(v),
    }
}

fn eval_unary<S: Field>(op: UnaryOp, v: RtValue<S>) -> RtValue<S> {
    match op {
        UnaryOp::Neg => RtValue::Scalar(rt_scalar(v).neg()),
        UnaryOp::Abs => RtValue::Scalar(rt_scalar(v).abs()),
        UnaryOp::Sqrt => RtValue::Scalar(rt_scalar(v).sqrt()),
        UnaryOp::Log => RtValue::Scalar(rt_scalar(v).ln()),
        UnaryOp::Sin => RtValue::Scalar(rt_scalar(v).sin()),
        UnaryOp::Cos => RtValue::Scalar(rt_scalar(v).cos()),
        UnaryOp::Acos => RtValue::Scalar(rt_scalar(v).acos()),
        UnaryOp::VecX => RtValue::Scalar(rt_vec3(v)[0]),
        UnaryOp::VecY => RtValue::Scalar(rt_vec3(v)[1]),
        UnaryOp::VecZ => RtValue::Scalar(rt_vec3(v)[2]),
        UnaryOp::Length => {
            let a = rt_vec3(v);
            RtValue::Scalar(
                a[0].mul(a[0])
                    .add(a[1].mul(a[1]))
                    .add(a[2].mul(a[2]))
                    .sqrt(),
            )
        }
        UnaryOp::Normalize => {
            let a = rt_vec3(v);
            let len = a[0]
                .mul(a[0])
                .add(a[1].mul(a[1]))
                .add(a[2].mul(a[2]))
                .sqrt();
            if len.primal() == 0.0 {
                RtValue::Vec3([S::constant(0.0), S::constant(0.0), S::constant(0.0)])
            } else {
                RtValue::Vec3([a[0].div(len), a[1].div(len), a[2].div(len)])
            }
        }
    }
}

fn eval_binary<S: Field>(op: BinaryOp, a: RtValue<S>, b: RtValue<S>) -> RtValue<S> {
    match op {
        BinaryOp::Add => match (a, b) {
            (RtValue::Vec3(av), RtValue::Vec3(bv)) => {
                RtValue::Vec3([av[0].add(bv[0]), av[1].add(bv[1]), av[2].add(bv[2])])
            }
            _ => RtValue::Scalar(rt_scalar(a).add(rt_scalar(b))),
        },
        BinaryOp::Sub => match (a, b) {
            (RtValue::Vec3(av), RtValue::Vec3(bv)) => {
                RtValue::Vec3([av[0].sub(bv[0]), av[1].sub(bv[1]), av[2].sub(bv[2])])
            }
            _ => RtValue::Scalar(rt_scalar(a).sub(rt_scalar(b))),
        },
        BinaryOp::Mul => RtValue::Scalar(rt_scalar(a).mul(rt_scalar(b))),
        BinaryOp::Div => RtValue::Scalar(rt_scalar(a).div(rt_scalar(b))),
        BinaryOp::Min => RtValue::Scalar(rt_scalar(a).min(rt_scalar(b))),
        BinaryOp::Max => RtValue::Scalar(rt_scalar(a).max(rt_scalar(b))),
        BinaryOp::Pow => RtValue::Scalar(rt_scalar(a).pow(rt_scalar(b))),
        BinaryOp::Atan2 => RtValue::Scalar(rt_scalar(a).atan2(rt_scalar(b))),
        BinaryOp::Lt => RtValue::Bool(rt_scalar(a).primal() < rt_scalar(b).primal()),
        BinaryOp::Le => RtValue::Bool(rt_scalar(a).primal() <= rt_scalar(b).primal()),
        BinaryOp::Gt => RtValue::Bool(rt_scalar(a).primal() > rt_scalar(b).primal()),
        BinaryOp::Ge => RtValue::Bool(rt_scalar(a).primal() >= rt_scalar(b).primal()),
        BinaryOp::Eq => RtValue::Bool(rt_scalar(a).primal() == rt_scalar(b).primal()),
        BinaryOp::Dot => {
            let (av, bv) = (rt_vec3(a), rt_vec3(b));
            RtValue::Scalar(av[0].mul(bv[0]).add(av[1].mul(bv[1])).add(av[2].mul(bv[2])))
        }
        BinaryOp::Cross => {
            let (av, bv) = (rt_vec3(a), rt_vec3(b));
            RtValue::Vec3([
                av[1].mul(bv[2]).sub(av[2].mul(bv[1])),
                av[2].mul(bv[0]).sub(av[0].mul(bv[2])),
                av[0].mul(bv[1]).sub(av[1].mul(bv[0])),
            ])
        }
        BinaryOp::Scale => {
            let (av, s) = (rt_vec3(a), rt_scalar(b));
            RtValue::Vec3([av[0].mul(s), av[1].mul(s), av[2].mul(s)])
        }
    }
}

fn eval_clamp<S: Field>(x: RtValue<S>, lo: RtValue<S>, hi: RtValue<S>) -> RtValue<S> {
    let (x, lo, hi) = (rt_scalar(x), rt_scalar(lo), rt_scalar(hi));
    if x.primal() < lo.primal() {
        RtValue::Scalar(lo)
    } else if x.primal() > hi.primal() {
        RtValue::Scalar(hi)
    } else {
        RtValue::Scalar(x)
    }
}

enum Flow {
    Continue,
    Break,
}

/// Run `instrs` against `stack`/`slots`. Every pop is defensive (falls back
/// to a zero value rather than panicking) as a second line of defense beyond
/// [`validate`] having already proven the stack effects balance.
fn eval_block<S: Field>(
    instrs: &[Instr],
    stack: &mut Vec<RtValue<S>>,
    slots: &mut [RtValue<S>],
    pos: [S; 3],
) -> Flow {
    for instr in instrs {
        match instr {
            Instr::PushConst { value } => stack.push(const_to_rt(*value)),
            Instr::PushPos => stack.push(RtValue::Vec3(pos)),
            Instr::LoadLocal { slot } => {
                if let Some(v) = slots.get(*slot as usize) {
                    stack.push(*v);
                }
            }
            Instr::StoreLocal { slot } => {
                if let Some(v) = stack.pop() {
                    if let Some(dst) = slots.get_mut(*slot as usize) {
                        *dst = v;
                    }
                }
            }
            Instr::Pop => {
                stack.pop();
            }
            Instr::Unary { op } => {
                if let Some(v) = stack.pop() {
                    stack.push(eval_unary(*op, v));
                }
            }
            Instr::Binary { op } => {
                if let (Some(b), Some(a)) = (stack.pop(), stack.pop()) {
                    stack.push(eval_binary(*op, a, b));
                }
            }
            Instr::Clamp => {
                if let (Some(hi), Some(lo), Some(x)) = (stack.pop(), stack.pop(), stack.pop()) {
                    stack.push(eval_clamp(x, lo, hi));
                }
            }
            Instr::Select => {
                if let (Some(cond), Some(b), Some(a)) = (stack.pop(), stack.pop(), stack.pop()) {
                    if let RtValue::Bool(c) = cond {
                        stack.push(if c { a } else { b });
                    }
                }
            }
            Instr::MakeVec3 => {
                if let (Some(z), Some(y), Some(x)) = (stack.pop(), stack.pop(), stack.pop()) {
                    stack.push(RtValue::Vec3([rt_scalar(x), rt_scalar(y), rt_scalar(z)]));
                }
            }
            Instr::BreakIf => {
                if let Some(RtValue::Bool(c)) = stack.pop() {
                    if c {
                        return Flow::Break;
                    }
                }
            }
            Instr::Repeat { count, body } => {
                for _ in 0..*count {
                    if let Flow::Break = eval_block(body, stack, slots, pos) {
                        break;
                    }
                }
            }
        }
    }
    Flow::Continue
}

/// A validated, sandboxed SDF field program. The only ways to obtain one are
/// [`Program::from_json`] and [`ProgramBuilder::build`] — both run
/// [`validate`], so every `Program` is guaranteed well-typed, stack-balanced,
/// and bounded in instructions/depth/iterations before it can ever be
/// evaluated.
#[derive(Debug, Clone)]
pub struct Program(ProgramData);

impl Program {
    /// Validate `data` and wrap it. The only safe way to construct a
    /// [`Program`] from a hand-built [`ProgramData`] (prefer [`ProgramBuilder`]
    /// or [`Program::from_json`] otherwise).
    pub fn compile(data: ProgramData) -> Result<Program, ProgramError> {
        validate(&data)?;
        Ok(Program(data))
    }

    /// The validated program data.
    pub fn data(&self) -> &ProgramData {
        &self.0
    }

    /// Parse and validate a program from JSON. Never panics on malformed
    /// input — parse and validation failures both surface as `Err`.
    pub fn from_json(json: &str) -> Result<Program, ProgramError> {
        let data: ProgramData =
            serde_json::from_str(json).map_err(|_| ProgramError::TypeMismatch)?;
        Program::compile(data)
    }

    /// Serialize back to JSON.
    pub fn to_json(&self) -> Result<String, ProgramError> {
        serde_json::to_string(&self.0).map_err(|_| ProgramError::TypeMismatch)
    }

    /// Evaluate the program's scalar output at `(x, y, z)`. Deterministic
    /// pure `f32` arithmetic — degenerate operations (e.g. division by zero)
    /// yield `NaN`/`inf` per IEEE 754 rather than panicking.
    pub fn eval(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut slots: Vec<RtValue<f32>> = self
            .0
            .slots
            .iter()
            .map(|ty| default_slot_value(*ty))
            .collect();
        let mut stack: Vec<RtValue<f32>> = Vec::new();
        eval_block(&self.0.instructions, &mut stack, &mut slots, [x, y, z]);
        match slots.get(self.0.output_slot as usize) {
            Some(RtValue::Scalar(v)) => *v,
            _ => f32::NAN,
        }
    }

    /// Forward-mode dual-number analytic gradient of the scalar output at
    /// `(x, y, z)`, normalized to a unit vector. `None` when the program
    /// isn't differentiable at this exact point (e.g. `length` of the zero
    /// vector, `sqrt`/`log`/`acos` outside their domain) — callers should
    /// fall back to a numerical estimate (see [`super::numerical_normal`])
    /// in that case.
    pub fn analytic_gradient(&self, x: f32, y: f32, z: f32) -> Option<[f32; 3]> {
        let mut slots: Vec<RtValue<Dual>> = self
            .0
            .slots
            .iter()
            .map(|ty| default_slot_value(*ty))
            .collect();
        let mut stack: Vec<RtValue<Dual>> = Vec::new();
        let pos = [Dual::seed(x, 0), Dual::seed(y, 1), Dual::seed(z, 2)];
        eval_block(&self.0.instructions, &mut stack, &mut slots, pos);
        let d = match slots.get(self.0.output_slot as usize) {
            Some(RtValue::Scalar(d)) => *d,
            _ => return None,
        };
        if !d.v.is_finite() || !d.d.iter().all(|c| c.is_finite()) {
            return None;
        }
        let len = (d.d[0] * d.d[0] + d.d[1] * d.d[1] + d.d[2] * d.d[2]).sqrt();
        if !(len > f32::EPSILON) {
            return None;
        }
        Some([d.d[0] / len, d.d[1] / len, d.d[2] / len])
    }

    /// The program's explicit, author-asserted finite bounds.
    pub fn aabb(&self) -> Aabb {
        Aabb {
            min: self.0.bounds.min,
            max: self.0.bounds.max,
        }
    }

    /// The kind of distance the program's output represents.
    pub fn distance_kind(&self) -> DistanceKind {
        self.0.distance_kind
    }
}

impl Serialize for Program {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Program {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = ProgramData::deserialize(deserializer)?;
        Program::compile(data).map_err(serde::de::Error::custom)
    }
}

/// Records instructions into nested blocks (for `Repeat` bodies) and
/// produces a validated [`Program`] on [`ProgramBuilder::build`]. Recording
/// itself never fails — malformed programs (bad types, dangling stack
/// values, unknown slots, out-of-budget loops) are only rejected once, at
/// `build()`, by the same [`validate`] that guards JSON import.
#[derive(Debug)]
pub struct ProgramBuilder {
    slots: Vec<ValueType>,
    blocks: Vec<Vec<Instr>>,
    repeat_counts: Vec<u32>,
    output_slot: Option<u16>,
    bounds: Option<ProgramBounds>,
    distance_kind: DistanceKind,
}

impl Default for ProgramBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramBuilder {
    pub fn new() -> Self {
        ProgramBuilder {
            slots: Vec::new(),
            blocks: vec![Vec::new()],
            repeat_counts: Vec::new(),
            output_slot: None,
            bounds: None,
            distance_kind: DistanceKind::default(),
        }
    }

    /// Declare a new typed local slot, initialized to a type-appropriate
    /// zero value, and return its index.
    pub fn add_slot(&mut self, ty: ValueType) -> u16 {
        let idx = self.slots.len() as u16;
        self.slots.push(ty);
        idx
    }

    fn current(&mut self) -> &mut Vec<Instr> {
        self.blocks.last_mut().expect("root block always present")
    }

    pub fn push_const(&mut self, value: Const) -> &mut Self {
        self.current().push(Instr::PushConst { value });
        self
    }

    pub fn push_pos(&mut self) -> &mut Self {
        self.current().push(Instr::PushPos);
        self
    }

    pub fn load_local(&mut self, slot: u16) -> &mut Self {
        self.current().push(Instr::LoadLocal { slot });
        self
    }

    pub fn store_local(&mut self, slot: u16) -> &mut Self {
        self.current().push(Instr::StoreLocal { slot });
        self
    }

    pub fn pop(&mut self) -> &mut Self {
        self.current().push(Instr::Pop);
        self
    }

    pub fn unary(&mut self, op: UnaryOp) -> &mut Self {
        self.current().push(Instr::Unary { op });
        self
    }

    pub fn binary(&mut self, op: BinaryOp) -> &mut Self {
        self.current().push(Instr::Binary { op });
        self
    }

    pub fn clamp(&mut self) -> &mut Self {
        self.current().push(Instr::Clamp);
        self
    }

    pub fn select(&mut self) -> &mut Self {
        self.current().push(Instr::Select);
        self
    }

    pub fn make_vec3(&mut self) -> &mut Self {
        self.current().push(Instr::MakeVec3);
        self
    }

    pub fn break_if(&mut self) -> &mut Self {
        self.current().push(Instr::BreakIf);
        self
    }

    /// Open a new `Repeat` block; subsequent instructions append to its body
    /// until [`ProgramBuilder::end_repeat`].
    pub fn begin_repeat(&mut self, count: u32) -> &mut Self {
        self.blocks.push(Vec::new());
        self.repeat_counts.push(count);
        self
    }

    /// Close the innermost open `Repeat` block, appending it to its parent.
    pub fn end_repeat(&mut self) -> Result<&mut Self, ProgramError> {
        if self.blocks.len() <= 1 {
            return Err(ProgramError::NoOpenRepeat);
        }
        let body = self.blocks.pop().expect("checked len > 1 above");
        let count = self.repeat_counts.pop().expect("parallel to blocks");
        self.current().push(Instr::Repeat { count, body });
        Ok(self)
    }

    pub fn set_output(&mut self, slot: u16) -> &mut Self {
        self.output_slot = Some(slot);
        self
    }

    pub fn set_bounds(&mut self, min: [f32; 3], max: [f32; 3]) -> &mut Self {
        self.bounds = Some(ProgramBounds { min, max });
        self
    }

    pub fn set_distance_kind(&mut self, kind: DistanceKind) -> &mut Self {
        self.distance_kind = kind;
        self
    }

    /// Finalize and validate the program.
    pub fn build(mut self) -> Result<Program, ProgramError> {
        if self.blocks.len() != 1 {
            return Err(ProgramError::UnclosedRepeat);
        }
        let instructions = self.blocks.pop().expect("root block always present");
        let output_slot = self.output_slot.ok_or(ProgramError::InvalidOutputSlot)?;
        let bounds = self.bounds.ok_or(ProgramError::InvalidBounds)?;
        let data = ProgramData {
            slots: self.slots,
            instructions,
            output_slot,
            bounds,
            distance_kind: self.distance_kind,
        };
        Program::compile(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_program(radius: f32) -> Program {
        let mut b = ProgramBuilder::new();
        let dist = b.add_slot(ValueType::Scalar);
        b.set_output(dist);
        b.set_bounds([-radius, -radius, -radius], [radius, radius, radius]);
        b.set_distance_kind(DistanceKind::Exact);

        b.push_pos();
        b.unary(UnaryOp::Length);
        b.push_const(Const::Scalar(radius));
        b.binary(BinaryOp::Sub);
        b.store_local(dist);

        b.build().expect("valid sphere program")
    }

    #[test]
    fn sphere_program_matches_sphere_sdf() {
        let program = sphere_program(2.0);
        assert!((program.eval(2.0, 0.0, 0.0) - 0.0).abs() < 1e-5);
        assert!(program.eval(0.0, 0.0, 0.0) < 0.0);
        assert!(program.eval(10.0, 0.0, 0.0) > 0.0);
        let expected = (3.0_f32 * 3.0 * 3.0).sqrt() - 2.0;
        assert!((program.eval(3.0, 3.0, 3.0) - expected).abs() < 1e-4);
    }

    #[test]
    fn validation_rejects_output_slot_wrong_type() {
        let mut b = ProgramBuilder::new();
        let flag = b.add_slot(ValueType::Bool);
        b.set_output(flag);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Bool(true));
        b.store_local(flag);
        assert_eq!(b.build().unwrap_err(), ProgramError::InvalidOutputSlot);
    }

    #[test]
    fn validation_rejects_unbalanced_stack() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Scalar(1.0));
        b.push_const(Const::Scalar(2.0)); // left dangling on the stack
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::StackNotEmptyAtEnd);
    }

    #[test]
    fn validation_rejects_stack_underflow() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Scalar(1.0));
        b.binary(BinaryOp::Add); // only one operand available
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::StackUnderflow);
    }

    #[test]
    fn validation_rejects_type_mismatch() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Scalar(1.0));
        b.push_const(Const::Bool(true));
        b.binary(BinaryOp::Add);
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::TypeMismatch);
    }

    #[test]
    fn validation_rejects_non_finite_constant() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Scalar(f32::NAN));
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::NonFiniteConstant);
    }

    #[test]
    fn validation_rejects_break_outside_repeat() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.push_const(Const::Bool(true));
        b.break_if();
        assert_eq!(b.build().unwrap_err(), ProgramError::BreakOutsideRepeat);
    }

    #[test]
    fn validation_rejects_invalid_bounds() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([1.0, 1.0, 1.0], [-1.0, -1.0, -1.0]);
        b.push_const(Const::Scalar(0.0));
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::InvalidBounds);
    }

    #[test]
    fn validation_rejects_repeat_count_out_of_range() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.begin_repeat(MAX_REPEAT_ITERATIONS + 1);
        b.end_repeat().unwrap();
        b.push_const(Const::Scalar(0.0));
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::RepeatCountOutOfRange);
    }

    #[test]
    fn validation_rejects_dynamic_step_budget_blowup_from_nested_repeats() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.begin_repeat(MAX_REPEAT_ITERATIONS);
        b.begin_repeat(MAX_REPEAT_ITERATIONS);
        b.push_const(Const::Scalar(0.0));
        b.pop();
        b.end_repeat().unwrap();
        b.end_repeat().unwrap();
        b.push_const(Const::Scalar(0.0));
        b.store_local(out);
        assert_eq!(
            b.build().unwrap_err(),
            ProgramError::DynamicStepBudgetExceeded
        );
    }

    #[test]
    fn validation_rejects_unclosed_repeat() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        b.begin_repeat(4);
        b.push_const(Const::Scalar(0.0));
        b.store_local(out);
        assert_eq!(b.build().unwrap_err(), ProgramError::UnclosedRepeat);
    }

    #[test]
    fn end_repeat_without_begin_is_rejected() {
        let mut b = ProgramBuilder::new();
        assert_eq!(b.end_repeat().unwrap_err(), ProgramError::NoOpenRepeat);
    }

    #[test]
    fn bounded_repeat_with_break_if_stops_early() {
        // counter += 1 each iteration, static bound 100, break once counter >= 3.
        let mut b = ProgramBuilder::new();
        let counter = b.add_slot(ValueType::Scalar);
        b.set_output(counter);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        b.begin_repeat(100);
        b.load_local(counter);
        b.push_const(Const::Scalar(1.0));
        b.binary(BinaryOp::Add);
        b.store_local(counter);

        b.load_local(counter);
        b.push_const(Const::Scalar(3.0));
        b.binary(BinaryOp::Ge);
        b.break_if();
        b.end_repeat().unwrap();

        let program = b.build().expect("valid bounded loop");
        assert!((program.eval(0.0, 0.0, 0.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn repeat_runs_full_static_count_without_break() {
        let mut b = ProgramBuilder::new();
        let counter = b.add_slot(ValueType::Scalar);
        b.set_output(counter);
        b.set_bounds([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        b.begin_repeat(7);
        b.load_local(counter);
        b.push_const(Const::Scalar(1.0));
        b.binary(BinaryOp::Add);
        b.store_local(counter);
        b.end_repeat().unwrap();

        let program = b.build().unwrap();
        assert!((program.eval(0.0, 0.0, 0.0) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn json_roundtrip_preserves_evaluation() {
        let program = sphere_program(3.5);
        let json = program.to_json().expect("serialize");
        let restored = Program::from_json(&json).expect("deserialize + validate");
        for p in [(0.0, 0.0, 0.0), (3.5, 0.0, 0.0), (10.0, -2.0, 4.0)] {
            assert!((program.eval(p.0, p.1, p.2) - restored.eval(p.0, p.1, p.2)).abs() < 1e-6);
        }
    }

    #[test]
    fn from_json_rejects_malformed_json_without_panicking() {
        assert!(Program::from_json("{ not json").is_err());
        assert!(Program::from_json("{}").is_err());
        assert!(Program::from_json("null").is_err());
        assert!(Program::from_json(
            r#"{"slots":[],"instructions":[],"outputSlot":0,"bounds":{"min":[0,0,0],"max":[0,0,0]}}"#
        )
        .is_err());
    }

    /// Classic Mandelbulb distance estimator, expressed entirely as a bounded
    /// program loop (no host callbacks):
    ///
    /// ```text
    /// z = pos; dr = 1.0
    /// repeat MAX_ITER:
    ///   r = length(z)
    ///   break if r > BAILOUT
    ///   theta = acos(z.z / r) * POWER
    ///   phi = atan2(z.y, z.x) * POWER
    ///   dr = pow(r, POWER - 1) * POWER * dr + 1.0
    ///   zr = pow(r, POWER)
    ///   z = zr * vec3(sin(theta) * cos(phi), sin(phi) * sin(theta), cos(theta)) + pos
    /// distance = 0.5 * log(r) * r / dr
    /// ```
    fn mandelbulb_program() -> Program {
        const POWER: f32 = 8.0;
        const BAILOUT: f32 = 4.0;
        const MAX_ITER: u32 = 12;

        let mut b = ProgramBuilder::new();
        let z = b.add_slot(ValueType::Vec3);
        let dr = b.add_slot(ValueType::Scalar);
        let r = b.add_slot(ValueType::Scalar);
        let dist = b.add_slot(ValueType::Scalar);

        b.set_output(dist);
        b.set_bounds([-1.3, -1.3, -1.3], [1.3, 1.3, 1.3]);
        b.set_distance_kind(DistanceKind::Estimate);

        // z = pos
        b.push_pos();
        b.store_local(z);
        // dr = 1.0
        b.push_const(Const::Scalar(1.0));
        b.store_local(dr);

        b.begin_repeat(MAX_ITER);
        {
            // r = length(z)
            b.load_local(z);
            b.unary(UnaryOp::Length);
            b.store_local(r);

            // break if r > BAILOUT
            b.load_local(r);
            b.push_const(Const::Scalar(BAILOUT));
            b.binary(BinaryOp::Gt);
            b.break_if();

            // theta = acos(clamp(z.z / max(r, EPS), -1, 1)) * POWER
            // (r == 0 exactly at the Mandelbulb's own center; guard the
            // division and clamp the acos argument against float rounding
            // so the origin evaluates to a finite value instead of NaN.)
            b.load_local(z);
            b.unary(UnaryOp::VecZ);
            b.load_local(r);
            b.push_const(Const::Scalar(1e-6));
            b.binary(BinaryOp::Max);
            b.binary(BinaryOp::Div);
            b.push_const(Const::Scalar(-1.0));
            b.push_const(Const::Scalar(1.0));
            b.clamp();
            b.unary(UnaryOp::Acos);
            b.push_const(Const::Scalar(POWER));
            b.binary(BinaryOp::Mul);
            let theta = b.add_slot(ValueType::Scalar);
            b.store_local(theta);

            // phi = atan2(z.y, z.x) * POWER
            b.load_local(z);
            b.unary(UnaryOp::VecY);
            b.load_local(z);
            b.unary(UnaryOp::VecX);
            b.binary(BinaryOp::Atan2);
            b.push_const(Const::Scalar(POWER));
            b.binary(BinaryOp::Mul);
            let phi = b.add_slot(ValueType::Scalar);
            b.store_local(phi);

            // dr = pow(r, POWER - 1) * POWER * dr + 1.0
            b.load_local(r);
            b.push_const(Const::Scalar(POWER - 1.0));
            b.binary(BinaryOp::Pow);
            b.push_const(Const::Scalar(POWER));
            b.binary(BinaryOp::Mul);
            b.load_local(dr);
            b.binary(BinaryOp::Mul);
            b.push_const(Const::Scalar(1.0));
            b.binary(BinaryOp::Add);
            b.store_local(dr);

            // zr = pow(r, POWER)
            b.load_local(r);
            b.push_const(Const::Scalar(POWER));
            b.binary(BinaryOp::Pow);
            let zr = b.add_slot(ValueType::Scalar);
            b.store_local(zr);

            // z = zr * vec3(sin(theta)*cos(phi), sin(phi)*sin(theta), cos(theta)) + pos
            b.load_local(theta);
            b.unary(UnaryOp::Sin);
            b.load_local(phi);
            b.unary(UnaryOp::Cos);
            b.binary(BinaryOp::Mul);

            b.load_local(phi);
            b.unary(UnaryOp::Sin);
            b.load_local(theta);
            b.unary(UnaryOp::Sin);
            b.binary(BinaryOp::Mul);

            b.load_local(theta);
            b.unary(UnaryOp::Cos);

            b.make_vec3();
            b.load_local(zr);
            b.binary(BinaryOp::Scale);
            b.push_pos();
            b.binary(BinaryOp::Add);
            b.store_local(z);
        }
        b.end_repeat().unwrap();

        // r_safe = max(r, EPS); distance = 0.5 * log(r_safe) * r_safe / dr
        // (avoids log(0) * 0 == NaN when the loop never escapes, e.g. at
        // the origin, which is always inside the Mandelbulb set.)
        b.load_local(r);
        b.push_const(Const::Scalar(1e-6));
        b.binary(BinaryOp::Max);
        let r_safe = b.add_slot(ValueType::Scalar);
        b.store_local(r_safe);

        b.push_const(Const::Scalar(0.5));
        b.load_local(r_safe);
        b.unary(UnaryOp::Log);
        b.binary(BinaryOp::Mul);
        b.load_local(r_safe);
        b.binary(BinaryOp::Mul);
        b.load_local(dr);
        b.binary(BinaryOp::Div);
        b.store_local(dist);

        b.build().expect("valid mandelbulb program")
    }

    #[test]
    fn mandelbulb_program_distance_estimate_is_plausible() {
        let program = mandelbulb_program();
        // Origin is always inside the Mandelbulb set.
        assert!(program.eval(0.0, 0.0, 0.0) <= 0.0);
        // A point clearly outside the bounding bulb must read positive and
        // roughly proportional to the true distance.
        let far = program.eval(5.0, 5.0, 5.0);
        assert!(far > 3.0, "expected a large positive DE, got {far}");
    }

    #[test]
    fn analytic_gradient_matches_expected_unit_normal_for_sphere() {
        let program = sphere_program(2.0);
        let g = program
            .analytic_gradient(2.0, 0.0, 0.0)
            .expect("differentiable away from the origin");
        assert!((g[0] - 1.0).abs() < 1e-4);
        assert!(g[1].abs() < 1e-4);
        assert!(g[2].abs() < 1e-4);

        let g2 = program.analytic_gradient(0.0, 0.0, -2.0).unwrap();
        assert!((g2[2] - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn analytic_gradient_is_none_at_non_differentiable_origin() {
        // length() at the zero vector has no analytic gradient (0/0).
        let program = sphere_program(1.0);
        assert!(program.analytic_gradient(0.0, 0.0, 0.0).is_none());
    }

    #[test]
    fn vector_ops_cover_dot_cross_length_normalize() {
        // f(pos) = dot(normalize(pos), cross(pos, pos + (1,0,0))) - length(pos) + length(pos)
        // reduces to dot(normalize(pos), cross(...)), a value we can hand-check at (0,1,0).
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0]);

        b.push_pos();
        b.unary(UnaryOp::Normalize);
        b.push_pos();
        b.push_pos();
        b.push_const(Const::Vec3([1.0, 0.0, 0.0]));
        b.binary(BinaryOp::Add);
        b.binary(BinaryOp::Cross);
        b.binary(BinaryOp::Dot);
        b.store_local(out);

        let program = b.build().unwrap();
        // pos = (0, 1, 0): normalize -> (0,1,0); pos+(1,0,0) = (1,1,0);
        // cross((0,1,0), (1,1,0)) = (1*0 - 0*1, 0*1 - 0*0, 0*1 - 1*1) = (0, 0, -1)
        // dot((0,1,0), (0,0,-1)) = 0
        assert!((program.eval(0.0, 1.0, 0.0) - 0.0).abs() < 1e-5);

        let mut lb = ProgramBuilder::new();
        let out2 = lb.add_slot(ValueType::Scalar);
        lb.set_output(out2);
        lb.set_bounds([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0]);
        lb.push_pos();
        lb.unary(UnaryOp::Length);
        lb.store_local(out2);
        let length_program = lb.build().unwrap();
        assert!((length_program.eval(3.0, 4.0, 0.0) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn scalar_math_ops_cover_min_max_clamp_abs_select() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-10.0, -10.0, -10.0], [10.0, 10.0, 10.0]);

        // select(x > 0, clamp(abs(x), 0, 2), min(x, max(x, -5))) with x = pos.x
        // Select pops (cond, b, a) — push order is a, b, cond.
        b.push_pos();
        b.unary(UnaryOp::VecX);
        b.unary(UnaryOp::Abs);
        b.push_const(Const::Scalar(0.0));
        b.push_const(Const::Scalar(2.0));
        b.clamp();

        b.push_pos();
        b.unary(UnaryOp::VecX);
        b.push_pos();
        b.unary(UnaryOp::VecX);
        b.push_const(Const::Scalar(-5.0));
        b.binary(BinaryOp::Max);
        b.binary(BinaryOp::Min);

        b.push_pos();
        b.unary(UnaryOp::VecX);
        b.push_const(Const::Scalar(0.0));
        b.binary(BinaryOp::Gt);

        b.select();
        b.store_local(out);

        let program = b.build().unwrap();
        assert!((program.eval(5.0, 0.0, 0.0) - 2.0).abs() < 1e-6); // clamp(5,0,2) = 2
        assert!((program.eval(-3.0, 0.0, 0.0) - (-3.0)).abs() < 1e-6); // min(-3, max(-3,-5)) = -3
    }

    #[test]
    fn sdf_node_program_composes_with_union_and_reports_bounds() {
        let program = sphere_program(2.0);
        let node = crate::sdf::SdfNode::Program {
            program: Box::new(program),
        };
        let other = crate::sdf::SdfNode::Translate {
            child: Box::new(crate::sdf::SdfNode::Sphere { radius: 1.0 }),
            offset: [10.0, 0.0, 0.0],
        };
        let union = crate::sdf::SdfNode::Union {
            children: vec![node, other],
        };
        assert!(union.eval(0.0, 0.0, 0.0) < 0.0);
        assert!(union.eval(10.0, 0.0, 0.0) < 0.0);
        assert!(union.eval(5.0, 0.0, 0.0) > 0.0);

        let bounds = union.bounds().expect("both children are bounded");
        assert!(bounds.min[0] <= -2.0);
        assert!(bounds.max[0] >= 11.0);
    }

    #[test]
    fn program_bounds_are_explicit_not_inferred() {
        // Deliberately loose bounds: much larger than the shape actually needs.
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds([-50.0, -50.0, -50.0], [50.0, 50.0, 50.0]);
        b.push_pos();
        b.unary(UnaryOp::Length);
        b.push_const(Const::Scalar(1.0));
        b.binary(BinaryOp::Sub);
        b.store_local(out);
        let program = b.build().unwrap();
        let aabb = program.aabb();
        assert_eq!(aabb.min, [-50.0, -50.0, -50.0]);
        assert_eq!(aabb.max, [50.0, 50.0, 50.0]);
    }

    #[test]
    fn extreme_finite_bounds_do_not_panic() {
        let mut b = ProgramBuilder::new();
        let out = b.add_slot(ValueType::Scalar);
        b.set_output(out);
        b.set_bounds(
            [f32::MIN, f32::MIN, f32::MIN],
            [f32::MAX, f32::MAX, f32::MAX],
        );
        b.push_const(Const::Scalar(0.0));
        b.store_local(out);
        let program = b.build().unwrap();
        assert_eq!(program.eval(f32::MAX, f32::MIN, 0.0), 0.0);
    }
}
