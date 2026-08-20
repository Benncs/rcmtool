// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::*;

///A tetra cut by a plane gives at most 4 vertices, and clipping a convex polygon by a half space
///adds at most one, so four bounds can never take it past 8
pub(crate) const MAX_POLYGON_VERTICES: usize = 12;

///Points kept by  clipper are the ones with `normal . point >= offset`
pub(crate) struct HalfSpace {
    pub(crate) normal: CartesianVec3,
    pub(crate) offset: f64,
}

impl HalfSpace {
    fn signed_distance(&self, point: &Coords3) -> f64 {
        self.normal.dot(&CartesianVec3(*point)) - self.offset
    }
}

///Convex polygon of the intersection, kept on the stack: this runs once per element per interface
pub(crate) struct Polygon {
    points: [Coords3; MAX_POLYGON_VERTICES],
    len: usize,
}

impl Polygon {
    pub(crate) fn from_slice(points: &[Coords3]) -> Self {
        let mut polygon = Self {
            points: [[0.; 3]; MAX_POLYGON_VERTICES],
            len: points.len().min(MAX_POLYGON_VERTICES),
        };
        polygon.points[..polygon.len].copy_from_slice(&points[..polygon.len]);
        polygon
    }

    pub(crate) fn as_slice(&self) -> &[Coords3] {
        &self.points[..self.len]
    }

    fn push(&mut self, point: Coords3) {
        if self.len < MAX_POLYGON_VERTICES {
            self.points[self.len] = point;
            self.len += 1;
        }
    }

    ///Sutherland-Hodgman clipping of the polygon by one half space, the polygon has to be convex
    ///and its vertices ordered. See https://en.wikipedia.org/wiki/Sutherland%E2%80%93Hodgman_algorithm
    ///
    ///        keep | drop            keep |
    ///     +-------|---+          +-------+
    ///     |       |  /           |      /
    ///     |  poly | /     =      |     /
    ///     |       |/             |    /
    ///     +-------+              +---+
    ///
    pub(crate) fn clip(&self, half_space: &HalfSpace) -> Self {
        let mut clipped = Self {
            points: [[0.; 3]; MAX_POLYGON_VERTICES],
            len: 0,
        };

        for i in 0..self.len {
            let current = self.points[i];
            let previous = self.points[(i + self.len - 1) % self.len];

            let d_current = half_space.signed_distance(&current);
            let d_previous = half_space.signed_distance(&previous);

            //The edge crosses the boundary, the crossing point belongs to the clipped polygon
            if (d_current >= 0.) != (d_previous >= 0.) {
                let t = d_previous / (d_previous - d_current);
                clipped.push([
                    previous[0] + t * (current[0] - previous[0]),
                    previous[1] + t * (current[1] - previous[1]),
                    previous[2] + t * (current[2] - previous[2]),
                ]);
            }

            if d_current >= 0. {
                clipped.push(current);
            }
        }

        clipped
    }
}
