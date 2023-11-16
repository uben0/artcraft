use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

use crate::{Affine, AffineTrait, MatrixTrait, Transmuter, VectorCrossTrait, VectorTrait};

impl<T, const M: usize, const N: usize> MatrixTrait<T, M, N> for [[T; M]; N] {
    fn matrix_map<U, F: FnMut(T) -> U>(self, mut f: F) -> [[U; M]; N] {
        self.map(|col| col.map(|elem| f(elem)))
    }

    fn matrix_map_index<U, F: FnMut(T, usize, usize) -> U>(self, mut f: F) -> [[U; M]; N] {
        let mut n = 0..N;
        self.map(|col| {
            let mut m = 0..M;
            let n = n.next().unwrap();
            col.map(|elem| f(elem, m.next().unwrap(), n))
        })
    }

    fn matrix_transpose(self) -> [[T; N]; M] {
        let mut option = self.matrix_map(Some);
        [[(); N]; M].matrix_map_index(|(), n, m| option[n][m].take().unwrap())
    }

    fn matrix_add(self, rhs: [[T; M]; N]) -> [[T; M]; N]
    where
        T: Add<T, Output = T>,
    {
        let mut rhs = rhs.into_iter();
        self.map(|lhs| {
            let mut rhs = rhs.next().unwrap().into_iter();
            lhs.map(|lhs| lhs + rhs.next().unwrap())
        })
    }

    fn matrix_sub(self, rhs: [[T; M]; N]) -> [[T; M]; N]
    where
        T: Sub<T, Output = T>,
    {
        let mut rhs = rhs.into_iter();
        self.map(|lhs| {
            let mut rhs = rhs.next().unwrap().into_iter();
            lhs.map(|lhs| lhs - rhs.next().unwrap())
        })
    }

    fn matrix_mul<const O: usize>(self, rhs: [[T; N]; O]) -> [[T; M]; O]
    where
        T: Mul<T, Output = T>,
        T: Sum,
        T: Copy,
    {
        [[(); M]; O].matrix_map_index(|_, m, o| (0..N).map(|n| self[n][m] * rhs[o][n]).sum())
    }

    fn matrix_scale(self, scalar: T) -> [[T; M]; N]
    where
        T: Mul<T, Output = T>,
        T: Copy,
    {
        self.matrix_map(|v| v * scalar)
    }
}

impl<T> Affine<T, 4> {
    /// Creates the identity matrix.
    pub fn identity() -> [[T; 4]; 4]
    where
        T: Sum,
        T: Product,
        T: Copy,
    {
        let zero = std::iter::empty().sum();
        let one = std::iter::empty().product();
        [
            [one, zero, zero, zero],
            [zero, one, zero, zero],
            [zero, zero, one, zero],
            [zero, zero, zero, one],
        ]
    }

    /// Creates a matrix that performs a translation.
    pub fn translate([x, y, z]: [T; 3]) -> [[T; 4]; 4]
    where
        T: Sum,
        T: Product,
        T: Copy,
    {
        let zero = std::iter::empty().sum();
        let one = std::iter::empty().product();
        [
            [one, zero, zero, zero],
            [zero, one, zero, zero],
            [zero, zero, one, zero],
            [x, y, z, one],
        ]
    }

    /// Creates a matrix that performs a scaling.
    pub fn scale(scalar: T) -> [[T; 4]; 4]
    where
        T: Sum,
        T: Product,
        T: Copy,
    {
        let zero = std::iter::empty().sum();
        let one = std::iter::empty().product();
        [
            [scalar, zero, zero, zero],
            [zero, scalar, zero, zero],
            [zero, zero, scalar, zero],
            [zero, zero, zero, one],
        ]
    }
}

impl Affine<f32, 4> {
    /// Creates a matrix that performs a rotation along the x axis.
    pub fn x_rotate(radian: f32) -> [[f32; 4]; 4] {
        let y_y = radian.cos();
        let z_z = radian.cos();
        let y_z = -radian.sin();
        let z_y = radian.sin();
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, y_y, y_z, 0.0],
            [0.0, z_y, z_z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    /// Creates a matrix that performs a rotation along the y axis.
    pub fn y_rotate(radian: f32) -> [[f32; 4]; 4] {
        let x_x = radian.cos();
        let z_z = radian.cos();
        let x_z = radian.sin();
        let z_x = -radian.sin();
        [
            [x_x, 0.0, x_z, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [z_x, 0.0, z_z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    /// Creates a matrix that performs a rotation along the z axis.
    pub fn z_rotate(radian: f32) -> [[f32; 4]; 4] {
        let x_x = radian.cos();
        let y_y = radian.cos();
        let x_y = -radian.sin();
        let y_x = radian.sin();
        [
            [x_x, x_y, 0.0, 0.0],
            [y_x, y_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
impl Affine<f64, 4> {
    /// Creates a matrix that performs a rotation along the x axis.
    pub fn x_rotate(radian: f64) -> [[f64; 4]; 4] {
        let y_y = radian.cos();
        let z_z = radian.cos();
        let y_z = -radian.sin();
        let z_y = radian.sin();
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, y_y, y_z, 0.0],
            [0.0, z_y, z_z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    /// Creates a matrix that performs a rotation along the y axis.
    pub fn y_rotate(radian: f64) -> [[f64; 4]; 4] {
        let x_x = radian.cos();
        let z_z = radian.cos();
        let x_z = radian.sin();
        let z_x = -radian.sin();
        [
            [x_x, 0.0, x_z, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [z_x, 0.0, z_z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    /// Creates a matrix that performs a rotation along the z axis.
    pub fn z_rotate(radian: f64) -> [[f64; 4]; 4] {
        let x_x = radian.cos();
        let y_y = radian.cos();
        let x_y = -radian.sin();
        let y_x = radian.sin();
        [
            [x_x, x_y, 0.0, 0.0],
            [y_x, y_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

impl AffineTrait<f32, 3, 4> for [[f32; 4]; 4] {
    fn affine_translate(self, vector: [f32; 3]) -> [[f32; 4]; 4] {
        self.matrix_mul(Affine::<f32, 4>::translate(vector))
    }
    fn affine_scale(self, scalar: f32) -> [[f32; 4]; 4] {
        self.matrix_mul(Affine::<f32, 4>::scale(scalar))
    }
    fn affine_x_rotate(self, radian: f32) -> [[f32; 4]; 4] {
        self.matrix_mul(Affine::<f32, 4>::x_rotate(radian))
    }
    fn affine_y_rotate(self, radian: f32) -> [[f32; 4]; 4] {
        self.matrix_mul(Affine::<f32, 4>::y_rotate(radian))
    }
    fn affine_z_rotate(self, radian: f32) -> [[f32; 4]; 4] {
        self.matrix_mul(Affine::<f32, 4>::z_rotate(radian))
    }
}

impl AffineTrait<f64, 3, 4> for [[f64; 4]; 4] {
    fn affine_translate(self, vector: [f64; 3]) -> [[f64; 4]; 4] {
        self.matrix_mul(Affine::<f64, 4>::translate(vector))
    }
    fn affine_scale(self, scalar: f64) -> [[f64; 4]; 4] {
        self.matrix_mul(Affine::<f64, 4>::scale(scalar))
    }
    fn affine_x_rotate(self, radian: f64) -> [[f64; 4]; 4] {
        self.matrix_mul(Affine::<f64, 4>::x_rotate(radian))
    }
    fn affine_y_rotate(self, radian: f64) -> [[f64; 4]; 4] {
        self.matrix_mul(Affine::<f64, 4>::y_rotate(radian))
    }
    fn affine_z_rotate(self, radian: f64) -> [[f64; 4]; 4] {
        self.matrix_mul(Affine::<f64, 4>::z_rotate(radian))
    }
}

impl Affine<f32, 3> {
    /// Creates a matrix that performs a rotation along the x axis.
    pub fn x_rotate(radian: f32) -> [[f32; 3]; 3] {
        let y_y = radian.cos();
        let z_z = radian.cos();
        let y_z = -radian.sin();
        let z_y = radian.sin();
        [[1.0, 0.0, 0.0], [0.0, y_y, y_z], [0.0, z_y, z_z]]
    }
    /// Creates a matrix that performs a rotation along the y axis.
    pub fn y_rotate(radian: f32) -> [[f32; 3]; 3] {
        let x_x = radian.cos();
        let z_z = radian.cos();
        let x_z = radian.sin();
        let z_x = -radian.sin();
        [[x_x, 0.0, x_z], [0.0, 1.0, 0.0], [z_x, 0.0, z_z]]
    }
    /// Creates a matrix that performs a rotation along the z axis.
    pub fn z_rotate(radian: f32) -> [[f32; 3]; 3] {
        let x_x = radian.cos();
        let y_y = radian.cos();
        let x_y = -radian.sin();
        let y_x = radian.sin();
        [[x_x, x_y, 0.0], [y_x, y_y, 0.0], [0.0, 0.0, 1.0]]
    }
}
impl Affine<f64, 3> {
    /// Creates a matrix that performs a rotation along the x axis.
    pub fn x_rotate(radian: f64) -> [[f64; 3]; 3] {
        let y_y = radian.cos();
        let z_z = radian.cos();
        let y_z = -radian.sin();
        let z_y = radian.sin();
        [[1.0, 0.0, 0.0], [0.0, y_y, y_z], [0.0, z_y, z_z]]
    }
    /// Creates a matrix that performs a rotation along the y axis.
    pub fn y_rotate(radian: f64) -> [[f64; 3]; 3] {
        let x_x = radian.cos();
        let z_z = radian.cos();
        let x_z = radian.sin();
        let z_x = -radian.sin();
        [[x_x, 0.0, x_z], [0.0, 1.0, 0.0], [z_x, 0.0, z_z]]
    }
    /// Creates a matrix that performs a rotation along the z axis.
    pub fn z_rotate(radian: f64) -> [[f64; 3]; 3] {
        let x_x = radian.cos();
        let y_y = radian.cos();
        let x_y = -radian.sin();
        let y_x = radian.sin();
        [[x_x, x_y, 0.0], [y_x, y_y, 0.0], [0.0, 0.0, 1.0]]
    }
}

impl<T, const N: usize> VectorTrait<T, N> for [T; N] {
    fn vector_map<U>(self, f: impl FnMut(T) -> U) -> [U; N] {
        self.map(f)
    }

    fn vector_scale(self, scalar: T) -> [T; N]
    where
        T: Mul<T, Output = T>,
        T: Copy,
    {
        self.map(|v| v * scalar)
    }

    fn vector_neg(self) -> [T; N]
    where
        T: Neg<Output = T>,
    {
        self.map(|v| v.neg())
    }

    fn vector_add(self, rhs: [T; N]) -> [T; N]
    where
        T: Add<T, Output = T>,
    {
        let mut rhs = rhs.into_iter();
        self.map(|lhs| lhs + rhs.next().unwrap())
    }

    fn vector_add_assign(&mut self, rhs: [T; N])
    where
        T: AddAssign<T>,
    {
        for (dst, src) in self.iter_mut().zip(rhs.into_iter()) {
            *dst += src;
        }
    }

    fn vector_sub(self, rhs: [T; N]) -> [T; N]
    where
        T: Sub<T, Output = T>,
    {
        let mut rhs = rhs.into_iter();
        self.map(|lhs| lhs - rhs.next().unwrap())
    }

    fn vector_sub_assign(&mut self, rhs: [T; N])
    where
        T: SubAssign<T>,
    {
        for (dst, src) in self.iter_mut().zip(rhs.into_iter()) {
            *dst -= src;
        }
    }

    fn vector_dot(self, rhs: [T; N]) -> T
    where
        T: Mul<T, Output = T>,
        T: Sum,
    {
        self.into_iter()
            .zip(rhs.into_iter())
            .map(|(lhs, rhs)| lhs * rhs)
            .sum()
    }

    fn vector_map_index<U>(self, mut f: impl FnMut(T, usize) -> U) -> [U; N] {
        let mut index = 0..;
        self.map(|v| f(v, index.next().unwrap()))
    }

    fn vector_x(self) -> T
    where
        T: Copy,
    {
        *self.get(0).unwrap()
    }
    fn vector_y(self) -> T
    where
        T: Copy,
    {
        *self.get(1).unwrap()
    }
    fn vector_z(self) -> T
    where
        T: Copy,
    {
        *self.get(2).unwrap()
    }
    fn vector_w(self) -> T
    where
        T: Copy,
    {
        *self.get(3).unwrap()
    }
    fn vector_v(self) -> T
    where
        T: Copy,
    {
        *self.get(4).unwrap()
    }
    fn vector_u(self) -> T
    where
        T: Copy,
    {
        *self.get(5).unwrap()
    }
}

impl<T> VectorCrossTrait<T, 3> for [T; 3] {
    fn vector_cross(self, rhs: [T; 3]) -> [T; 3]
    where
        T: Mul<T, Output = T>,
        T: Sub<T, Output = T>,
        T: Copy,
    {
        [
            self[1] * rhs[2] - self[2] * rhs[1],
            self[2] * rhs[0] - self[0] * rhs[0],
            self[0] * rhs[1] - self[1] * rhs[0],
        ]
    }
}

impl<T> VectorCrossTrait<T, 4> for [T; 4] {
    fn vector_cross(self, rhs: [T; 4]) -> [T; 4]
    where
        T: Mul<T, Output = T>,
        T: Sub<T, Output = T>,
        T: Copy,
    {
        [
            self[1] * rhs[2] - self[2] * rhs[1],
            self[2] * rhs[0] - self[0] * rhs[0],
            self[0] * rhs[1] - self[1] * rhs[0],
            self[3],
        ]
    }
}

// 1t 0a
impl<T> Transmuter for ([T; 0],) {
    type Target = [(T,); 0];

    fn transmute(self) -> Self::Target {
        let ([],) = self;
        []
    }
}

// 1t 1a
impl<T> Transmuter for ([T; 1],) {
    type Target = [(T,); 1];

    fn transmute(self) -> Self::Target {
        let ([a],) = self;
        [(a,)]
    }
}

// 2t 1a
impl<T, U> Transmuter for ([T; 1], [U; 1]) {
    type Target = [(T, U); 1];

    fn transmute(self) -> Self::Target {
        let ([a], [b]) = self;
        [(a, b)]
    }
}
// 2t 2a
impl<T, U> Transmuter for ([T; 2], [U; 2]) {
    type Target = [(T, U); 2];

    fn transmute(self) -> Self::Target {
        let ([a1, a2], [b1, b2]) = self;
        [(a1, b1), (a2, b2)]
    }
}

// 2t 3a
impl<T, U> Transmuter for ([T; 3], [U; 3]) {
    type Target = [(T, U); 3];

    fn transmute(self) -> Self::Target {
        let ([a1, a2, a3], [b1, b2, b3]) = self;
        [(a1, b1), (a2, b2), (a3, b3)]
    }
}

// 3t 1a
impl<T, U, V> Transmuter for ([T; 1], [U; 1], [V; 1]) {
    type Target = [(T, U, V); 1];

    fn transmute(self) -> Self::Target {
        let ([a1], [b1], [c1]) = self;
        [(a1, b1, c1)]
    }
}
// 3t 2a
impl<T, U, V> Transmuter for ([T; 2], [U; 2], [V; 2]) {
    type Target = [(T, U, V); 2];

    fn transmute(self) -> Self::Target {
        let ([a1, a2], [b1, b2], [c1, c2]) = self;
        [(a1, b1, c1), (a2, b2, c2)]
    }
}

// 3t 3a
impl<T, U, V> Transmuter for ([T; 3], [U; 3], [V; 3]) {
    type Target = [(T, U, V); 3];

    fn transmute(self) -> Self::Target {
        let ([a1, a2, a3], [b1, b2, b3], [c1, c2, c3]) = self;
        [(a1, b1, c1), (a2, b2, c2), (a3, b3, c3)]
    }
}

// /// Atempt to automatise `Transmuter` implementation (unsuccessfull)
// macro_rules! transmuter_impl {
//     (($($type:ident),*) * [$n:literal]) => {
//         impl<$($type),*> Transmuter for ( $([$type; $n],)* ) {
//             type Target = [($( $type ,)*); $n];

//             fn transmute(self) -> Self::Target {
//                 // let ([a1, a2], [b1, b2]) = self;
//                 // [(a1, b1), (a2, b2)]
//                 todo!()
//             }
//         }
//     }
// }
// // transmuter_impl!((T, U, V) * [3]);
