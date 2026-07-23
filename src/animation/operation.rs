//! Exact transform operations recorded by [`super::BuildAnimation`].

use serde::{Deserialize, Serialize};

use super::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope", content = "name", rename_all = "snake_case")]
pub enum OperationScope {
    DefaultRegion,
    Region(String),
    Schematic,
    StampRegion(String),
    StampBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationKind {
    Translate {
        delta: [i32; 3],
    },
    Rotate {
        axis: TransformAxis,
        quarter_turns: u8,
    },
    Flip {
        axis: TransformAxis,
    },
    Stamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationBounds {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl From<crate::bounding_box::BoundingBox> for OperationBounds {
    fn from(value: crate::bounding_box::BoundingBox) -> Self {
        Self {
            min: [value.min.0, value.min.1, value.min.2],
            max: [value.max.0, value.max.1, value.max.2],
        }
    }
}

impl OperationBounds {
    pub fn pivot2(self) -> [i64; 3] {
        [
            self.min[0] as i64 + self.max[0] as i64,
            self.min[1] as i64 + self.max[1] as i64,
            self.min[2] as i64 + self.max[2] as i64,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatticeAffine {
    pub linear: [[i8; 3]; 3],
    pub offset: [i64; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellDelta {
    pub region: String,
    pub before_position: [i32; 3],
    pub final_position: [i32; 3],
    pub before_block: String,
    pub final_block: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockEntityDelta {
    pub source_position: [i32; 3],
    pub final_position: [i32; 3],
    pub source: Option<crate::block_entity::BlockEntity>,
    pub replaced_destination: Option<crate::block_entity::BlockEntity>,
    pub final_state: Option<crate::block_entity::BlockEntity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDelta {
    pub before: crate::entity::Entity,
    pub final_state: crate::entity::Entity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationReceipt {
    pub start_ms: f32,
    pub duration_ms: f32,
    pub id: u32,
    pub scope: OperationScope,
    pub kind: OperationKind,
    pub before_bounds: Option<OperationBounds>,
    pub final_bounds: Option<OperationBounds>,
    pub pivot2: Option<[i64; 3]>,
    pub final_pivot2: Option<[i64; 3]>,
    pub affine: Option<LatticeAffine>,
    pub cells: Vec<CellDelta>,
    pub excluded_cells: Vec<[i32; 3]>,
    pub block_entities: Vec<BlockEntityDelta>,
    pub entities: Vec<EntityDelta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OperationTransform {
    Rotate {
        axis: TransformAxis,
        inverse_degrees: f32,
        pivot: [f32; 3],
        final_pivot: [f32; 3],
    },
    Translate {
        inverse_delta: [f32; 3],
    },
    Flip {
        axis: TransformAxis,
        plane: f32,
    },
}

impl OperationTransform {
    /// Matrix taking pre-operation geometry toward its authoritative final position.
    pub fn matrix_at(self, progress: f32) -> Mat4 {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            Self::Rotate {
                axis,
                inverse_degrees,
                pivot,
                final_pivot,
            } => {
                let center_delta = [
                    (final_pivot[0] - pivot[0]) * progress,
                    (final_pivot[1] - pivot[1]) * progress,
                    (final_pivot[2] - pivot[2]) * progress,
                ];
                multiply(
                    translation(center_delta),
                    rotation_about(axis, -inverse_degrees * progress, pivot),
                )
            }
            Self::Translate { inverse_delta } => translation(inverse_delta.map(|v| -v * progress)),
            Self::Flip { axis, plane } => {
                let mut scale = [1.0; 3];
                scale[axis_index(axis)] = 1.0 - 2.0 * progress;
                scale_about(scale, axis, plane)
            }
        }
    }

    /// Exact endpoint matrix mapping final coordinates back to source coordinates.
    pub fn inverse_matrix(self) -> Mat4 {
        match self {
            Self::Rotate {
                axis,
                inverse_degrees,
                pivot,
                final_pivot,
            } => multiply(
                rotation_about(axis, inverse_degrees, pivot),
                translation([
                    pivot[0] - final_pivot[0],
                    pivot[1] - final_pivot[1],
                    pivot[2] - final_pivot[2],
                ]),
            ),
            Self::Translate { inverse_delta } => translation(inverse_delta),
            Self::Flip { axis, plane } => {
                let mut scale = [1.0; 3];
                scale[axis_index(axis)] = -1.0;
                scale_about(scale, axis, plane)
            }
        }
    }
}

pub fn identity() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Column-major matrix product `a * b`.
pub fn multiply(a: Mat4, b: Mat4) -> Mat4 {
    let mut out = [[0.0; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            out[col][row] = (0..4).map(|k| a[k][row] * b[col][k]).sum();
        }
    }
    out
}

pub fn transform_point(m: Mat4, p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0],
        m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1],
        m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2],
    ]
}

fn translation(delta: [f32; 3]) -> Mat4 {
    let mut out = identity();
    out[3][0] = delta[0];
    out[3][1] = delta[1];
    out[3][2] = delta[2];
    out
}

fn axis_index(axis: TransformAxis) -> usize {
    match axis {
        TransformAxis::X => 0,
        TransformAxis::Y => 1,
        TransformAxis::Z => 2,
    }
}

fn rotation_about(axis: TransformAxis, degrees: f32, pivot: [f32; 3]) -> Mat4 {
    let (s, c) = degrees.to_radians().sin_cos();
    let mut r = identity();
    match axis {
        TransformAxis::X => {
            r[1][1] = c;
            r[1][2] = s;
            r[2][1] = -s;
            r[2][2] = c;
        }
        TransformAxis::Y => {
            r[0][0] = c;
            r[0][2] = -s;
            r[2][0] = s;
            r[2][2] = c;
        }
        TransformAxis::Z => {
            r[0][0] = c;
            r[0][1] = s;
            r[1][0] = -s;
            r[1][1] = c;
        }
    }
    multiply(
        translation(pivot),
        multiply(r, translation(pivot.map(|v| -v))),
    )
}

fn scale_about(scale: [f32; 3], axis: TransformAxis, plane: f32) -> Mat4 {
    let mut s = identity();
    s[0][0] = scale[0];
    s[1][1] = scale[1];
    s[2][2] = scale[2];
    let mut pivot = [0.0; 3];
    pivot[axis_index(axis)] = plane;
    multiply(
        translation(pivot),
        multiply(s, translation(pivot.map(|v| -v))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-4)
    }

    fn apply(m: Mat4, p: [f32; 3]) -> [f32; 3] {
        [
            m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0],
            m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1],
            m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2],
        ]
    }

    #[test]
    fn y_forward_rotation_about_discrete_pivot_reaches_final() {
        let op = OperationTransform::Rotate {
            axis: TransformAxis::Y,
            inverse_degrees: 90.0,
            pivot: [10.0, 0.0, 10.0],
            final_pivot: [10.0, 0.0, 10.0],
        };
        assert!(close(
            apply(op.matrix_at(0.0), [11.0, 0.0, 10.0]),
            [11.0, 0.0, 10.0]
        ));
        assert!(close(
            apply(op.matrix_at(1.0), [11.0, 0.0, 10.0]),
            [10.0, 0.0, 11.0]
        ));
        assert!(close(
            apply(op.inverse_matrix(), [10.0, 0.0, 11.0]),
            [11.0, 0.0, 10.0]
        ));
    }
}
