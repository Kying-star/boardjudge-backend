use sha2::Digest;
use sha2::Sha256;

pub fn sha256(text: &impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(text);
    hasher.finalize().to_vec()
}

#[macro_export]
macro_rules! uuid {
    ($id: expr) => {
        uuid::Uuid::from_str(&$id).unwrap()
    };
}
