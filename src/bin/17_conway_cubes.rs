type Dim = u16;
type Time = u16;
type Coord = i32;
// x and y take 5 bits and other dimensions take 4, so that's a limit of 15 dimensions.
type Pos = i64;
// Since only 0, 1, 2, 3, 4+ matter, can use a u8 and saturating adds.
// This doesn't really appear to make a performance difference though,
// vs this u32 and not using saturating adds.
// u32 will support up to 20 dimensions (3 ** 20 < 2 ** 32 - 1)
type NeighCount = u32;

fn step(now_active: &[Pos], dimensions: Dim, dposes: &[Pos], wzbits: usize) -> Vec<Pos> {
    use std::collections::HashMap;

    // (neighbour count << 1) | self
    let mut neigh_and_self: HashMap<Pos, NeighCount> = HashMap::new();

    // Hmm, if I put this inside the loop,
    // would the compiler optimise it?
    let mut wz = vec![0; usize::from(dimensions) - 2];

    for &pos in now_active {
        let mut tmppos = pos;
        for i in 0..dimensions - 2 {
            wz[usize::from(i)] = (tmppos & ((1 << wzbits) - 1)) as Coord;
            tmppos >>= wzbits;
        }
        for dpos in dposes {
            let npos = pos + dpos;
            let mut tmpnpos = npos;
            let mut count = 2;
            for i in 0..dimensions - 2 {
                let c = tmpnpos & ((1 << wzbits) - 1);
                tmpnpos >>= wzbits;
                if c == 0 {
                    // negative coordinate (remember we offset by 1)
                    count *= 0;
                } else if c == 1 && wz[usize::from(i)] == 2 {
                    count *= 2;
                }
            }
            if count != 0 {
                *neigh_and_self.entry(npos).or_insert(0) += count;
            }
        }
    }
    for &pos in now_active {
        // If it was zero, adding 1 won't make a difference.
        neigh_and_self.entry(pos).and_modify(|e| *e += 1);
    }

    neigh_and_self
        .into_iter()
        .filter_map(|(pos, count)| {
            // 101 (2 neigh + self) 5
            // 110 (3 neigh)        6
            // 111 (3 neigh + self) 7
            if (5..=7).contains(&count) {
                Some(pos)
            } else {
                None
            }
        })
        .collect()
}

fn compress(
    x: Coord,
    y: Coord,
    wz: &[Coord],
    xyoffset: Coord,
    wzoffset: Coord,
    ybits: usize,
    wzbits: usize,
) -> Pos {
    let xy = ((x + xyoffset) << ybits) + y + xyoffset;
    wz.iter().fold(Pos::from(xy), |a, coord| {
        (a << wzbits) + Pos::from(coord + wzoffset)
    })
}

fn decompress(
    pos: Pos,
    dimensions: Dim,
    xyoffset: Coord,
    wzoffset: Coord,
    ybits: usize,
    wzbits: usize,
) -> Vec<Coord> {
    let mut pos = pos;
    let mut coord = vec![0; usize::from(dimensions)];
    for i in 0..dimensions - 2 {
        coord[usize::from(i) + 2] = ((pos & ((1 << wzbits) - 1)) as Coord) - wzoffset;
        pos >>= wzbits;
    }
    // y
    coord[1] = ((pos & ((1 << ybits) - 1)) as Coord) - xyoffset;
    // x
    coord[0] = ((pos >> ybits) as Coord) - xyoffset;
    coord
}

fn neigh(dimensions: Dim, ybits: usize, wzbits: usize) -> Vec<Pos> {
    // 0 will be first from repeated_permutation, so drop it with [1..]
    let coords = &repeated_permutation(&[0, -1, 1], usize::from(dimensions))[1..];
    coords
        .iter()
        .map(|coord| compress(coord[0], coord[1], &coord[2..], 0, 0, ybits, wzbits))
        .collect()
}

fn size(compressed: &[Pos], dimensions: Dim, wzbits: usize) -> u64 {
    compressed
        .iter()
        .map(|pos| {
            let mut count = 1_u64;
            let mut tmppos = *pos;
            for _ in 0..dimensions - 2 {
                let coord = tmppos & ((1 << wzbits) - 1);
                if coord != 1 {
                    count *= 2;
                }
                tmppos >>= wzbits;
            }
            count
        })
        .sum()
}

// Probably would be better represented as an iterator,
// but it's called just once so I don't really care.
fn repeated_permutation<T: Copy>(xs: &[T], n: usize) -> Vec<Vec<T>> {
    let mut vs = vec![vec![]];
    let mut n = n;
    while n > 0 {
        let mut vs_with = vec![];
        for x in xs {
            for v_without in &vs {
                let mut v_with = v_without.clone();
                v_with.push(*x);
                vs_with.push(v_with);
            }
        }
        vs = vs_with;
        n -= 1;
    }
    vs
}

fn bit_width(n: usize) -> usize {
    let mut w = 0;
    let mut n = n;
    while n > 0 {
        w += 1;
        n >>= 1;
    }
    w
}

fn active(s: &str) -> Vec<(usize, usize)> {
    s.lines()
        .enumerate()
        .flat_map(|(y, row)| {
            row.chars().enumerate().filter_map(
                move |(x, c)| {
                    if c == '#' {
                        Some((x, y))
                    } else {
                        None
                    }
                },
            )
        })
        .collect()
}

fn opts() -> (Option<Dim>, Time, String, bool) {
    let mut dim = None;
    let mut time = 6;
    let mut f = "/dev/stdin".to_string();
    let mut verbose = false;

    for arg in std::env::args() {
        if arg == "-v" {
            verbose = true;
        } else if let Some(stripped) = arg.strip_prefix("-d") {
            dim = Some(stripped.parse().expect("can't parse dim"));
        } else if let Some(stripped) = arg.strip_prefix("-t") {
            time = stripped.parse().expect("can't parse time");
        } else {
            f = arg;
        }
    }

    (dim, time, f, verbose)
}

fn main() {
    use std::time::Instant;

    let (dim, time, f, verbose) = opts();
    let grid = std::fs::read_to_string(f).expect("couldn't read file");
    let active2 = active(&grid);
    let max_y = active2.iter().map(|(_, y)| *y).max().unwrap_or(0);
    let ybits = bit_width(max_y + (time as usize) * 2 + 1);
    let wzbits = bit_width((time as usize) + 2);

    let dims = match dim {
        Some(d) => vec![d],
        None => vec![3, 4],
    };

    for dim in dims {
        let t1 = Instant::now();
        let dposes = neigh(dim, ybits, wzbits);
        let elapsed_neigh = t1.elapsed();

        let zs = vec![0; usize::from(dim) - 2];
        let mut active: Vec<_> = active2
            .iter()
            .map(|(x, y)| {
                compress(
                    i32::try_from(*x).expect("input too wide"),
                    i32::try_from(*y).expect("input too tall"),
                    &zs,
                    i32::from(time),
                    1,
                    ybits,
                    wzbits,
                )
            })
            .collect();

        let t2 = Instant::now();
        for _t in 1..=time {
            active = step(&active, dim, &dposes, wzbits);
            if false {
                println!("t={} {}", _t, size(&active, dim, wzbits));
                println!("t={} {:?}", _t, active);
                let coord: Vec<_> = active
                    .iter()
                    .map(|&pos| decompress(pos, dim, i32::from(time), 1, ybits, wzbits))
                    .collect();
                println!("{:?}", coord);
            }
        }
        println!("{}", size(&active, dim, wzbits));
        let elapsed_tot = t1.elapsed();
        let elapsed_steps = t2.elapsed();
        if dim > 4 || verbose {
            println!("neigh: {} ms", elapsed_neigh.as_millis());
            println!("steps: {} ms", elapsed_steps.as_millis());
            println!("total: {} ms", elapsed_tot.as_millis());
        }
    }
}
