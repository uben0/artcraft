use mat::{Affine, AffineTrait, VectorTrait};

const RADIAN: f32 = 2.0 * std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pos: [f32; 3],
    pub h_angle: f32, // horizontal
    pub v_angle: f32, // vertical
}
impl Camera {
    // compute the rendering matrix which is the inverse of
    // camera positioning matrix
    pub fn projector(&self) -> [[f32; 4]; 4] {
        Affine::identity()
            .affine_x_rotate(-self.v_angle)
            .affine_y_rotate(-self.h_angle)
            .affine_translate(self.pos.vector_neg())
    }

    // compute camera positioning matrix as it is not directly
    // stored because it not practical to move and oritentate
    // the player with it
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        Affine::identity()
            .affine_translate(self.pos)
            .affine_y_rotate(self.h_angle)
            .affine_x_rotate(self.v_angle)
    }

    // rotate player horizontaly by given delta
    pub fn delta_angle_h(&mut self, d: f32) {
        self.h_angle += d;
        while self.h_angle >= RADIAN {
            self.h_angle -= RADIAN;
        }
        while self.h_angle < 0.0 {
            self.h_angle += RADIAN;
        }
    }
    // rotate player vertically by given delta
    pub fn delta_angle_v(&mut self, d: f32) {
        self.v_angle = (self.v_angle + d).max(-RADIAN / 4.0).min(RADIAN / 4.0);
    }

    // as vertical orientation does not affect movement
    // only the horizontal orientation is considered
    pub fn move_matrix(&self) -> [[f32; 3]; 3] {
        Affine::<f32, 3>::y_rotate(self.h_angle)
    }

    // move player by specified vector
    pub fn delta_pos(&mut self, vector: [f32; 3]) {
        self.pos.vector_add_assign(vector);
    }
}
