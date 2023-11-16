pub const FACE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
pub const FACE_TEXTURE: [[u32; 2]; 4] = [[0, 0], [0, 1], [1, 1], [1, 0]];
pub const LINE_INDICES: [u32; 24] = [0, 3, 0, 1, 0, 7, 5, 4, 5, 2, 5, 6, 2, 3, 2, 1, 4, 3, 4, 7, 6, 1, 6, 7];
pub const LINE_VERTICES: [[u32; 3]; 8] = [
    [0, 0, 0],
    [0, 1, 0],
    [1, 1, 0],
    [1, 0, 0],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1],
    [0, 0, 1],
];