use sha1::Digest;

pub struct Sha1(::sha1::Sha1);

impl Sha1 {
    pub fn new() -> Self {
        Self(::sha1::Sha1::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finalize(self) -> [u8; 20] {
        self.0.finalize().into()
    }
}
