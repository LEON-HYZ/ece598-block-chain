use serde::{Serialize, Deserialize};
use std::convert::TryInto;

/// An object that can be meaningfully hashed.


pub trait Hashable {
    /// Hash the object using SHA256.
    fn hash(&self) -> H256;
}

/// A SHA256 hash.
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Default, Copy)]
pub struct H256([u8; 32]); // big endian u256

impl Hashable for H256 {
    fn hash(&self) -> H256 {
        ring::digest::digest(&ring::digest::SHA256, &self.0).into()
    }
}

impl std::fmt::Display for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = if let Some(precision) = f.precision() {
            if precision >= 64 {
                0
            } else {
                32 - precision / 2
            }
        } else {
            0
        };
        for byte_idx in start..32 {
            write!(f, "{:>02x}", &self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &self.0[0], &self.0[1], &self.0[30], &self.0[31]
        )
    }
}


impl std::convert::AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::convert::From<&[u8; 32]> for H256 {
    fn from(input: &[u8; 32]) -> H256 {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(input);
        H256(buffer)
    }
}

impl std::convert::From<&H256> for [u8; 32] {
    fn from(input: &H256) -> [u8; 32] {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(&input.0);
        buffer
    }
}

impl std::convert::From<[u8; 32]> for H256 {
    fn from(input: [u8; 32]) -> H256 {
        H256(input)
    }
}

impl std::convert::From<H256> for [u8; 32] {
    fn from(input: H256) -> [u8; 32] {
        input.0
    }
}

impl std::convert::From<ring::digest::Digest> for H256 {
    fn from(input: ring::digest::Digest) -> H256 {
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].copy_from_slice(input.as_ref());
        H256(raw_hash)
    }
}

impl Ord for H256 {
    fn cmp(&self, other: &H256) -> std::cmp::Ordering {
        let self_higher = u128::from_be_bytes(self.0[0..16].try_into().unwrap());
        let self_lower = u128::from_be_bytes(self.0[16..32].try_into().unwrap());
        let other_higher = u128::from_be_bytes(other.0[0..16].try_into().unwrap());
        let other_lower = u128::from_be_bytes(other.0[16..32].try_into().unwrap());
        let higher = self_higher.cmp(&other_higher);
        match higher {
            std::cmp::Ordering::Equal => self_lower.cmp(&other_lower),
            _ => higher,
        }
    }
}

impl PartialOrd for H256 {
    fn partial_cmp(&self, other: &H256) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(any(test, test_utilities))]
pub mod tests {
    use super::H256;
    use rand::Rng;

    pub fn generate_random_hash() -> H256 {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let mut raw_bytes = [0; 32];
        raw_bytes.copy_from_slice(&random_bytes);
        (&raw_bytes).into()
    }

}


pub trait Hashable_160 {
    /// Hash the object using SHA256.
    fn hash(&self) -> H160;
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Default, Copy)]
pub struct H160([u8; 20]);



impl Hashable_160 for H160{
    fn hash(&self) -> H160{
        let mut Hash256 : H256 = ring::digest::digest(&ring::digest::SHA256, &self.0).into();
        let mut Hash160 = [0; 20];
        Hash160.copy_from_slice(&Hash256.0[12..32]); //last 20 bytes
        //let mut Hash160 =
        //Hash160.into();
        return H160(Hash160)
    }
}

impl std::fmt::Debug for H160 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &self.0[0], &self.0[1], &self.0[18], &self.0[19]
        )
    }
}


impl std::convert::From<&[u8; 32]> for H160 {
    fn from(input: &[u8; 32]) -> H160 {
        let mut buffer160: [u8; 20] = [0; 20];
        buffer160[..].copy_from_slice(&input[12..32]);
        H160(buffer160)
    }
}


impl std::convert::From<H256> for H160 {
    fn from(input: H256) -> H160 {
        let buffer256:[u8; 32] = input.0;
        let mut buffer160: [u8; 20] = [0; 20];
        buffer160[..].copy_from_slice(&buffer256[12..32]);
        H160(buffer160)
    }
}

impl std::convert::From<[u8; 20]> for H160 {
    fn from(input: [u8; 20]) -> H160 {
        H160(input)
    }
}

impl std::convert::From<H160> for [u8; 20] {
    fn from(input: H160) -> [u8; 20] {
        input.0
    }
}

impl std::convert::From<ring::digest::Digest> for H160 {
    fn from(input: ring::digest::Digest) -> H160 {
        let mut raw_hash: [u8; 20] = [0; 20];
        raw_hash[0..20].copy_from_slice(&input.as_ref()[12..32]);
        H160(raw_hash)
    }
}

impl Ord for H160 {
    fn cmp(&self, other: &H160) -> std::cmp::Ordering {
        let self_higher = u128::from_be_bytes(self.0[0..10].try_into().unwrap());
        let self_lower = u128::from_be_bytes(self.0[10..20].try_into().unwrap());
        let other_higher = u128::from_be_bytes(other.0[0..10].try_into().unwrap());
        let other_lower = u128::from_be_bytes(other.0[10..20].try_into().unwrap());
        let higher = self_higher.cmp(&other_higher);
        match higher {
            std::cmp::Ordering::Equal => self_lower.cmp(&other_lower),
            _ => higher,
        }
    }
}

impl PartialOrd for H160 {
    fn partial_cmp(&self, other: &H160) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}