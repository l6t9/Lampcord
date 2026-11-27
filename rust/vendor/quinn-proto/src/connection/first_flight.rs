//! Local patch (vendor/README.md): the layout Chromium gives a client's first flight.
//!
//! Chromium splits the ClientHello across two Initial packets of equal share, cuts each into many CRYPTO frames of very different sizes in
//! random order, and fills the rest of both 1250-byte packets with PING frames and runs of PADDING scattered between them. This builds that
//! layout (measured from Chrome 154); the connection writes it in place of its usual single CRYPTO frame.

use bytes::Bytes;
use rand::{Rng, RngExt, seq::SliceRandom as _};

use crate::VarInt;

#[derive(Debug)]
pub(crate) enum Item {
    Crypto { offset: u64, data: Bytes },
    Ping,
    Padding(usize),
}

/// The bytes a CRYPTO frame of `len` bytes at `offset` takes.
fn crypto_size(offset: u64, len: usize) -> usize {
    1 + VarInt::from_u64(offset).map_or(8, |v| v.size()) + VarInt::from_u64(len as u64).map_or(8, |v| v.size()) + len
}

/// Splits `hello` into the payloads of two Initial packets with `capacity` bytes of frames each (the second packet's packet number is a byte longer). `None` if it should not be split
/// (too small, or it would not fit).
pub(crate) fn plan(hello: &Bytes, capacity: [usize; 2], rng: &mut impl Rng) -> Option<[Vec<Item>; 2]> {
    let total = hello.len();
    if total < 600 {
        return None;
    }
    // Cut points: some spread over the whole hello, more close behind an earlier one, which makes the sizes very uneven.
    let pieces_wanted = rng.random_range(18..=26usize);
    let mut cuts: Vec<usize> = Vec::new();
    while cuts.len() < pieces_wanted - 1 {
        let cut = if cuts.is_empty() || rng.random_range(0..10) < 4 {
            rng.random_range(1..total)
        } else {
            let base = cuts[rng.random_range(0..cuts.len())];
            base + rng.random_range(1..=40usize)
        };
        if cut > 0 && cut < total && !cuts.contains(&cut) {
            cuts.push(cut);
        }
    }
    cuts.sort_unstable();
    let mut pieces: Vec<(usize, usize)> = Vec::with_capacity(cuts.len() + 1);
    let mut start = 0;
    for cut in cuts.into_iter().chain(std::iter::once(total)) {
        pieces.push((start, cut - start));
        start = cut;
    }
    pieces.shuffle(rng);

    // The first packet takes about half (Chromium's is a few bytes short of exactly half).
    let first_target = (total / 2).saturating_sub(2).max(1);
    let mut groups: [Vec<(usize, usize)>; 2] = [Vec::new(), Vec::new()];
    let mut taken = 0;
    for (offset, len) in pieces {
        if taken >= first_target {
            groups[1].push((offset, len));
        } else if taken + len > first_target {
            let head = first_target - taken;
            groups[0].push((offset, head));
            groups[1].push((offset + head, len - head));
            taken = first_target;
        } else {
            groups[0].push((offset, len));
            taken += len;
        }
    }

    let mut out: [Vec<Item>; 2] = [Vec::new(), Vec::new()];
    for ((group, items), capacity) in groups.iter().zip(out.iter_mut()).zip(capacity) {
        let used: usize = group.iter().map(|&(offset, len)| crypto_size(offset as u64, len)).sum();
        if used > capacity {
            return None;
        }
        let mut spare = capacity - used;
        let pings = rng.random_range(3..=12usize).min(spare);
        spare -= pings;
        for &(offset, len) in group {
            items.push(Item::Crypto { offset: offset as u64, data: hello.slice(offset..offset + len) });
        }
        items.extend((0..pings).map(|_| Item::Ping));
        if spare > 0 {
            let runs = rng.random_range(4..=10usize).min(spare);
            let mut points: Vec<usize> = (0..runs - 1).map(|_| rng.random_range(1..spare.max(2))).collect();
            points.push(0);
            points.push(spare);
            points.sort_unstable();
            points.dedup();
            for pair in points.windows(2) {
                items.push(Item::Padding(pair[1] - pair[0]));
            }
        }
        items.shuffle(rng);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    fn size_of(items: &[Item]) -> usize {
        items
            .iter()
            .map(|item| match item {
                Item::Crypto { offset, data } => crypto_size(*offset, data.len()),
                Item::Ping => 1,
                Item::Padding(n) => *n,
            })
            .sum()
    }

    #[test]
    fn the_hello_is_covered_once_and_both_packets_are_full() {
        for seed in 0..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            let hello = Bytes::from((0..1910u32).map(|i| i as u8).collect::<Vec<u8>>());
            let [a, b] = plan(&hello, [1215, 1214], &mut rng).expect("a plan");
            assert_eq!(size_of(&a), 1215, "seed {seed}");
            assert_eq!(size_of(&b), 1214, "seed {seed}");
            let mut covered = vec![0u8; hello.len()];
            for item in a.iter().chain(b.iter()) {
                if let Item::Crypto { offset, data } = item {
                    assert_eq!(&hello[*offset as usize..*offset as usize + data.len()], &data[..]);
                    for slot in &mut covered[*offset as usize..*offset as usize + data.len()] {
                        *slot += 1;
                    }
                }
            }
            assert!(covered.iter().all(|&n| n == 1), "seed {seed}: every byte exactly once");
        }
    }

    #[test]
    fn a_small_hello_is_left_alone() {
        let mut rng = StdRng::seed_from_u64(1);
        assert!(plan(&Bytes::from(vec![0u8; 300]), [1215, 1214], &mut rng).is_none());
    }
}
