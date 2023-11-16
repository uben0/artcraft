/// Keyboard state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Control {
    pub front: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub shift: bool,
}
impl Control {
    pub fn update(&mut self, key: u32, state: bool) {
        // key binding
        *match key {
            17 => &mut self.front,
            31 => &mut self.back,
            30 => &mut self.left,
            32 => &mut self.right,
            57 => &mut self.up,
            29 => &mut self.down,
            42 => &mut self.shift,
            _ => return,
        } = state;
    }
}
