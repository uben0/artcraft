//! Introduces matrix and vector operations.
//!
//! This implementation does not aim to be fast or optimized but can perfectly
//! be used to compute uniforms for graphic applications.
//! ```
//! use arrayscalar::MatrixTrait;
//!
//! let m1 = [
//!     [1, 2, 3],
//!     [4, 5, 6],
//! ].matrix_transpose();
//!
//! let m2 = [
//!     [0, 1],
//!     [2, 3],
//!     [4, 5],
//! ].matrix_transpose();
//!
//! let m3 = m1.matrix_mul(m2);
//!
//! let m4 = [
//!     [16, 22],
//!     [34, 49],
//! ].matrix_transpose();
//!
//! assert_eq!(m3, m4);
//! ```
//! As matrices are considered to be column major (array of column), it is possible
//! to write and read them in line major by using transposition to perform conversion.

mod behavior;

use std::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign},
};

/// Extends array with matrix operations.
///
/// This implementation consider the matrix to be column major, ie, an array of columns.
/// Implemented for `[[T; M]; N]`, M for the line number and N for the column number.
pub trait MatrixTrait<T, const M: usize, const N: usize> {
    /// Transforms all element of the matrix with the given function.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [0, 1, 2],
    ///     [3, 4, 5],
    /// ];
    /// let m2 = m1.matrix_map(|v| if v < 2 { -1 } else { 2 });
    ///
    /// assert_eq!(m2, [
    ///     [-1, -1, 2],
    ///     [ 2,  2, 2],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_map<U, F: FnMut(T) -> U>(self, f: F) -> [[U; M]; N];
    /// Transforms all element of the matrix with the given function and current index.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [10, 13],
    ///     [10, 11],
    /// ];
    /// let m2 = m1.matrix_map_index(|v, m, n| (v, m, n));
    ///
    /// assert_eq!(m2, [
    ///     [(10, 0, 0), (13, 1, 0)],
    ///     [(10, 0, 1), (11, 1, 1)],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_map_index<U, F: FnMut(T, usize, usize) -> U>(self, f: F) -> [[U; M]; N];
    /// Returns the transposed matrix.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [1, 2],
    ///     [3, 4],
    ///     [5, 6],
    /// ];
    /// let m2 = m1.matrix_transpose();
    /// assert_eq!(m2, [
    ///     [1, 3, 5],
    ///     [2, 4, 6],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_transpose(self) -> [[T; N]; M];
    /// Returns the scaled matrix by a given factor.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [1, 2, 3],
    ///     [4, 5, 6],
    /// ];
    /// let m2 = m1.matrix_scale(3);
    /// assert_eq!(m2, [
    ///     [ 3,  6,  9],
    ///     [12, 15, 18],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_scale(self, scalar: T) -> [[T; M]; N]
    where
        T: Mul<T, Output = T>,
        T: Copy;
    /// Returns the addition of the two matrices.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [1, 2, 3],
    ///     [4, 5, 6],
    /// ];
    /// let m2 = [
    ///     [60, 50, 40],
    ///     [30, 20, 10],
    /// ];
    /// let m3 = m1.matrix_add(m2);
    /// assert_eq!(m3, [
    ///     [61, 52, 43],
    ///     [34, 25, 16],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_add(self, rhs: [[T; M]; N]) -> [[T; M]; N]
    where
        T: Add<T, Output = T>;
    /// Returns the subtraction of the two matrices.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [61, 52, 43],
    ///     [34, 25, 16],
    /// ];
    /// let m2 = [
    ///     [1, 2, 3],
    ///     [4, 5, 6],
    /// ];
    /// let m3 = m1.matrix_sub(m2);
    /// assert_eq!(m3, [
    ///     [60, 50, 40],
    ///     [30, 20, 10],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_sub(self, rhs: [[T; M]; N]) -> [[T; M]; N]
    where
        T: Sub<T, Output = T>;
    /// Returns the multiplication of the two matrices.
    ///
    /// ```
    /// # use arrayscalar::MatrixTrait;
    /// let m1 = [
    ///     [1, 2, 3],
    ///     [4, 5, 6],
    /// ];
    /// let m2 = [
    ///     [0, 1],
    ///     [2, 3],
    ///     [4, 5],
    /// ];
    /// let m3 = m1.matrix_mul(m2);
    /// assert_eq!(m3, [
    ///     [ 4,  5,  6],
    ///     [14, 19, 24],
    ///     [24, 33, 42],
    /// ]);
    /// ```
    #[must_use]
    fn matrix_mul<const O: usize>(self, rhs: [[T; N]; O]) -> [[T; M]; O]
    where
        T: Mul<T, Output = T>,
        T: Sum,
        T: Copy;
}

/// Dummy type used as generic module for affine matrices (4x4 with coords x, y, z and w as homogeneous).
///
/// ```
/// use arrayscalar::{Affine, AffineTrait};
///
/// let m: [[f32; 4]; 4] = Affine::identity()
///     .affine_translate([1.0, 10.0, 5.0])
///     .affine_scale(2.0)
///     .affine_y_rotate(3.1416);
/// ```
/// The above matrix will perform a y rotation followed by a scalling then a translation.
pub struct Affine<T, const N: usize = 4> {
    _holder: [T; N],
}

/// Extends array with affine operations (4x4 with coords x, y, z and w as homogeneous).
pub trait AffineTrait<T, const N: usize = 3, const O: usize = 4>
where
    T: Mul<T, Output = T>,
    T: Sum,
    T: Product,
    T: Copy,
{
    /// Returns the multiplication of the current matrix and one that performs a translation.
    #[must_use]
    fn affine_translate(self, vector: [T; N]) -> [[T; O]; O];
    /// Returns the multiplication of the current matrix and one that performs a scaling.
    #[must_use]
    fn affine_scale(self, scalar: T) -> [[T; O]; O];
    /// Returns the multiplication of the current matrix and one that performs a rotation on the x axis.
    #[must_use]
    fn affine_x_rotate(self, radian: T) -> [[T; O]; O];
    /// Returns the multiplication of the current matrix and one that performs a rotation on the y axis.
    #[must_use]
    fn affine_y_rotate(self, radian: T) -> [[T; O]; O];
    /// Returns the multiplication of the current matrix and one that performs a rotation on the z axis.
    #[must_use]
    fn affine_z_rotate(self, radian: T) -> [[T; O]; O];
}

/// Extends array with vector operations.
pub trait VectorTrait<T, const N: usize> {
    /// Equivalent to array map.
    #[must_use]
    fn vector_map<U>(self, f: impl FnMut(T) -> U) -> [U; N];
    /// Returns the mapped vector with the given function and the current index.
    #[must_use]
    fn vector_map_index<U>(self, f: impl FnMut(T, usize) -> U) -> [U; N];

    /// Returns the scalled vector by a given factor.
    #[must_use]
    fn vector_scale(self, scalar: T) -> [T; N]
    where
        T: Mul<T, Output = T>,
        T: Copy;

    /// Returns the negative of the vector.
    #[must_use]
    fn vector_neg(self) -> [T; N]
    where
        T: Neg<Output = T>;

    /// Returns the addition of the two vector.
    #[must_use]
    fn vector_add(self, rhs: [T; N]) -> [T; N]
    where
        T: Add<T, Output = T>;

    /// Performs add assignation.
    fn vector_add_assign(&mut self, rhs: [T; N])
    where
        T: AddAssign<T>;

    /// Returns the subtraction of the two vector.
    #[must_use]
    fn vector_sub(self, rhs: [T; N]) -> [T; N]
    where
        T: Sub<T, Output = T>;

    /// Performs the subtraction assignation.
    fn vector_sub_assign(&mut self, rhs: [T; N])
    where
        T: SubAssign<T>;

    /// Returns the dot product of the two vector.
    #[must_use]
    fn vector_dot(self, rhs: [T; N]) -> T
    where
        T: Mul<T, Output = T>,
        T: Sum;

    /// Returns the first element of the vector.
    #[must_use]
    fn vector_x(self) -> T
    where
        T: Copy;

    /// Returns the second element of the vector.
    #[must_use]
    fn vector_y(self) -> T
    where
        T: Copy;

    /// Returns the third element of the vector.
    #[must_use]
    fn vector_z(self) -> T
    where
        T: Copy;

    /// Returns the fourth element of the vector.
    #[must_use]
    fn vector_w(self) -> T
    where
        T: Copy;

    /// Returns the fifth element of the vector.
    #[must_use]
    fn vector_v(self) -> T
    where
        T: Copy;

    /// Returns the sixth element of the vector.
    #[must_use]
    fn vector_u(self) -> T
    where
        T: Copy;
}

/// Extends 3 dimensional vector with cross product operation (including 4 dimension because of homogeneous).
pub trait VectorCrossTrait<T, const N: usize>: VectorTrait<T, N> {
    #[must_use]
    fn vector_cross(self, rhs: [T; N]) -> [T; N]
    where
        T: Mul<T, Output = T>,
        T: Sub<T, Output = T>,
        T: Copy;
}

pub trait Transmuter {
    type Target;

    fn transmute(self) -> Self::Target;
}
