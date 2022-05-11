use common::FastMap;
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::Egregoria;
use geom::{vec2, vec3, Color, LinearColor, PolyLine3, Polygon, Spline, Vec2, Vec3};
use map_model::{
    BuildingKind, Intersection, LaneKind, LotKind, Map, PylonPosition, Road, Roads, Terrain,
    TurnKind, CROSSWALK_WIDTH,
};
use std::ops::{Mul, Neg};
use std::sync::Arc;
use wgpu_engine::earcut::earcut;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::wgpu::RenderPass;
use wgpu_engine::{
    Drawable, GfxContext, InstancedMesh, InstancedMeshBuilder, Mesh, MeshBuilder, MeshInstance,
    MeshVertex, MultiSpriteBatch, SpriteBatch, SpriteBatchBuilder, Tesselator,
};

pub struct MapMeshHandler {
    builders: MapBuilders,
    cache: Option<Arc<MapMeshes>>,
    map_dirt_id: u32,
    last_config: usize,
}

struct MapBuilders {
    buildsprites: FastMap<BuildingKind, SpriteBatchBuilder>,
    buildmeshes: FastMap<BuildingKind, InstancedMeshBuilder>,
    trainstations: InstancedMeshBuilder,
    houses_mesh: MeshBuilder,
    arrow_builder: SpriteBatchBuilder,
    crosswalk_builder: MeshBuilder,
    tess_map: Tesselator,
}

pub struct MapMeshes {
    map: Option<Mesh>,
    crosswalks: Option<Mesh>,
    bsprites: MultiSpriteBatch,
    bmeshes: Vec<InstancedMesh>,
    trainstation: Option<InstancedMesh>,
    houses_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
}

impl MapMeshHandler {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/arrow_one_way.png");

        let mut buildsprites = FastMap::default();
        let mut buildmeshes = FastMap::default();

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let asset = descr.asset_location;
            if !asset.ends_with(".png") {
                continue;
            }
            buildsprites.insert(
                descr.bkind,
                SpriteBatchBuilder::new(gfx.texture(asset, asset)),
            );
        }

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let asset = descr.asset_location;
            if !asset.ends_with(".glb") {
                continue;
            }
            buildmeshes.insert(
                descr.bkind,
                InstancedMeshBuilder::new(unwrap_contlog!(
                    load_mesh(asset, gfx),
                    "couldn't load obj: {}",
                    asset
                )),
            );
        }

        let builders = MapBuilders {
            arrow_builder,
            buildsprites,
            crosswalk_builder: MeshBuilder::new(),
            tess_map: Tesselator::new(None, 15.0),
            houses_mesh: MeshBuilder::new(),
            buildmeshes,
            trainstations: InstancedMeshBuilder::new(
                load_mesh("assets/models/trainstation.glb", gfx).unwrap(),
            ),
        };

        Self {
            builders,
            cache: None,
            map_dirt_id: 0,
            last_config: common::config_id(),
        }
    }

    pub fn latest_mesh(&mut self, map: &Map, gfx: &mut GfxContext) -> &Option<Arc<MapMeshes>> {
        if map.dirt_id.0 != self.map_dirt_id || self.last_config != common::config_id() {
            self.builders.map_mesh(map);
            self.builders.arrows(map);
            self.builders.crosswalks(map);
            self.builders.bspritesmesh(map);
            self.builders.trainstation(map);
            self.builders.houses_mesh(map);

            self.last_config = common::config_id();
            self.map_dirt_id = map.dirt_id.0;

            let m = &mut self.builders.tess_map.meshbuilder;

            let cw = gfx.texture("assets/crosswalk.png", "crosswalk");

            let meshes = MapMeshes {
                map: m.build(gfx, gfx.palette()),
                crosswalks: self.builders.crosswalk_builder.build(gfx, cw),
                bsprites: self
                    .builders
                    .buildsprites
                    .values_mut()
                    .flat_map(|x| x.build(gfx))
                    .collect(),
                bmeshes: self
                    .builders
                    .buildmeshes
                    .values_mut()
                    .flat_map(|x| x.build(gfx))
                    .collect(),
                trainstation: self.builders.trainstations.build(gfx),
                houses_mesh: self.builders.houses_mesh.build(gfx, gfx.palette()),
                arrows: self.builders.arrow_builder.build(gfx),
            };

            self.cache = Some(Arc::new(meshes));
        }
        &self.cache
    }
}

impl MapBuilders {
    fn arrows(&mut self, map: &Map) {
        self.arrow_builder.clear();
        let lanes = map.lanes();
        let roads = map.roads();
        for road in roads.values() {
            let fade = (road.length()
                - 5.0
                - road.interface_from(road.src)
                - road.interface_from(road.dst))
            .mul(0.2)
            .clamp(0.0, 1.0);

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.vehicles());
            let n_arrows = ((road.length() / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.points.length();
                for i in 0..n_arrows {
                    let (mid, dir) = lane
                        .points
                        .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                    self.arrow_builder.push(
                        mid.up(0.03),
                        dir,
                        LinearColor::gray(0.3 + fade * 0.1),
                        (2.0, 2.0),
                    );
                }
            }
        }
    }

    fn crosswalks(&mut self, map: &Map) {
        let builder = &mut self.crosswalk_builder;
        builder.clear();

        let walking_w: f32 = LaneKind::Walking.width();

        let lanes = map.lanes();
        let intersections = map.intersections();
        for (inter_id, inter) in intersections {
            for turn in inter.turns() {
                let id = turn.id;

                if matches!(turn.kind, TurnKind::Crosswalk) {
                    let from = lanes[id.src].get_inter_node_pos(inter_id).up(0.01);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id).up(0.01);

                    let l = (to - from).magnitude();

                    if l < walking_w {
                        continue;
                    }

                    let dir = (to - from) / l;
                    let perp = dir.perp_up() * CROSSWALK_WIDTH * 0.5;
                    let pos = from + dir * walking_w * 0.5;
                    let height = l - walking_w;

                    builder.extend_with(|vertices, add_index| {
                        let mk_v = |position: Vec3, uv: Vec2| MeshVertex {
                            position: position.into(),
                            uv: uv.into(),
                            normal: Vec3::Z,
                            color: [1.0; 4],
                        };

                        vertices.push(mk_v(pos - perp, Vec2::ZERO));
                        vertices.push(mk_v(pos + perp, Vec2::ZERO));
                        vertices.push(mk_v(pos + perp + dir * height, Vec2::x(height)));
                        vertices.push(mk_v(pos - perp + dir * height, Vec2::x(height)));

                        add_index(0);
                        add_index(1);
                        add_index(2);

                        add_index(0);
                        add_index(2);
                        add_index(3);
                    });
                }
            }
        }
    }

    fn trainstation(&mut self, map: &Map) {
        self.trainstations.instances.clear();

        let inter = map.intersections();
        let roads = map.roads();

        for station in map.trainstations().values() {
            let lefti = inter[station.left].pos;
            let righti = inter[station.right].pos;
            let rw = roads[station.track].width;

            let center = (lefti + righti) * 0.5;
            let dir = (righti - lefti).normalize();

            self.trainstations.instances.push(MeshInstance {
                pos: center - dir.perp_up() * (rw * 0.5 + 10.5),
                dir,
                tint: LinearColor::WHITE,
            });
        }
    }

    fn bspritesmesh(&mut self, map: &Map) {
        for v in self.buildsprites.values_mut() {
            v.clear();
        }

        for v in self.buildmeshes.values_mut() {
            v.instances.clear();
        }

        let buildings = &map.buildings();

        for building in buildings.values() {
            if let Some(x) = self.buildsprites.get_mut(&building.kind) {
                let axis = building.obb.axis();
                let c = building.obb.center();
                let w = axis[0].magnitude();
                let d = axis[0] / w;
                let h = axis[1].magnitude();
                x.push(
                    c.z(building.height + 0.1),
                    d.z0(),
                    LinearColor::WHITE,
                    (w, h),
                );
            }

            if let Some(x) = self.buildmeshes.get_mut(&building.kind) {
                let pos = building.obb.center().z(building.height);
                let dir = building.obb.axis()[0].normalize().z0();

                x.instances.push(MeshInstance {
                    pos,
                    dir,
                    tint: LinearColor::WHITE,
                });
            }
        }
    }

    fn houses_mesh(&mut self, map: &Map) {
        self.houses_mesh.clear();

        let buildings = &map.buildings();

        let mut projected = Polygon(Vec::with_capacity(10));

        for building in buildings.values() {
            for (face, col) in &building.mesh.faces {
                self.houses_mesh.extend_with(|vertices, add_index| {
                    let o = face[1];
                    let u = unwrap_ret!((face[0] - o).try_normalize());
                    let v = unwrap_ret!((face[2] - o).try_normalize());

                    let mut nor = u.cross(v);

                    let mut reverse = false;

                    if nor.z < 0.0 {
                        reverse = true;
                        nor = -nor;
                    }

                    projected.clear();
                    for &p in face {
                        let off = p - o;
                        projected.0.push(vec2(off.dot(u), off.dot(v)));

                        vertices.push(MeshVertex {
                            position: p.into(),
                            normal: nor,
                            uv: [0.0; 2],
                            color: col.into(),
                        })
                    }

                    projected.simplify();

                    earcut(&projected.0, |mut a, b, mut c| {
                        if reverse {
                            std::mem::swap(&mut a, &mut c);
                        }
                        add_index(a as u32);
                        add_index(b as u32);
                        add_index(c as u32);
                    })
                });
            }
        }
    }

    fn draw_rail(tess: &mut Tesselator, cut: &PolyLine3, off: f32, limits: bool) {
        tess.set_color(Color::gray(0.5));
        tess.draw_polyline_full(
            cut.as_slice().iter().map(|v| vec3(v.x, v.y, v.z + 0.02)),
            unwrap_ret!(cut.first_dir()).xy(),
            unwrap_ret!(cut.last_dir()).xy(),
            0.1,
            off + 0.6,
        );
        tess.draw_polyline_full(
            cut.as_slice().iter().map(|v| vec3(v.x, v.y, v.z + 0.02)),
            unwrap_ret!(cut.first_dir()).xy(),
            unwrap_ret!(cut.last_dir()).xy(),
            0.1,
            off - 0.6,
        );
        for (v, dir) in cut.equipoints_dir(1.0, !limits) {
            let up = vec3(v.x, v.y, v.z + 0.04);
            tess.draw_polyline_full(
                std::array::IntoIter::new([up, up + dir * 0.1]),
                dir.xy(),
                dir.xy(),
                2.0,
                off,
            );
        }
    }

    fn map_mesh(&mut self, map: &Map) {
        let tess = &mut self.tess_map;
        tess.meshbuilder.clear();

        let low_col: LinearColor = common::config().road_low_col.into();
        let mid_col: LinearColor = common::config().road_mid_col.into();
        let hig_col: LinearColor = common::config().road_hig_col.into();
        let line_col: LinearColor = common::config().road_line_col.into();

        let inters = map.intersections();
        let lanes = map.lanes();
        let roads = map.roads();
        let lots = map.lots();
        let terrain = &map.terrain;

        for road in roads.values() {
            let cut = road.interfaced_points();

            road_pylons(&mut tess.meshbuilder, terrain, road);

            tess.normal.z = -1.0;
            tess.draw_polyline_full(
                cut.iter().map(|x| x.up(-0.3)),
                cut.first_dir().unwrap_or_default().xy(),
                cut.last_dir().unwrap_or_default().xy(),
                road.width,
                0.0,
            );
            tess.normal.z = 1.0;

            let draw_off = |tess: &mut Tesselator, col: LinearColor, w, off| {
                tess.set_color(col);
                tess.draw_polyline_full(
                    cut.as_slice().iter().copied(),
                    unwrap_ret!(cut.first_dir()).xy(),
                    unwrap_ret!(cut.last_dir()).xy(),
                    w,
                    off,
                );
            };

            let mut start = true;
            for l in road.lanes_iter().flat_map(|(l, _)| lanes.get(l)) {
                if l.kind.is_rail() {
                    let off = l.dist_from_bottom - road.width * 0.5 + LaneKind::Rail.width() * 0.5;
                    draw_off(tess, mid_col, LaneKind::Rail.width(), off);
                    Self::draw_rail(tess, cut, off, true);
                    start = true;
                    continue;
                }
                if start {
                    draw_off(tess, line_col, 0.25, l.dist_from_bottom - road.width * 0.5);
                    start = false;
                }
                draw_off(
                    tess,
                    match l.kind {
                        LaneKind::Walking => hig_col,
                        LaneKind::Parking => low_col,
                        _ => mid_col,
                    },
                    l.kind.width() - 0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width() * 0.5,
                );
                draw_off(
                    tess,
                    line_col,
                    0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width(),
                );
            }
        }

        // Intersections
        let mut p = Vec::with_capacity(8);
        let mut ppoly = unsafe { PolyLine3::new_unchecked(vec![]) };
        for inter in inters.values() {
            if inter.roads.is_empty() {
                tess.set_color(line_col);
                tess.draw_circle(inter.pos, 5.5);

                tess.set_color(mid_col);
                tess.draw_circle(inter.pos, 5.0);
                continue;
            }

            inter_pylon(&mut tess.meshbuilder, terrain, inter, roads);
            intersection_mesh(&mut tess.meshbuilder, inter, roads);

            // Walking corners
            for turn in inter
                .turns()
                .iter()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess.set_color(line_col);
                let id = turn.id;

                let w = lanes[id.src].kind.width();

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, w * 0.5);
                tess.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, -w * 0.5);

                tess.set_color(hig_col);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, w - 0.25);
            }

            // Rail turns
            for turn in inter
                .turns()
                .iter()
                .filter(|turn| matches!(turn.kind, TurnKind::Rail))
            {
                ppoly.clear_extend(turn.points.as_slice());
                Self::draw_rail(tess, &ppoly, 0.0, false);
            }
        }

        // Lots
        for lot in lots.values() {
            let col = match lot.kind {
                LotKind::Unassigned => common::config().lot_unassigned_col,
                LotKind::Residential => common::config().lot_residential_col,
            };
            tess.set_color(col);
            tess.draw_filled_polygon(&lot.shape.corners, lot.height + 0.3);
        }
    }
}

impl Drawable for MapMeshes {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(ref v) = self.map {
            v.draw(gfx, rp);
        }
        self.bsprites.draw(gfx, rp);
        for v in &self.bmeshes {
            v.draw(gfx, rp);
        }
        if let Some(ref v) = self.houses_mesh {
            v.draw(gfx, rp);
        }
        if let Some(ref v) = self.arrows {
            v.draw(gfx, rp);
        }
        if let Some(ref v) = self.crosswalks {
            v.draw(gfx, rp);
        }
        if let Some(ref v) = self.trainstation {
            v.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu_engine::wgpu::BindGroup,
    ) {
        macro_rules! deferdepth {
            ($x:expr) => {
                $x.draw_depth(gfx, rp, shadow_map, proj);
            };
        }
        if let Some(ref map) = self.map {
            map.draw_depth(gfx, rp, shadow_map, proj);
        }
        deferdepth!(self.bsprites);
        for v in &self.bmeshes {
            deferdepth!(v);
        }
        if let Some(ref v) = self.houses_mesh {
            deferdepth!(v);
        }
        if let Some(ref v) = self.arrows {
            deferdepth!(v);
        }
        if let Some(ref v) = self.crosswalks {
            deferdepth!(v);
        }
        if let Some(ref v) = self.trainstation {
            deferdepth!(v);
        }
    }
}

fn add_polyon(
    mut meshb: &mut MeshBuilder,
    w: f32,
    PylonPosition {
        terrain_height,
        pos,
        dir,
    }: PylonPosition,
) {
    let color = LinearColor::from(common::config().road_pylon_col);
    let color: [f32; 4] = color.into();

    let up = pos.up(-0.2);
    let down = pos.xy().z(terrain_height);
    let dirp = dir.perp_up();
    let d2 = dir.xy().z0();
    let d2p = d2.perp_up();
    let d2 = d2 * w * 0.5;
    let d2p = d2p * w * 0.5;
    let dir = dir * w * 0.5;
    let dirp = dirp * w * 0.5;
    // down rect
    // 2 --- 1 -> dir
    // |     |
    // |     |
    // 3-----0
    // | dirp
    // v

    // up rect
    // 6 --- 5
    // |     |
    // |     |
    // 7-----4
    let verts = [
        down + d2 + d2p, // 0
        down + d2 - d2p, // 1
        down - d2 - d2p, // 2
        down - d2 + d2p, // 3
        up + dir + dirp, // 4
        up + dir - dirp, // 5
        up - dir - dirp, // 6
        up - dir + dirp, // 7
    ];

    let mr = &mut meshb;
    let mut quad = move |a, b, c, d, nor| {
        mr.extend_with(move |vertices, add_idx| {
            let mut pvert = move |p: Vec3, normal: Vec3| {
                vertices.push(MeshVertex {
                    position: p.into(),
                    normal,
                    uv: [0.0; 2],
                    color,
                })
            };

            pvert(verts[a], nor);
            pvert(verts[b], nor);
            pvert(verts[c], nor);
            pvert(verts[d], nor);

            add_idx(0);
            add_idx(1);
            add_idx(2);

            add_idx(1);
            add_idx(3);
            add_idx(2);
        });
    };
    quad(0, 1, 4, 5, d2);
    quad(1, 2, 5, 6, -d2p);
    quad(2, 3, 6, 7, -d2);
    quad(3, 0, 7, 4, d2p);
}

fn road_pylons(meshb: &mut MeshBuilder, terrain: &Terrain, road: &Road) {
    for pylon in Road::pylons_positions(road.interfaced_points(), terrain) {
        add_polyon(meshb, road.width * 0.5, pylon);
    }
}

fn inter_pylon(meshb: &mut MeshBuilder, terrain: &Terrain, inter: &Intersection, roads: &Roads) {
    let h = unwrap_ret!(terrain.height(inter.pos.xy()));
    if (h - inter.pos.z).abs() <= 2.0 {
        return;
    }

    let mut maxw = 3.0f32;
    let mut avgp = Vec3::ZERO;

    for &road in &inter.roads {
        let r = &roads[road];
        maxw = maxw.max(r.width * 0.5);
        avgp += r.interface_point(inter.id);
    }
    if !inter.roads.is_empty() {
        avgp /= inter.roads.len() as f32;
    } else {
        avgp = inter.pos;
    }

    add_polyon(
        meshb,
        maxw,
        PylonPosition {
            terrain_height: h,
            pos: avgp,
            dir: Vec3::X,
        },
    );
}

fn intersection_mesh(meshb: &mut MeshBuilder, inter: &Intersection, roads: &Roads) {
    let id = inter.id;

    let getw = |road: &Road| {
        if road.sidewalks(id).outgoing.is_some() {
            road.width * 0.5 - LaneKind::Walking.width()
        } else {
            road.width * 0.5
        }
    };

    let mut polygon = Polygon::default();

    for (i, &road) in inter.roads.iter().enumerate() {
        #[allow(clippy::indexing_slicing)]
        let road = &roads[road];

        #[allow(clippy::indexing_slicing)]
        let next_road = &roads[inter.roads[(i + 1) % inter.roads.len()]];

        let ip = road.interfaced_points();

        let firstp;
        let firstdir;
        if road.dst == inter.id {
            firstp = ip.last();
            firstdir = ip.last_dir().map(Vec3::neg);
        } else {
            firstp = ip.first();
            firstdir = ip.first_dir();
        }

        let src_orient = unwrap_cont!(firstdir).xy();

        let left = firstp.xy() - src_orient.perpendicular() * getw(road);

        let ip = next_road.interfaced_points();

        let firstp;
        let firstdir;
        if next_road.dst == inter.id {
            firstp = ip.last();
            firstdir = ip.last_dir().map(Vec3::neg);
        } else {
            firstp = ip.first();
            firstdir = ip.first_dir();
        }

        let dst_orient = unwrap_cont!(firstdir).xy();
        let next_right = firstp.xy() + dst_orient.perpendicular() * getw(next_road);

        let ang = (-src_orient).angle(dst_orient);

        const TURN_ANG_ADD: f32 = 0.29;
        const TURN_ANG_MUL: f32 = 0.36;
        const TURN_MUL: f32 = 0.46;

        let dist =
            (next_right - left).magnitude() * (TURN_ANG_ADD + ang.abs() * TURN_ANG_MUL) * TURN_MUL;

        let spline = Spline {
            from: left,
            to: next_right,
            from_derivative: -src_orient * dist,
            to_derivative: dst_orient * dist,
        };

        polygon.extend(spline.smart_points(1.0, 0.0, 1.0));
    }

    polygon.simplify();

    let col = LinearColor::from(common::config().road_mid_col).into();
    meshb.extend_with(|vertices, add_idx| {
        vertices.extend(polygon.iter().map(|pos| MeshVertex {
            position: pos.z(inter.pos.z - 0.001).into(),
            normal: Vec3::Z,
            uv: [0.0; 2],
            color: col,
        }));
        earcut(&polygon.0, |a, b, c| {
            add_idx(a as u32);
            add_idx(b as u32);
            add_idx(c as u32);
            add_idx(c as u32);
            add_idx(b as u32);
            add_idx(a as u32);
        });
    });
}
