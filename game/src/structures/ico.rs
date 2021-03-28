use std::{collections::BTreeMap, num::NonZeroU32};

use engine::graphics::helper::calc_normal;

use num_traits::FloatConst;
use rand::prelude::*;

use crate::pipelines::ico::IcoVertex;

#[derive(Debug)]
#[allow(dead_code)]
pub struct IcoFace {
    pub index: NonZeroU32,
    pub indices: [u16; 3],
    pub vertices: [glam::Vec3; 3],
    pub normal: glam::Vec3,
    pub siblings: [Option<NonZeroU32>; 3],
    pub tex_coords: [glam::Vec2; 3],
    pub tex_index: u32,
}

pub struct Ico {
    sub: u32,
    faces: Vec<IcoFace>,
}

impl Ico {
    #[rustfmt::skip]
    fn base_vertices() -> [[f32; 3]; 12] {
        let r = (1.0 + f32::sqrt(5.0)) / 2.0;
        let s = f32::sqrt(r + 2.0);
        let x = r / s;
        let z = 1.0 / s;
        [[-z, x, 0.0], [z, x, 0.0], [-z, -x, 0.0], [z, -x, 0.0],
         [0.0, -z, x], [0.0, z, x], [0.0, -z, -x], [0.0, z, -x],
         [x, 0.0, -z], [x, 0.0, z], [-x, 0.0, -z], [-x, 0.0, z]]
    }

    #[rustfmt::skip]
    fn base_faces() -> [[u16; 3]; 20] {
        [[0, 11, 5], [0, 5, 1],  [0, 1, 7],   [0, 7, 10], [0, 10, 11],
         [1, 5, 9],  [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
         [3, 9, 4],  [3, 4, 2],  [3, 2, 6],   [3, 6, 8],  [3, 8, 9],
         [4, 9, 5],  [2, 4, 11], [6, 2, 10],  [8, 6, 7],  [9, 8, 1]]
    }

    fn tex_from_radius(base: f32, offset: usize) -> [f32; 2] {
        let offset = f32::PI() * (2.0 / 3.0) * offset as f32;
        [
            f32::cos(base + offset) * -0.5 + 0.5,
            f32::sin(base + offset) * -0.5 + 0.5,
        ]
    }

    fn tex_coords(index: u16) -> [glam::Vec2; 3] {
        let base = index as f32 % (2.0 * f32::PI());
        [
            Self::tex_from_radius(base, 0).into(),
            Self::tex_from_radius(base, 1).into(),
            Self::tex_from_radius(base, 2).into(),
        ]
    }

    pub fn base() -> Self {
        let mut rng = rand::thread_rng();
        let mut edges = BTreeMap::new();
        let vs = Self::base_vertices();

        let mut faces: Vec<IcoFace> = Vec::new();
        for (index, [i0, i1, i2]) in Self::base_faces().iter().copied().enumerate() {
            let index = NonZeroU32::new(index as u32 + 1).expect("Index 0 not valid");
            edges.insert([i0, i1], index);
            edges.insert([i1, i2], index);
            edges.insert([i2, i0], index);

            let f0: glam::Vec3 = vs[i0 as usize].into();
            let f1: glam::Vec3 = vs[i1 as usize].into();
            let f2: glam::Vec3 = vs[i2 as usize].into();

            let u = f1 - f0;
            let v = f2 - f0;
            let normal = u.cross(v);

            let face = IcoFace {
                index,
                indices: [i0, i1, i2],
                vertices: [f0, f1, f2],
                normal,
                siblings: [None; 3],
                tex_coords: Self::tex_coords(i0 + (i1 * i2)),
                tex_index: rng.gen_range(0..=3),
            };

            faces.push(face);
        }

        for face in faces.iter_mut() {
            let [i0, i1, i2] = face.indices;
            face.siblings = [
                edges.get(&[i1, i0]).copied(),
                edges.get(&[i2, i1]).copied(),
                edges.get(&[i0, i2]).copied(),
            ];
        }

        Self { sub: 0, faces }
    }

    pub fn subdivide(&mut self) {
        let mut rng = rand::thread_rng();
        let mut vertices = Vec::new();
        let mut index_count = (12 * 2u16.pow(self.sub)) - 1;
        let mut indices = BTreeMap::new();
        let mut edges = BTreeMap::new();
        for f in self.faces.drain(..) {
            let IcoFace {
                index,
                indices: [i0, i1, i2],
                vertices: [f0, f1, f2],
                ..
            } = f;
            let n = index.get();
            let n0 = f0.lerp(f1, 0.5).normalize();
            let n1 = f1.lerp(f2, 0.5).normalize();
            let n2 = f2.lerp(f0, 0.5).normalize();

            let j0 = *indices
                .entry([u16::max(i0, i1), u16::min(i0, i1)])
                .or_insert_with(|| {
                    index_count += 1;
                    index_count
                });
            let j1 = *indices
                .entry([u16::max(i1, i2), u16::min(i1, i2)])
                .or_insert_with(|| {
                    index_count += 1;
                    index_count
                });
            let j2 = *indices
                .entry([u16::max(i2, i0), u16::min(i2, i0)])
                .or_insert_with(|| {
                    index_count += 1;
                    index_count
                });

            edges.insert((j2, i0), n * 4 - 3);
            edges.insert((i0, j0), n * 4 - 3);
            edges.insert((j0, j2), n * 4 - 3);

            edges.insert((j1, i2), n * 4 - 2);
            edges.insert((i2, j2), n * 4 - 2);
            edges.insert((j2, j1), n * 4 - 2);

            edges.insert((j0, i1), n * 4 - 1);
            edges.insert((i1, j1), n * 4 - 1);
            edges.insert((j1, j0), n * 4 - 1);

            edges.insert((j0, j1), n * 4);
            edges.insert((j1, j2), n * 4);
            edges.insert((j2, j0), n * 4);

            vertices.extend(std::array::IntoIter::new([
                IcoFace {
                    index: NonZeroU32::new(n * 4 - 3).unwrap(),
                    indices: [j2, i0, j0],
                    vertices: [n2, f0, n0],
                    normal: calc_normal(n2, f0, n0),
                    siblings: [None; 3],
                    tex_coords: Self::tex_coords(j2.wrapping_add(i0.wrapping_mul(j0))),
                    tex_index: rng.gen_range(0..=3),
                },
                IcoFace {
                    index: NonZeroU32::new(n * 4 - 2).unwrap(),
                    indices: [j1, i2, j2],
                    vertices: [n1, f2, n2],
                    normal: calc_normal(n1, f2, n2),
                    siblings: [None; 3],
                    tex_coords: Self::tex_coords(j1.wrapping_add(i2.wrapping_mul(j2))),
                    tex_index: rng.gen_range(0..=3),
                },
                IcoFace {
                    index: NonZeroU32::new(n * 4 - 1).unwrap(),
                    indices: [j0, i1, j1],
                    vertices: [n0, f1, n1],
                    normal: calc_normal(n0, f1, n1),
                    siblings: [None; 3],
                    tex_coords: Self::tex_coords(j0.wrapping_add(i1.wrapping_mul(j1))),
                    tex_index: rng.gen_range(0..=3),
                },
                IcoFace {
                    index: NonZeroU32::new(n * 4).unwrap(),
                    indices: [j0, j1, j2],
                    vertices: [n0, n1, n2],
                    normal: calc_normal(n0, n1, n2),
                    siblings: [None; 3],
                    tex_coords: Self::tex_coords(j0.wrapping_add(j1.wrapping_mul(j2))),
                    tex_index: rng.gen_range(0..=3),
                },
            ]));
        }

        for f in vertices.iter_mut() {
            let [i0, i1, i2] = f.indices;
            f.siblings = [
                edges.get(&(i1, i0)).map(|v| NonZeroU32::new(*v)).flatten(),
                edges.get(&(i2, i1)).map(|v| NonZeroU32::new(*v)).flatten(),
                edges.get(&(i0, i2)).map(|v| NonZeroU32::new(*v)).flatten(),
            ]
        }
        self.sub += 1;
        self.faces = vertices;
    }

    pub fn divs(divs: usize) -> Self {
        let mut ico = Self::base();
        for _ in 0..divs {
            ico.subdivide()
        }
        ico
    }

    pub fn vertex_data(&self) -> Vec<IcoVertex> {
        self.faces
            .iter()
            .flat_map(|f| {
                f.vertices
                    .iter()
                    .copied()
                    .zip(f.tex_coords.iter().copied())
                    .map(move |(v, t)| IcoVertex {
                        position: v.into(),
                        normal: f.normal.into(),
                        index: f.index.get(),
                        tex_coords: t.into(),
                        tex_idx: f.tex_index,
                    })
            })
            .collect()
    }

    pub fn face(&self, index: u32) -> Option<&IcoFace> {
        if index == 0 {
            return None;
        }
        let place = (index - 1) as usize;
        let face = self.faces.get(place);
        if let Some(face) = face {
            debug_assert!(
                face.index.get() == index,
                "{} != {}",
                face.index.get(),
                index
            );
        }
        face
    }
}
