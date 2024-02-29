#[derive(Debug, Clone)]
///Almacena la información sobre qué modos están activados y cuáles desactivados
/// para un determinado canal.

pub struct ChannelModes {
    pub p: bool,
    pub s: bool,
    pub i: bool,
    pub t: bool,
    pub n: bool,
    pub m: bool,
    pub l: bool,
    pub k: bool,
}
impl Default for ChannelModes {
    fn default() -> Self {
        Self::new()
    }
}
impl ChannelModes {
    pub fn new() -> Self {
        Self {
            p: false,
            s: false,
            i: false,
            t: false,
            n: false,
            m: false,
            l: false,
            k: false,
        }
    }

    pub fn activate_p(&mut self) {
        self.p = true
    }
    pub fn deactivate_p(&mut self) {
        self.p = false
    }
    pub fn activate_s(&mut self) {
        self.s = true
    }
    pub fn deactivate_s(&mut self) {
        self.s = false
    }
    pub fn activate_i(&mut self) {
        self.i = true
    }
    pub fn deactivate_i(&mut self) {
        self.i = false
    }
    pub fn activate_t(&mut self) {
        self.t = true
    }
    pub fn deactivate_t(&mut self) {
        self.t = false
    }
    pub fn activate_n(&mut self) {
        self.n = true
    }
    pub fn deactivate_n(&mut self) {
        self.n = false
    }
    pub fn activate_m(&mut self) {
        self.m = true
    }
    pub fn deactivate_m(&mut self) {
        self.m = false
    }
    pub fn activate_l(&mut self) {
        self.l = true
    }
    pub fn deactivate_l(&mut self) {
        self.l = false
    }
    pub fn activate_k(&mut self) {
        self.k = true
    }
    pub fn deactivate_k(&mut self) {
        self.k = false
    }
}
