use crate::aabb::AABB;
use crate::circle::Circle;
use crate::segment::Segment;
use crate::{vec2, Intersect, Shape, Vec2, OBB};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::Index;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Polygon(pub Vec<Vec2>);

impl Polygon {
    pub fn rect(w: f32, h: f32) -> Self {
        Self(vec![Vec2::ZERO, vec2(w, 0.0), vec2(w, h), vec2(0.0, h)])
    }

    pub fn centered_rect(pos: Vec2, w: f32, h: f32) -> Self {
        Self(vec![
            pos + vec2(-w * 0.5, -h * 0.5),
            pos + vec2(w * 0.5, -h * 0.5),
            pos + vec2(w * 0.5, h * 0.5),
            pos + vec2(-w * 0.5, h * 0.5),
        ])
    }

    pub fn translate(&mut self, p: Vec2) -> &mut Self {
        for x in self.0.iter_mut() {
            *x += p
        }
        self
    }

    pub fn rotate(&mut self, cossin: Vec2) -> &mut Self {
        for x in self.0.iter_mut() {
            *x = x.rotated_by(cossin)
        }
        self
    }

    pub fn segment(&self, seg: usize) -> Segment {
        Segment::new(
            self.0[seg],
            self.0[if seg + 1 == self.0.len() { 0 } else { seg + 1 }],
        )
    }

    pub fn split_segment(&mut self, seg: usize, coeff: f32) -> &mut Self {
        let Segment { src, dst } = self.segment(seg);
        self.0.insert(seg + 1, src + (dst - src) * coeff);
        self
    }

    pub fn extrude(&mut self, seg: usize, dist: f32) -> &mut Self {
        assert!(dist.abs() > 0.0);

        let Segment { src, dst } = self.segment(seg);
        let perp = match (dst - src).perpendicular().try_normalize() {
            Some(x) => x,
            None => return self,
        };

        self.0.insert(seg + 1, src + perp * dist);
        self.0.insert(seg + 2, dst + perp * dist);
        self
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let nvert = self.0.len();

        let mut j = nvert - 1;
        let mut c = false;

        for i in 0..nvert {
            let verti = self.0[i];
            let vertj = self.0[j];
            let off = vertj - verti;

            let vip = p - verti;
            let vjp = p - vertj;

            if ((vip.y < 0.0) != (vjp.y < 0.0))
                && (vip.x * off.y.abs() < off.x * vip.y * off.y.signum())
            {
                c = !c;
            }
            j = i;
        }
        c
    }

    pub fn project(&self, pos: Vec2) -> Vec2 {
        self.project_segment(pos).0
    }

    pub fn simplify(&mut self) {
        let mut to_remove = vec![];
        for i in (0..self.len()).rev() {
            let prev = self.get_prev(i);
            let cur = self.get(i);
            let next = self.get_next(i);

            if prev.approx_eq(*cur) || (cur - prev).normalize().approx_eq((next - cur).normalize())
            {
                to_remove.push(i);
            }
        }

        for v in to_remove {
            self.0.remove(v);
        }
    }

    pub fn get_prev(&self, i: usize) -> &Vec2 {
        if i == 0 {
            self.get(self.len() - 1)
        } else {
            self.get(i - 1)
        }
    }

    pub fn get_next(&self, i: usize) -> &Vec2 {
        if i == self.len() - 1 {
            self.get(0)
        } else {
            self.get(i + 1)
        }
    }

    pub fn get(&self, i: usize) -> &Vec2 {
        self.0.get(i).unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn first(&self) -> Vec2 {
        *self.0.first().unwrap()
    }

    pub fn last(&self) -> Vec2 {
        *self.0.last().unwrap()
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment> + '_ {
        self.0
            .windows(2)
            .map(|w| Segment::new(w[0], w[1]))
            .chain(Some(Segment::new(self.last(), self.first())))
    }

    pub fn project_segment(&self, p: Vec2) -> (Vec2, usize) {
        match self.0.len() {
            0 => unreachable!(),
            1 => (self.first(), 0),
            2 => (
                Segment {
                    src: self.0[0],
                    dst: self.0[1],
                }
                .project(p),
                1,
            ),
            _ => {
                let l = [self.last(), self.first()];
                self.0
                    .windows(2)
                    .chain(std::iter::once(l.as_ref()))
                    .enumerate()
                    .map(|(i, w)| {
                        if let [a, b] = *w {
                            (Segment { src: a, dst: b }.project(p), i + 1)
                        } else {
                            unsafe { unreachable_unchecked() } // windows(2)
                        }
                    })
                    .min_by_key(|&(proj, _)| OrderedFloat((p - proj).magnitude2()))
                    .unwrap()
            } // Unwrap ok: n_points > 2
        }
    }

    pub fn barycenter(&self) -> Vec2 {
        self.0.iter().sum::<Vec2>() / (self.0.len() as f32)
    }

    pub fn bcircle(&self) -> Circle {
        let center = self.barycenter();
        let radius = self
            .0
            .iter()
            .map(move |x| OrderedFloat(x.distance2(center)))
            .max()
            .unwrap()
            .0
            .sqrt();
        Circle { center, radius }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Vec2> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Vec2> {
        self.0.iter_mut()
    }

    pub fn as_slice(&self) -> &[Vec2] {
        self.0.as_slice()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn extend<A, T>(&mut self, s: T)
    where
        T: IntoIterator<Item = A>,
        Vec<Vec2>: Extend<A>,
    {
        self.0.extend(s);
    }
}

impl Index<usize> for Polygon {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Vec2 {
        &self.0[index]
    }
}

impl From<Vec<Vec2>> for Polygon {
    fn from(v: Vec<Vec2>) -> Self {
        Self(v)
    }
}

impl Shape for Polygon {
    fn bbox(&self) -> AABB {
        let (min, max) = match super::minmax(self.0.iter().copied()) {
            Some(x) => x,
            None => return AABB::zero(),
        };

        AABB::new(min, max)
    }
}

impl Intersect<AABB> for Polygon {
    fn intersects(&self, shape: &AABB) -> bool {
        self.segments().any(|s| s.intersects(shape)) || self.contains(shape.ll)
    }
}

impl Intersect<Vec2> for Polygon {
    fn intersects(&self, &shape: &Vec2) -> bool {
        self.contains(shape)
    }
}

impl Intersect<Circle> for Polygon {
    fn intersects(&self, shape: &Circle) -> bool {
        self.segments().any(|s| s.intersects(shape)) || self.contains(shape.center)
    }
}

impl Intersect<Polygon> for Polygon {
    fn intersects(&self, other: &Polygon) -> bool {
        let mybbox = self.bbox();
        let his_bbox = other.bbox();

        mybbox.intersects(&his_bbox)
            && (self
                .0
                .iter()
                .any(|&point| his_bbox.contains(point) && other.contains(point))
                || other
                    .0
                    .iter()
                    .any(|&point| mybbox.contains(point) && self.contains(point)))
    }
}

impl Intersect<OBB> for Polygon {
    fn intersects(&self, shape: &OBB) -> bool {
        self.segments().any(|s| shape.intersects(&s)) || self.contains(shape.corners[0])
    }
}
