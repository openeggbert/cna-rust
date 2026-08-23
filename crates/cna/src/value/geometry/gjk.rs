use crate::value::{MathHelper, Vector3};

const BITS_TO_INDICES: [i32; 16] = [
    0, 1, 2, 17, 3, 25, 26, 209, 4, 33, 34, 273, 35, 281, 282, 2257,
];

pub(super) struct Gjk {
    closest_point: Vector3,
    y: [Vector3; 4],
    y_length_squared: [f32; 4],
    edges: [[Vector3; 4]; 4],
    edge_length_squared: [[f32; 4]; 4],
    determinants: [[f32; 4]; 16],
    simplex_bits: usize,
    max_length_squared: f32,
}

impl Default for Gjk {
    fn default() -> Self {
        Self {
            closest_point: Vector3::Zero,
            y: [Vector3::Zero; 4],
            y_length_squared: [0.0; 4],
            edges: [[Vector3::Zero; 4]; 4],
            edge_length_squared: [[0.0; 4]; 4],
            determinants: [[0.0; 4]; 16],
            simplex_bits: 0,
            max_length_squared: 0.0,
        }
    }
}

impl Gjk {
    pub(super) fn full_simplex(&self) -> bool {
        self.simplex_bits == 15
    }
    pub(super) fn max_length_squared(&self) -> f32 {
        self.max_length_squared
    }
    pub(super) fn closest_point(&self) -> Vector3 {
        self.closest_point
    }

    pub(super) fn reset(&mut self) {
        self.simplex_bits = 0;
        self.max_length_squared = 0.0;
    }

    pub(super) fn add_support_point(&mut self, new_point: Vector3) -> bool {
        let new_index = ((BITS_TO_INDICES[self.simplex_bits ^ 15] & 7) - 1) as usize;
        self.y[new_index] = new_point;
        self.y_length_squared[new_index] = new_point.LengthSquared();
        let mut encoded = BITS_TO_INDICES[self.simplex_bits];
        while encoded != 0 {
            let index = ((encoded & 7) - 1) as usize;
            let edge = self.y[index] - new_point;
            self.edges[index][new_index] = edge;
            self.edges[new_index][index] = -edge;
            let length = edge.LengthSquared();
            self.edge_length_squared[index][new_index] = length;
            self.edge_length_squared[new_index][index] = length;
            encoded >>= 3;
        }
        self.update_determinant(new_index);
        self.update_simplex(new_index)
    }

    fn dot(a: Vector3, b: Vector3) -> f32 {
        a.X * b.X + a.Y * b.Y + a.Z * b.Z
    }

    #[allow(clippy::many_single_char_names)]
    fn update_determinant(&mut self, new_index: usize) {
        let new_bit = 1 << new_index;
        self.determinants[new_bit][new_index] = 1.0;
        let all_encoded = BITS_TO_INDICES[self.simplex_bits];
        let mut encoded = all_encoded;
        let mut earlier_count = 0;
        while encoded != 0 {
            let old_index = ((encoded & 7) - 1) as usize;
            let old_bit = 1 << old_index;
            let pair = old_bit | new_bit;
            self.determinants[pair][old_index] =
                Self::dot(self.edges[new_index][old_index], self.y[new_index]);
            self.determinants[pair][new_index] =
                Self::dot(self.edges[old_index][new_index], self.y[old_index]);
            let mut earlier = all_encoded;
            for _ in 0..earlier_count {
                let third_index = ((earlier & 7) - 1) as usize;
                let third_bit = 1 << third_index;
                let triple = pair | third_bit;

                let edge_index = if self.edge_length_squared[old_index][third_index]
                    < self.edge_length_squared[new_index][third_index]
                {
                    old_index
                } else {
                    new_index
                };
                self.determinants[triple][third_index] = self.determinants[pair][old_index]
                    * Self::dot(self.edges[edge_index][third_index], self.y[old_index])
                    + self.determinants[pair][new_index]
                        * Self::dot(self.edges[edge_index][third_index], self.y[new_index]);

                let edge_index = if self.edge_length_squared[third_index][old_index]
                    < self.edge_length_squared[new_index][old_index]
                {
                    third_index
                } else {
                    new_index
                };
                self.determinants[triple][old_index] = self.determinants[third_bit | new_bit]
                    [third_index]
                    * Self::dot(self.edges[edge_index][old_index], self.y[third_index])
                    + self.determinants[third_bit | new_bit][new_index]
                        * Self::dot(self.edges[edge_index][old_index], self.y[new_index]);

                let edge_index = if self.edge_length_squared[old_index][new_index]
                    < self.edge_length_squared[third_index][new_index]
                {
                    old_index
                } else {
                    third_index
                };
                self.determinants[triple][new_index] = self.determinants[old_bit | third_bit]
                    [third_index]
                    * Self::dot(self.edges[edge_index][new_index], self.y[third_index])
                    + self.determinants[old_bit | third_bit][old_index]
                        * Self::dot(self.edges[edge_index][new_index], self.y[old_index]);
                earlier >>= 3;
            }
            encoded >>= 3;
            earlier_count += 1;
        }

        if self.simplex_bits | new_bit == 15 {
            let mut edge_index =
                if !(self.edge_length_squared[1][0] < self.edge_length_squared[2][0]) {
                    if self.edge_length_squared[2][0] < self.edge_length_squared[3][0] {
                        2
                    } else {
                        3
                    }
                } else if self.edge_length_squared[1][0] < self.edge_length_squared[3][0] {
                    1
                } else {
                    3
                };
            self.determinants[15][0] = self.determinants[14][1]
                * Self::dot(self.edges[edge_index][0], self.y[1])
                + self.determinants[14][2] * Self::dot(self.edges[edge_index][0], self.y[2])
                + self.determinants[14][3] * Self::dot(self.edges[edge_index][0], self.y[3]);

            edge_index = if !(self.edge_length_squared[0][1] < self.edge_length_squared[2][1]) {
                if self.edge_length_squared[2][1] < self.edge_length_squared[3][1] {
                    2
                } else {
                    3
                }
            } else if !(self.edge_length_squared[0][1] < self.edge_length_squared[3][1]) {
                3
            } else {
                0
            };
            self.determinants[15][1] = self.determinants[13][0]
                * Self::dot(self.edges[edge_index][1], self.y[0])
                + self.determinants[13][2] * Self::dot(self.edges[edge_index][1], self.y[2])
                + self.determinants[13][3] * Self::dot(self.edges[edge_index][1], self.y[3]);

            edge_index = if !(self.edge_length_squared[0][2] < self.edge_length_squared[1][2]) {
                if self.edge_length_squared[1][2] < self.edge_length_squared[3][2] {
                    1
                } else {
                    3
                }
            } else if !(self.edge_length_squared[0][2] < self.edge_length_squared[3][2]) {
                3
            } else {
                0
            };
            self.determinants[15][2] = self.determinants[11][0]
                * Self::dot(self.edges[edge_index][2], self.y[0])
                + self.determinants[11][1] * Self::dot(self.edges[edge_index][2], self.y[1])
                + self.determinants[11][3] * Self::dot(self.edges[edge_index][2], self.y[3]);

            edge_index = if !(self.edge_length_squared[0][3] < self.edge_length_squared[1][3]) {
                if self.edge_length_squared[1][3] < self.edge_length_squared[2][3] {
                    1
                } else {
                    2
                }
            } else if !(self.edge_length_squared[0][3] < self.edge_length_squared[2][3]) {
                2
            } else {
                0
            };
            self.determinants[15][3] = self.determinants[7][0]
                * Self::dot(self.edges[edge_index][3], self.y[0])
                + self.determinants[7][1] * Self::dot(self.edges[edge_index][3], self.y[1])
                + self.determinants[7][2] * Self::dot(self.edges[edge_index][3], self.y[2]);
        }
    }

    fn update_simplex(&mut self, new_index: usize) -> bool {
        let all_bits = self.simplex_bits | (1 << new_index);
        let new_bit = 1 << new_index;
        let mut subset = self.simplex_bits;
        while subset != 0 {
            if subset & all_bits == subset && self.satisfies_rule(subset | new_bit, all_bits) {
                self.simplex_bits = subset | new_bit;
                self.closest_point = self.compute_closest_point();
                return true;
            }
            subset -= 1;
        }
        if self.satisfies_rule(new_bit, all_bits) {
            self.simplex_bits = new_bit;
            self.closest_point = self.y[new_index];
            self.max_length_squared = self.y_length_squared[new_index];
            true
        } else {
            false
        }
    }

    fn compute_closest_point(&mut self) -> Vector3 {
        let mut denominator = 0.0;
        let mut result = Vector3::Zero;
        self.max_length_squared = 0.0;
        let mut encoded = BITS_TO_INDICES[self.simplex_bits];
        while encoded != 0 {
            let index = ((encoded & 7) - 1) as usize;
            let determinant = self.determinants[self.simplex_bits][index];
            denominator += determinant;
            result += self.y[index] * determinant;
            self.max_length_squared =
                MathHelper::Max(self.max_length_squared, self.y_length_squared[index]);
            encoded >>= 3;
        }
        result / denominator
    }

    fn satisfies_rule(&self, x_bits: usize, y_bits: usize) -> bool {
        let mut encoded = BITS_TO_INDICES[y_bits];
        while encoded != 0 {
            let index = ((encoded & 7) - 1) as usize;
            let bit = 1 << index;
            if bit & x_bits != 0 {
                if self.determinants[x_bits][index] <= 0.0 {
                    return false;
                }
            } else if self.determinants[x_bits | bit][index] > 0.0 {
                return false;
            }
            encoded >>= 3;
        }
        true
    }
}
