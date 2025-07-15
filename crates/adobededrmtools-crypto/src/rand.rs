use rand_chacha::{
    ChaCha20Rng,
    rand_core::{RngCore, SeedableRng},
};
use rsa::rand_core::{
    CryptoRng as RsaCryptoRng, CryptoRngCore as RsaCryptoRngCore, RngCore as RsaRngCore,
};
use std::{cell::RefCell, rc::Rc, sync::OnceLock};

type Rng = ChaCha20Rng;

static INITIAL_SEED: OnceLock<[u8; 32]> = OnceLock::new();

pub fn init_rand(initial_seed: [u8; 32]) {
    INITIAL_SEED
        .set(initial_seed)
        .ok()
        .expect("cannot initialize seed multiple times");
}

struct RngRef(Rc<RefCell<Rng>>);

impl RsaCryptoRng for RngRef {}

impl RsaRngCore for RngRef {
    fn next_u32(&mut self) -> u32 {
        self.0.borrow_mut().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.borrow_mut().next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.borrow_mut().fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rsa::rand_core::Error> {
        Ok(self.0.borrow_mut().fill_bytes(dest))
    }
}

thread_local! {
    static THREAD_RNG: Rc<RefCell<Rng>> = create_rng();
}

fn create_rng() -> Rc<RefCell<Rng>> {
    let seed = *INITIAL_SEED
        .get()
        .expect("rand seed is not initialized. call init_rand(seed)");
    Rc::new(RefCell::new(Rng::from_seed(seed)))
}

pub fn rng() -> impl RsaCryptoRngCore {
    THREAD_RNG.with(|rng| RngRef(rng.clone()))
}

pub fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0; N];
    rng().fill_bytes(&mut buf);
    buf
}
