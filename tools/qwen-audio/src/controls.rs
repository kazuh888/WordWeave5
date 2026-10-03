use eframe::egui::*;
pub(crate) fn button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let first = ui
        .ctx()
        .graphics_mut(|g| g.entry(ui.layer_id()).next_idx().0);
    let response = ui.add_enabled(enabled, Button::new(label).wrap().min_size(vec2(0.0, 36.0)));
    ui.ctx().graphics_mut(|g| {
        let list = g.entry(ui.layer_id());
        for index in first..list.next_idx().0 {
            list.mutate_shape(layers::ShapeIdx(index), |shape| {
                if let Shape::Text(t) = &mut shape.shape {
                    let bounds =
                        if t.galley.mesh_bounds.is_finite() && t.galley.mesh_bounds.is_positive() {
                            t.galley.mesh_bounds
                        } else {
                            t.galley.rect
                        };
                    t.pos = response.rect.center() - bounds.center().to_vec2();
                }
            })
        }
    });
    response
}
