use glam::Vec2Swizzles;
pub struct Camera {
    pub rot: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,

    pub aspect: f32,
    pub fovy: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new(sc_desc: &wgpu::SwapChainDescriptor, fovy: f32, zoom: f32) -> Self {
        Self {
            rot: glam::vec3(1.0, 1.0, 1.0).normalize(),
            target: glam::vec3(0.0, 0.0, 0.0),
            up: glam::Vec3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy,
            zoom,
        }
    }

    pub fn build(&self, perspective: bool) -> glam::Mat4 {
        let margin = 1.15;
        if perspective {
            let dist = (margin) / f32::tan(self.fovy / 2.0);
            let view = glam::Mat4::look_at_rh(
                self.target + (dist * self.zoom * self.rot),
                self.target,
                self.up,
            );
            let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, 0.01, 1000.0);
            proj * view
        } else {
            let view = glam::Mat4::look_at_rh(self.target + self.rot, self.target, self.up);
            let zoom = margin * self.zoom;
            let proj = glam::Mat4::orthographic_rh(
                -zoom * self.aspect,
                zoom * self.aspect,
                -zoom,
                zoom,
                -1000.0,
                1000.0,
            );
            proj * view
        }
    }

    pub fn pan(&mut self, mut dir: glam::Vec2, length: f32) {
        let angle = f32::atan2(self.rot.z, self.rot.x);
        dir.y /= -self.aspect;
        let rotation = glam::Mat2::from_angle(angle);
        let target = rotation.mul_vec2(dir.yx());
        let mov = length * self.zoom * glam::Vec3::new(target.x, 0.0, target.y);
        self.target += mov;
    }

    pub fn view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(self.rot, self.target, self.up)
    }

    pub fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor) {
        self.aspect = sc_desc.width as f32 / sc_desc.height as f32
    }

    pub fn rotate(&mut self, dir: glam::Vec2, length: f32) {
        let forward = self.rot;
        let right = forward.cross(self.up);
        let m_y = f32::cos(self.up.dot(forward));
        let rot = forward + (right * -dir.x * -length) + (self.up * dir.y * length / m_y);
        self.rot = rot.normalize();
    }
}
