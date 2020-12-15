const ROUNDS: u32 = 30_000_000;

struct Bitset {
    bits: Vec<u32>,
}

impl Bitset {
    // this should be u32::BITS, but that's unstable.
    const BLOCKSIZE: usize = 32;

    fn new(n: usize) -> Self {
        Self {
            bits: vec![0; n / Self::BLOCKSIZE],
        }
    }

    fn test(&self, i: usize) -> bool {
        let block = i / Self::BLOCKSIZE;
        let in_block = 1 << (i % Self::BLOCKSIZE);
        self.bits[block] & in_block == in_block
    }

    fn set(&mut self, i: usize) {
        let block = i / Self::BLOCKSIZE;
        let in_block = 1 << (i % Self::BLOCKSIZE);
        self.bits[block] |= in_block;
    }
}

fn game(
    t0: u32,
    spoken_now: u32,
    last_spoken_at: &mut [u32],
    seen: &mut Bitset,
    limit: u32,
) -> u32 {
    // Surprising (to me) speedup:
    // Keeping a bitset that tells whether a number has been seen at all.
    // (taken from askalski's C++ solution)
    // The entire last_spoken_at array is 114 MB, where as the bitset is 3,57 MB,
    // so I guess the bitset plays nicer with the cache.
    ((t0 + 1)..limit).fold(spoken_now, |speak, t| {
        // Value is small, so more than likely it's been seen.
        // Factor by which small is too small is subject to tuning.
        // 5 seemed to work the best for me.
        if speak < t >> 5 {
            // We don't need to seen.set here,
            // because values that are too small will remain too small
            // (t increases monotonically).
            let tprev = std::mem::replace(&mut last_spoken_at[speak as usize], t);
            if tprev == 0 {
                0
            } else {
                t - tprev
            }
        } else if seen.test(speak as usize) {
            t - std::mem::replace(&mut last_spoken_at[speak as usize], t)
        } else {
            seen.set(speak as usize);
            last_spoken_at[speak as usize] = t;
            0
        }
    })
}

fn rindex<T: PartialEq>(xs: &[T], x: T) -> Option<usize> {
    for i in 0..xs.len() {
        let j = xs.len() - 1 - i;
        if xs[j] == x {
            return Some(j);
        }
    }
    None
}

fn main() {
    let initial: Vec<_> = adventofcode::read_input_file()
        .trim()
        .split(',')
        .map(|n| n.parse::<usize>().expect("can't parse integer"))
        .collect();
    let mut last_spoken_at = vec![0; ROUNDS as usize];
    let mut seen = Bitset::new(ROUNDS as usize);
    for (t, &x) in initial.iter().enumerate() {
        seen.set(x as usize);
        last_spoken_at[x] = t as u32 + 1;
    }
    let spoken_now = match rindex(&initial[0..initial.len() - 1], initial[initial.len() - 1]) {
        Some(i) => (initial.len() - 1 - i) as u32,
        None => 0,
    };
    let spoken_now = game(
        initial.len() as u32,
        spoken_now,
        &mut last_spoken_at,
        &mut seen,
        2020,
    );
    println!("{}", spoken_now);
    println!(
        "{}",
        game(2019, spoken_now, &mut last_spoken_at, &mut seen, ROUNDS)
    );
}
