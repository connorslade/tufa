use std::f32::consts::FRAC_PI_2;

#[cfg(feature = "interactive")]
use egui::{Context, Key, Ui, WidgetText};
use nalgebra::{Matrix4, Point3, Vector3};
#[cfg(feature = "interactive")]
use winit::event::DeviceEvent;

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    pub position: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl PerspectiveCamera {
    pub fn with_far(self, far: f32) -> Self {
        Self { far, ..self }
    }

    pub fn with_position(self, position: Vector3<f32>) -> Self {
        Self { position, ..self }
    }

    pub fn with_yaw(self, yaw: f32) -> Self {
        Self { yaw, ..self }
    }

    pub fn with_pitch(self, pitch: f32) -> Self {
        Self { pitch, ..self }
    }
}

impl PerspectiveCamera {
    #[cfg(feature = "interactive")]
    pub fn update(&mut self, ctx: &Context) {
        ctx.input(|input| {
            let facing = self.facing();
            let forward = Vector3::new(facing.x, 0.0, facing.z).normalize();
            let right = facing.cross(&Vector3::new(0.0, 1.0, 0.0));
            let directions = [
                (Key::W, forward),
                (Key::S, -forward),
                (Key::A, -right),
                (Key::D, right),
                (Key::Space, Vector3::new(0.0, 1.0, 0.0)),
            ];

            let mut delta = Vector3::zeros();
            delta -= Vector3::new(0.0, 1.0, 0.0) * input.modifiers.shift as u8 as f32;
            for (key, direction) in directions.iter() {
                delta += direction * input.key_down(*key) as u8 as f32;
            }

            self.position += delta.try_normalize(0.0).unwrap_or_default() * input.stable_dt;
        });
    }

    #[cfg(feature = "interactive")]
    pub fn device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.yaw -= delta.0 as f32 * 0.01;
            self.pitch -= delta.1 as f32 * 0.01;
        }
    }

    #[cfg(feature = "interactive")]
    pub fn ui(&mut self, ui: &mut Ui, name: impl Into<WidgetText>) {
        use crate::interactive::ui::{dragger, vec3_dragger};

        ui.collapsing(name, |ui| {
            ui.horizontal(|ui| {
                ui.label("Position");
                vec3_dragger(ui, &mut self.position, |x| x.speed(0.1));
            });
            dragger(ui, "Pitch", &mut self.pitch, |x| x.speed(0.1));
            dragger(ui, "Yaw", &mut self.yaw, |x| x.speed(0.1));
            ui.separator();
            dragger(ui, "Fov", &mut self.fov, |x| x.speed(0.1));
            dragger(ui, "Near", &mut self.near, |x| x.speed(0.1));
            dragger(ui, "Far", &mut self.far, |x| x.speed(0.1));
        });
    }

    pub fn facing(&self) -> Vector3<f32> {
        Vector3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    pub fn view_projection(&self, aspect: f32) -> Matrix4<f32> {
        let facing = self.facing();
        Matrix4::new_perspective(aspect, self.fov, self.near, self.far)
            * Matrix4::look_at_rh(
                &Point3::from(self.position),
                &Point3::from(self.position + facing),
                &Vector3::new(0.0, 1.0, 0.0),
            )
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        PerspectiveCamera {
            position: Vector3::zeros(),
            pitch: 0.0,
            yaw: 0.0,

            fov: FRAC_PI_2,
            near: 0.1,
            far: 10_000.0,
        }
    }
}
