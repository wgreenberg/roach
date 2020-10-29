#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hex {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub const ORIGIN: Hex = Hex {
    x: 0,
    y: 0,
    z: 0,
};