use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BoundingBox {
    pub min: (i32, i32, i32),
    pub max: (i32, i32, i32),
}

impl BoundingBox {
    pub fn new(min: (i32, i32, i32), max: (i32, i32, i32)) -> Self {
        BoundingBox { min, max }
    }

    pub fn contains(&self, point: (i32, i32, i32)) -> bool {
        point.0 >= self.min.0
            && point.0 <= self.max.0
            && point.1 >= self.min.1
            && point.1 <= self.max.1
            && point.2 >= self.min.2
            && point.2 <= self.max.2
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.0 <= other.max.0
            && self.max.0 >= other.min.0
            && self.min.1 <= other.max.1
            && self.max.1 >= other.min.1
            && self.min.2 <= other.max.2
            && self.max.2 >= other.min.2
    }

    pub fn intersects_range(
        &self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> bool {
        !(self.max.0 < min_x
            || self.min.0 >= max_x
            || self.max.1 < min_y
            || self.min.1 >= max_y
            || self.max.2 < min_z
            || self.min.2 >= max_z)
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: (
                self.min.0.min(other.min.0),
                self.min.1.min(other.min.1),
                self.min.2.min(other.min.2),
            ),
            max: (
                self.max.0.max(other.max.0),
                self.max.1.max(other.max.1),
                self.max.2.max(other.max.2),
            ),
        }
    }

    pub fn coords_to_index(&self, x: i32, y: i32, z: i32) -> usize {
        let (width, _, length) = self.get_dimensions();
        let dx = x - self.min.0;
        let dy = y - self.min.1;
        let dz = z - self.min.2;
        (dx + dz * width + dy * width * length) as usize
    }

    pub fn index_to_coords(&self, index: usize) -> (i32, i32, i32) {
        let (width, _, length) = self.get_dimensions();
        let dx = (index % width as usize) as i32;
        let dy = (index / (width * length) as usize) as i32;
        let dz = ((index / width as usize) % length as usize) as i32;
        (dx + self.min.0, dy + self.min.1, dz + self.min.2)
    }

    pub fn get_dimensions(&self) -> (i32, i32, i32) {
        (
            (self.max.0 - self.min.0 + 1),
            (self.max.1 - self.min.1 + 1),
            (self.max.2 - self.min.2 + 1),
        )
    }

    pub fn to_position_and_size(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        (self.min, self.get_dimensions())
    }

    pub fn try_from_position_and_size(
        position: (i32, i32, i32),
        size: (i32, i32, i32),
    ) -> Result<Self, String> {
        fn endpoint(position: i32, size: i32) -> Option<i32> {
            let inclusive_delta = size - size.signum();
            position.checked_add(inclusive_delta)
        }

        let end = (
            endpoint(position.0, size.0)
                .ok_or_else(|| "Bounding box X endpoint overflow".to_string())?,
            endpoint(position.1, size.1)
                .ok_or_else(|| "Bounding box Y endpoint overflow".to_string())?,
            endpoint(position.2, size.2)
                .ok_or_else(|| "Bounding box Z endpoint overflow".to_string())?,
        );
        Ok(BoundingBox::new(
            (
                position.0.min(end.0),
                position.1.min(end.1),
                position.2.min(end.2),
            ),
            (
                position.0.max(end.0),
                position.1.max(end.1),
                position.2.max(end.2),
            ),
        ))
    }

    pub fn from_position_and_size(position: (i32, i32, i32), size: (i32, i32, i32)) -> Self {
        Self::try_from_position_and_size(position, size)
            .expect("bounding box position and size exceed i32 coordinates")
    }
    pub fn volume(&self) -> u64 {
        let (width, height, length) = self.get_dimensions();
        width as u64 * height as u64 * length as u64
    }
}

#[cfg(test)]
mod tests {
    use super::BoundingBox;

    #[test]
    fn one_cell_boxes_are_valid_at_signed_endpoints() {
        let min = BoundingBox::from_position_and_size((i32::MIN, 0, 0), (1, 1, 1));
        assert_eq!(min.min, (i32::MIN, 0, 0));
        assert_eq!(min.max, (i32::MIN, 0, 0));

        let max = BoundingBox::from_position_and_size((i32::MAX, 0, 0), (1, 1, 1));
        assert_eq!(max.min, (i32::MAX, 0, 0));
        assert_eq!(max.max, (i32::MAX, 0, 0));
    }
}
