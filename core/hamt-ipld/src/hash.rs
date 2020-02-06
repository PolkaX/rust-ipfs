// Copyright 2019-2020 PolkaX. Licensed under MIT or Apache-2.0.

use std::cmp::Ordering;

/// ```go
/// func (d *digest64) Sum64() uint64 {
///     h1, _ = (*digest128)(d).Sum128()
///     return h1
/// }
/// ```
/// murmur3 hash for a bytes value. using hash128 but just pick half for result
#[cfg(not(feature = "test-hash"))]
pub fn hash<T: AsRef<[u8]>>(v: T) -> [u8; 8] {
    let result =
        murmur3::murmur3_x64_128(&mut v.as_ref(), 0).expect("murmur3 hash shouldn't be fail");
    // to big-ending sequence
    let all: [u8; 16] = result.to_be_bytes();
    // digest64 is half a digest128.
    let mut bytes = [0_u8; 8];
    // drop other half
    bytes.copy_from_slice(&all[8..]);
    bytes
}

/// replace hash function. jus for testing
/// `identityHash` just copy v to output
#[cfg(feature = "test-hash")]
pub fn hash<T: AsRef<[u8]>>(v: T) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    for (index, byte) in v.as_ref().iter().take(32).enumerate() {
        bytes[index] = *byte;
    }
    bytes
}

/// hashBits is a helper that allows the reading of the 'next n bits' as an integer.
/// e.g. bytes: [1, 66, 3], ([0b00000001, 0b01000010, 0b00000011]), read 10 bits would like
/// [0b________, 0b__0000010, 0b00000011], and return 0b0000000001 = 2 (u32)
#[derive(Debug, Clone)]
pub struct HashBits<'a> {
    b: &'a [u8],
    consumed: u32,
    pub bit_width: u32,
}

impl<'a> HashBits<'a> {
    pub fn new(buf: &'a [u8], bit_width: u32) -> HashBits<'a> {
        Self::new_with_consumed(buf, 0, bit_width)
    }

    pub fn new_with_consumed(buf: &'a [u8], consumed: u32, bit_width: u32) -> HashBits<'a> {
        HashBits {
            b: buf,
            consumed,
            bit_width,
        }
    }

    pub fn consumed(&self) -> u32 {
        self.consumed
    }

    /// Next returns the next 'i' bits of the hashBits value as an u32,
    /// or `None `if there aren't enough bits.
    pub fn next(&mut self) -> Option<u32> {
        let i = self.bit_width;
        let new_consumed = self.consumed.checked_add(i)?;
        if new_consumed > self.b.len() as u32 * 8 {
            return None;
        }
        // return value is u32, couldn't pick over 32
        if i > 32 || i == 0 {
            return None;
        }

        let out = self.next_bit(i);
        Some(out)
    }

    fn next_bit(&mut self, i: u32) -> u32 {
        let cur_byte_index = (self.consumed / 8) as usize;
        let left_bit = 8 - (self.consumed % 8); // consumed % 8, left_bit is less and equal than 8

        let cur_byte = self.b[cur_byte_index];
        match i.cmp(&left_bit) {
            Ordering::Equal => {
                // i and left_bit must less or equal than 8
                let out = mkmask(i) & cur_byte;
                self.consumed += i;
                out as u32
            }
            Ordering::Less => {
                // i must less than 8, left_bit must less or equal than 8
                // e.g. cur_byte: 0b11111111, self.consumed % 8=1, left_bit=7, i=2, then:
                // a=0b_1111111
                let a = cur_byte & mkmask(left_bit); // mask out the high bits we don't want, do not need consumed bits
                                                     // b=0b_11_____
                let b = a & (!mkmask(left_bit - i)); // mask out the low bits we don't want, do not need unused bits
                                                     // c=0b______11
                let c = b as u32 >> (left_bit - i); // shift whats left down
                self.consumed += i;
                c
            }
            Ordering::Greater => {
                // must beyond current byte, pick all left_bit
                let mut out = (mkmask(left_bit) & cur_byte) as u32;
                out <<= i - left_bit;
                self.consumed += left_bit;
                out += self.next_bit(i - left_bit);
                out
            }
        }
    }
}

#[inline]
fn mkmask(n: u32) -> u8 {
    ((1_u32 << n) - 1) as u8
}
