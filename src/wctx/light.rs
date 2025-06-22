

use serde::{
    Serialize,
    Deserialize
};

use cgmath::{
    Vector3,
    Point3
};

use crate::wctx::camera::Camera;



pub trait Light {
    fn get_data(&self) -> PointLight;
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    position: [f32; 3],
    radius: f32,
    color: [f32; 3],
    padding: f32
}

#[derive(Serialize, Deserialize)]
pub struct StaticLight {
    radius: f32,
    position: [f32; 3],
    color: [f32; 3]
}

impl StaticLight {
    pub fn new( radius: f32, position: [f32; 3], color: [f32; 3] ) -> StaticLight {
        Self {
            radius,
            position,
            color
        }
    }
}

impl Light for StaticLight {
    fn get_data(&self) -> PointLight {
        PointLight {
            radius: self.radius,
            position: self.position,
            color: self.color,
            padding: 0.0
        }
    }
}



pub struct PlayerCameraLight {
    radius: f32,
    position: Point3<f32>,
    color: [f32; 3]
}

impl PlayerCameraLight {
    pub fn new( radius: f32, position: Point3<f32>, color: [f32; 3] ) -> PlayerCameraLight {
        Self {
            radius,
            position,
            color
        }
    }

    pub fn update(&mut self, cam: &Camera) {
        self.position = cam.position.clone();
    }
}

impl Light for PlayerCameraLight {
    fn get_data(&self) -> PointLight {
        PointLight {
            radius: self.radius,
            position: [self.position.x, self.position.y, self.position.z],
            color: self.color,
            padding: 0.0
        }
    }
}
