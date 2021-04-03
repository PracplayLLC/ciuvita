use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::uiworld::UiWorld;
use common::{AudioKind, Z_TOOL};
use egregoria::Egregoria;
use geom::{Vec2, OBB};
use map_model::{BuildingGen, BuildingKind, ProjectKind};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

register_resource_noserialize!(SpecialBuildingResource);
#[derive(Serialize, Deserialize)]
pub struct SpecialBuildingResource {
    pub opt: Option<(BuildingKind, BuildingGen, f32, String)>,
}

impl Default for SpecialBuildingResource {
    fn default() -> Self {
        Self { opt: None }
    }
}

pub fn specialbuilding(goria: &Egregoria, uiworld: &mut UiWorld) {
    let res = uiworld.read::<SpecialBuildingResource>();
    let tool = *uiworld.read::<Tool>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let mut sound = uiworld.write::<ImmediateSound>();

    let map = goria.map();

    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::SpecialBuilding) {
        return;
    }
    let (kind, gen, size, asset) = unwrap_or!(&res.opt, return);
    let size = *size;

    let mpos = mouseinfo.unprojected;
    let roads = map.roads();

    let closest_road = map
        .spatial_map()
        .query_around(mpos, size)
        .filter_map(|x| match x {
            ProjectKind::Road(id) => Some(&roads[id]),
            _ => None,
        })
        .min_by_key(move |p| OrderedFloat(p.points().project_dist2(mpos)));

    let mut draw_red = || {
        draw.textured_obb(OBB::new(mpos, Vec2::UNIT_Y, size, size), asset.to_owned())
            .color(common::config().special_building_invalid_col)
            .z(Z_TOOL);
    };

    let closest_road = unwrap_or!(closest_road, return draw_red());

    let (proj, _, dir) = closest_road.points().project_segment_dir(mpos);

    if !proj.is_close(mpos, size + closest_road.width * 0.5) {
        return draw_red();
    }

    let side = if (mpos - proj).dot(dir.perpendicular()) > 0.0 {
        dir.perpendicular()
    } else {
        -dir.perpendicular()
    };

    let first = closest_road.points().first();
    let last = closest_road.points().last();

    let obb = OBB::new(
        proj + side * (size + closest_road.width + 0.5) * 0.5,
        side,
        size,
        size,
    );

    if proj.distance(first) < 0.5 * size
        || proj.distance(last) < 0.5 * size
        || closest_road.sidewalks(closest_road.src).incoming.is_none()
    {
        draw.textured_obb(obb, asset.to_owned())
            .color(common::config().special_building_invalid_col)
            .z(Z_TOOL);
        return;
    }

    let rid = closest_road.id;

    if mouseinfo.just_pressed.contains(&MouseButton::Left) {
        commands.map_build_special_building(rid, obb, *kind, *gen);
        sound.play("road_lay", AudioKind::Ui);
    }

    draw.textured_obb(obb, asset.to_owned())
        .color(common::config().special_building_col)
        .z(Z_TOOL);
}
