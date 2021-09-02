pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}
