use egui::{emath::Numeric, DragValue, Ui};
use nalgebra::{Vector2, Vector3};

pub fn dragger<Num: Numeric>(
    ui: &mut Ui,
    label: &str,
    value: &mut Num,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(value)));
        ui.label(label);
    });
}

pub fn vec2_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut Vector2<Num>,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("×");
        ui.add(func(DragValue::new(&mut val[1])));
    });
}

pub fn vec3_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut Vector3<Num>,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("×");
        ui.add(func(DragValue::new(&mut val[1])));
        ui.label("×");
        ui.add(func(DragValue::new(&mut val[2])));
    });
}
