use crate::gui::InspectedEntity;
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::Selectable;
use egregoria::pedestrians::Location;
use egregoria::Egregoria;
use geom::Color;

#[profiling::function]
pub fn inspected_aura(goria: &Egregoria, uiworld: &mut UiWorld) {
    let inspected = uiworld.write::<InspectedEntity>();
    let map = goria.map();
    let mut draw = uiworld.write::<ImmediateDraw>();

    if let Some(sel) = inspected.e {
        let mut pos = goria.pos(sel);

        if let Some(loc) = goria.comp::<Location>(sel) {
            match *loc {
                Location::Outside => {}
                Location::Vehicle(v) => pos = goria.pos(v.0),
                Location::Building(b) => pos = map.buildings().get(b).map(|b| b.door_pos),
            }
        }

        if let Some((pos, selectable)) = pos.zip(goria.comp::<Selectable>(sel)) {
            draw.stroke_circle(pos, selectable.radius, (selectable.radius * 0.01).max(0.1))
                .color(Color::gray(0.7));
        }
    }
}
