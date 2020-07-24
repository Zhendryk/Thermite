use glm;

const YAW_DEFAULT: f32 = -90.0;
const PITCH_DEFAULT: f32 = 0.0;
const SPEED_DEFAULT: f32 = 2.5;
const SENSITIVITY_DEFAULT: f32 = 0.1;
const ZOOM_DEFAULT: f32 = 45.0;

pub enum CameraMovementDirection {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
}

pub struct Camera {
    position: glm::Vec3,
    front: glm::Vec3,
    up: glm::Vec3,
    right: glm::Vec3,
    world_up: glm::Vec3,
    yaw: f32,
    pitch: f32,
    movement_speed: f32,
    mouse_sensitivity: f32,
    zoom: f32,
}

impl Camera {
    /// Construct a new `Camera`
    ///
    /// ### Returns
    ///
    /// - A new `Camera` where `position => glm::vec3(0.0, 0.0, 0.0)`, `front => glm::vec3(0.0, 0.0, -1.0)`, `up => glm::vec3(0.0, 1.0, 0.0)`, `yaw => -90.0` and `pitch => 0.0`
    pub fn new() -> Camera {
        let front = glm::vec3(0.0, 0.0, -1.0);
        let up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::cross(&front, &up);
        Camera {
            position: glm::vec3(0.0, 0.0, 0.0),
            front: front,
            up: up,
            right: right,
            world_up: up,
            yaw: YAW_DEFAULT,
            pitch: PITCH_DEFAULT,
            movement_speed: SPEED_DEFAULT,
            mouse_sensitivity: SENSITIVITY_DEFAULT,
            zoom: ZOOM_DEFAULT,
        }
    }
    /// Construct a new `Camera` with the given `position`, `up` direction, `yaw` and `pitch`
    ///
    /// ### Returns
    ///
    /// - A new `Camera` where `position => position`, `front => glm::vec3(0.0, 0.0, -1.0)`, `up => up`, `yaw => yaw` and `pitch => pitch`
    pub fn new_from_vec(position: glm::Vec3, up: glm::Vec3, yaw: f32, pitch: f32) -> Camera {
        let front = glm::vec3(0.0, 0.0, -1.0);
        let right = glm::cross(&front, &up);
        Camera {
            position: position,
            front: front,
            up: up,
            right: right,
            world_up: up,
            yaw: yaw,
            pitch: pitch,
            movement_speed: SPEED_DEFAULT,
            mouse_sensitivity: SENSITIVITY_DEFAULT,
            zoom: ZOOM_DEFAULT,
        }
    }
    /// Construct a new `Camera` with the given `position`, `up` direction, `yaw` and `pitch` (as separated components)
    ///
    /// ### Returns
    ///
    /// - A new `Camera` where `position => glm::vec3(pos_x, pos_y, pos_z)`, `front => glm::vec3(0.0, 0.0, -1.0)`, `up => glm::vec3(up_x, up_y, up_z)`, `yaw => yaw` and `pitch => pitch`
    pub fn new_from_comps(
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
        yaw: f32,
        pitch: f32,
    ) -> Camera {
        let position = glm::vec3(pos_x, pos_y, pos_z);
        let up = glm::vec3(up_x, up_y, up_z);
        let front = glm::vec3(0.0, 0.0, -1.0);
        let right = glm::cross(&front, &up);
        Camera {
            position: position,
            front: front,
            up: up,
            right: right,
            world_up: up,
            yaw: yaw,
            pitch: pitch,
            movement_speed: SPEED_DEFAULT,
            mouse_sensitivity: SENSITIVITY_DEFAULT,
            zoom: ZOOM_DEFAULT,
        }
    }

    /// Returns a reference to this `Camera`'s `position` vector
    pub fn position(&self) -> &glm::Vec3 {
        &self.position
    }
    /// Returns a reference to this `Camera`'s `front` vector
    pub fn front(&self) -> &glm::Vec3 {
        &self.front
    }
    /// Returns a reference to this `Camera`'s `up` vector
    pub fn up(&self) -> &glm::Vec3 {
        &self.up
    }
    /// Returns a reference to this `Camera`'s `movement_speed` value
    pub fn speed(&self) -> &f32 {
        &self.movement_speed
    }

    /// Returns the view matrix calculated using Euler Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> glm::TMat4<f32> {
        glm::look_at(&self.position, &(self.position + self.front), &self.up)
    }

    /// Process keystrokes to move this `Camera` in the given `CameraMovementDirection`
    pub fn process_keyboard(&mut self, direction: CameraMovementDirection, delta_time: &f32) {
        let velocity = self.movement_speed * delta_time;
        match direction {
            CameraMovementDirection::FORWARD => self.position += self.front * velocity,
            CameraMovementDirection::BACKWARD => self.position -= self.front * velocity,
            CameraMovementDirection::LEFT => self.position -= self.right * velocity,
            CameraMovementDirection::RIGHT => self.position += self.right * velocity,
        }
    }

    /// Processes mouse movement across the screen to mutate this `Camera`'s `yaw` and `pitch`
    pub fn process_mouse_move(&mut self, x_offset: f32, y_offset: f32, constrain_pitch: bool) {
        self.yaw += x_offset * self.mouse_sensitivity;
        self.pitch += y_offset * self.mouse_sensitivity;
        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            }
            if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }
        self.update_vectors()
    }

    /// Processes mouse scrolls to mutate this `Camera`'s `zoom`
    pub fn process_mouse_scroll(&mut self, y_offset: f32) {
        self.zoom -= y_offset;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
        if self.zoom > 45.0 {
            self.zoom = 45.0;
        }
    }

    /// Updates the `front`, `right` and `up` vectors for this `Camera` based on the current `yaw` and `pitch`
    fn update_vectors(&mut self) {
        let front_x = radians(&self.yaw).cos() * radians(&self.pitch).cos();
        let front_y = radians(&self.pitch).sin();
        let front_z = radians(&self.yaw).sin() * radians(&self.pitch).cos();
        self.front = glm::normalize(&glm::vec3(front_x, front_y, front_z));
        self.right = glm::normalize(&glm::cross(&self.front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &self.front))
    }
}

/// Converts degrees to radians
fn radians(deg: &f32) -> f32 {
    glm::radians(&glm::vec1(*deg)).x
}
