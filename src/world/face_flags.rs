pub type FaceFlags = usize;
pub const FACE_NONE: FaceFlags = 0;
pub const FACE_LEFT: FaceFlags = 1;
pub const FACE_RIGHT: FaceFlags = 2;
pub const FACE_BOTTOM: FaceFlags = 4;
pub const FACE_TOP: FaceFlags = 8;
pub const FACE_BACK: FaceFlags = 16;
pub const FACE_FRONT: FaceFlags = 32;
pub const FACE_ALL: FaceFlags =
    FACE_LEFT | FACE_RIGHT | FACE_BOTTOM | FACE_TOP | FACE_BACK | FACE_FRONT;
