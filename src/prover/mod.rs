pub mod image_circuit;
pub mod poseidon;
pub mod greyscale;
pub mod sponge;
pub mod packer;
pub mod number;
#[cfg(test)]
mod tests;

// test-only tamper hooks: emulate a malicious prover by writing a dishonest witness
// value while leaving every circuit constraint in place. No effect outside cfg(test).
pub mod tamper {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum T { None, PackerInit, SpongeInit, SpongePad, PackByte(usize), ExifRange }

    #[cfg(test)]
    thread_local! { static CUR: std::cell::Cell<T> = std::cell::Cell::new(T::None); }

    #[cfg(test)]
    pub fn set(t: T) { CUR.with(|c| c.set(t)); }
    #[cfg(test)]
    pub fn get() -> T { CUR.with(|c| c.get()) }
    #[cfg(not(test))]
    pub fn get() -> T { T::None }
}