use std::hash::{Hasher, Hash, Writer};

#[deriving(Clone, Default)]
pub struct FnvHasher;

pub struct FnvState(u64);

impl Hasher<FnvState> for FnvHasher {
    fn hash<Sized? T: Hash<FnvState>>(&self, t: &T) -> u64 {
        let mut state = FnvState(0xcbf29ce484222325);
        t.hash(&mut state);
        let FnvState(ret) = state;
        return ret;
    }
}

impl Writer for FnvState {
    fn write(&mut self, bytes: &[u8]) {
        let FnvState(mut hash) = *self;
        for byte in bytes.iter() {
            hash = hash ^ (*byte as u64);
            hash = hash * 0x100000001b3;
        }
        *self = FnvState(hash);
    }
}