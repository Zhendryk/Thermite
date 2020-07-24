use std::ops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl ops::Add<Vector3f> for Vector3f {
    type Output = Vector3f;

    fn add(self, _rhs: Vector3f) -> Vector3f {
        Vector3f {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
            z: self.z + _rhs.z,
        }
    }
}

impl ops::Sub<Vector3f> for Vector3f {
    type Output = Vector3f;

    fn sub(self, _rhs: Vector3f) -> Vector3f {
        Vector3f {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
            z: self.z - _rhs.z,
        }
    }
}

impl ops::Mul<i32> for Vector3f {
    type Output = Vector3f;

    fn mul(self, _rhs: i32) -> Vector3f {
        Vector3f {
            x: self.x * _rhs as f32,
            y: self.y * _rhs as f32,
            z: self.z * _rhs as f32,
        }
    }
}

impl ops::Mul<u32> for Vector3f {
    type Output = Vector3f;

    fn mul(self, _rhs: u32) -> Vector3f {
        Vector3f {
            x: self.x * _rhs as f32,
            y: self.y * _rhs as f32,
            z: self.z * _rhs as f32,
        }
    }
}

impl Vector3f {
    /// Returns the dot product of this `Vector3f` and another
    ///
    /// If A and B are perpendicular (at 90 degrees to each other), the result of the dot product will be zero, because cos(Θ) will be zero.
    /// If the angle between A and B are less than 90 degrees, the dot product will be positive (greater than zero), as cos(Θ) will be positive, and the vector lengths are always positive values.
    /// If the angle between A and B are greater than 90 degrees, the dot product will be negative (less than zero), as cos(Θ) will be negative, and the vector lengths are always positive values.
    pub fn dot(&self, _rhs: &Vector3f) -> f32 {
        (self.x * _rhs.x) + (self.y * _rhs.y) + (self.z * _rhs.z)
    }

    /// Returns the length of this `Vector3f`
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// Returns the angle (in radians) between this `Vector3f` and the given `Vector3f`
    pub fn angle_between_self_and(&self, _other: &Vector3f) -> f32 {
        (self.dot(_other) / (self.magnitude().abs() * _other.magnitude().abs())).acos()
    }

    /// Negates this `Vector3f` (reverses the direction)
    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }
}
