use std::cmp::Ordering;

use crate::{BlockCoords, Direction};
use arrayvec::ArrayVec;
use mat::Transmuter;
use mat::VectorTrait;

const EPSILON: f32 = 0.0001;

/// An iterator over voxels crossed by a ray cast
#[derive(Debug, Clone)]
pub struct RayTravel {
    travelers: ArrayVec<RayTraveler<f32>, 3>,
    time: f32,
    limit: f32,
    origin: [f32; 3],
    ray: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
struct RayTraveler<T> {
    direction: Direction,
    next: T,
    step: T,
}

impl RayTravel {
    pub fn new(origin: [f32; 3], ray: [f32; 3], limit: f32) -> Self {
        Self {
            // For each axis (x, y and z) we define a traveler
            // But because voxel coord are centered on [west, down, north] of a block
            // and not the geometrical ceneter of mass of a cube, the positive and
            // negative cases have to be processed differently.
            travelers: (
                ray,
                origin,
                [
                    (Direction::East, Direction::West),
                    (Direction::Up, Direction::Down),
                    (Direction::South, Direction::North),
                ],
            )
                .transmute()
                .into_iter()
                .filter_map(|(ray, origin, direction)| match ray.partial_cmp(&0.0)? {
                    // Negative direction (west, down or north)
                    Ordering::Less => Some(RayTraveler {
                        direction: direction.0,
                        step: 1.0 / ray.abs(),
                        next: (origin - origin.floor()) / ray.abs(),
                    }),
                    // Positive direction (east, up or south)
                    Ordering::Greater => Some(RayTraveler {
                        direction: direction.1,
                        step: 1.0 / ray,
                        next: (origin.ceil() - origin) / ray,
                    }),
                    // Not moving on current axis, so ignore it
                    Ordering::Equal => None,
                })
                .collect(),
            limit,
            time: 0.0,
            ray,
            origin,
        }
    }
}

impl Iterator for RayTravel {
    type Item = Option<(BlockCoords, Direction)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.time > self.limit {
            // Stop if ray trace length (or time, as you prefer) exceeded
            return None;
        }

        // find the first traveler crossing integer axis value
        let traveler = self
            .travelers
            .iter_mut()
            .min_by(|lhs, rhs| lhs.next.partial_cmp(&rhs.next).unwrap_or(Ordering::Equal))?;

        // the ray have at least reach this time (or length, as you prefer)
        // we add an epsilon to be sure the position will be inside the desired voxel
        self.time = traveler.next + EPSILON;
        // update the traveler
        traveler.next += traveler.step;

        // compute in which voxel to position ends up
        if let Ok(position) = self
            .origin
            .vector_add(self.ray.vector_scale(self.time))
            .try_into()
        {
            Some(Some((position, traveler.direction)))
        } else {
            // out of the world
            Some(None)
        }
    }
}
