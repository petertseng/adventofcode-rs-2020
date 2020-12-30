use std::collections::HashMap;

type Dim = u16;
type Time = u16;
type Coord = i32;
// x and y take 5 bits; the other dimensions take in total (1 + nrounds) * log_2(dimensions).
// 54 / 7 is 7, so this could support up to 127 dimensions.
type Pos = i64;
// Since only 0, 1, 2, 3, 4+ matter, can use a u8 and saturating adds.
// This doesn't really appear to make a performance difference though,
// vs this u32 and not using saturating adds.
// u32 will support up to 20 dimensions (3 ** 20 < 2 ** 32 - 1)
type NeighCount = u32;

type NeighMap = HashMap<Pos, HashMap<Pos, NeighCount>>;
type CollapsedNeighMap = HashMap<Pos, Vec<(Pos, NeighCount)>>;

fn step(
    now_active: &[Pos],
    rounds: Time,
    weights: &CollapsedNeighMap,
    ybits: usize,
    wzbits: usize,
) -> Vec<Pos> {
    // (neighbour count << 1) | self
    let mut neigh_and_self: HashMap<Pos, NeighCount> = HashMap::new();
    let wzshift = wzbits * (usize::from(rounds) + 1);
    let wzmask = (1_i64 << wzshift) - 1;
    let pos_per_dy = 1_i64 << wzshift;
    let pos_per_dx = pos_per_dy << ybits;
    let dxys: Vec<_> = repeated_permutation(&[0, -1, 1], 2)
        .iter()
        .map(|dxy| dxy[0] * pos_per_dx + dxy[1] * pos_per_dy)
        .collect();

    for &pos in now_active {
        for (nwz, weight) in &weights[&(pos & wzmask)] {
            let npos = pos & !wzmask | nwz;
            for dxy in &dxys {
                *neigh_and_self.entry(npos + dxy).or_insert(0) += weight << 1;
            }
        }
        // for e.g. [x, y, z, w] -> [x + 1, y, z, w]
        // NOTE that if a cell is a representative of one of its own neighbours,
        // e.g, [x, y, 0, 1] -> [x, y, 1, 0] (which is represented by [x, y, 0, 1]),
        // the above weights will already have included that fact.
        // This is only for the single extra neighbour for nonequal [x, y].
        // TODO: Actually, it'd be possible to include the extra 1 in the neighbour map,
        // and then subtract the extra neighbour from pos...
        // You'd think runtime would benefit from avoiding this extra loop,
        // but it didn't seem to make an actual difference when I tried it?
        for dxy in &dxys[1..] {
            *neigh_and_self.entry(pos + dxy).or_insert(0) += 1 << 1;
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

// counting-based representation:
// x, y, higher_dimensions
// higher_dimensions is a count of how many of each of 0, 1, 2, ... rounds are present.
fn compress(
    x: Coord,
    y: Coord,
    wz: &[Time],
    rounds: Time,
    xyoffset: Coord,
    ybits: usize,
    wzbits: usize,
) -> Pos {
    let xy = ((x + xyoffset) << ybits) + y + xyoffset;
    (Pos::from(xy) << ((usize::from(rounds) + 1) * wzbits))
        + wz.iter()
            .map(|&z| 1_i64 << ((z as usize) * wzbits))
            .sum::<Pos>()
}

fn decompress(
    pos: Pos,
    dimensions: Dim,
    rounds: Time,
    xyoffset: Coord,
    ybits: usize,
    wzbits: usize,
) -> Vec<Coord> {
    let mut pos = pos;
    let mut coord = vec![0; usize::from(dimensions)];
    let mut i = 2;
    for j in 0..=rounds {
        let n = (pos & ((1 << wzbits) - 1)) as Coord;
        for _ in 0..n {
            coord[i] = Coord::from(j);
            i += 1;
        }
        pos >>= wzbits;
    }
    // y
    coord[1] = ((pos & ((1 << ybits) - 1)) as Coord) - xyoffset;
    // x
    coord[0] = ((pos >> ybits) as Coord) - xyoffset;
    coord
}

fn neigh_weights(dimensions: Dim, rounds: Time, wzbits: usize) -> CollapsedNeighMap {
    let mut weights = HashMap::new();
    // 0 will be first from repeated_permutation, so drop it with [1..]
    let ds = &repeated_permutation(&[0, -1, 1], usize::from(dimensions - 2))[1..];

    // Recursive closure pattern:
    // https://stackoverflow.com/questions/16946888/is-it-possible-to-make-a-recursive-closure-in-rust
    struct BuildIfRepresentative<'s> {
        f: &'s dyn Fn(&BuildIfRepresentative, Dim, &mut [Time], &mut NeighMap),
    }
    let build_if_representative = BuildIfRepresentative {
        f: &|build_if_rep, n, prefix, weights| {
            if n == dimensions - 2 {
                neigh_weights_for(prefix, ds, rounds, wzbits, weights);
            } else {
                let last = if n == 0 {
                    0
                } else {
                    prefix[usize::from(n) - 1]
                };
                for x in last..=rounds {
                    prefix[usize::from(n)] = x;
                    (build_if_rep.f)(build_if_rep, n + 1, prefix, weights);
                }
            }
        },
    };
    let mut prefix = vec![0; usize::from(dimensions) - 2];
    (build_if_representative.f)(&build_if_representative, 0, &mut prefix, &mut weights);

    weights
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect::<Vec<_>>()))
        .collect()
}

fn neigh_weights_for(
    pt: &[Time],
    ds: &[Vec<Coord>],
    rounds: Time,
    wzbits: usize,
    h: &mut NeighMap,
) {
    assert!(is_representative(pt));
    let comp_pt = compress(0, 0, pt, rounds, 0, 0, wzbits);
    for d in ds {
        let npt: Vec<_> = (0..pt.len()).map(|i| Coord::from(pt[i]) + d[i]).collect();
        // points with any coordinate equal to # rounds only appear in the last iteration,
        // so we don't need to compute their outgoing neighbours
        if npt.iter().any(|n| n.abs() >= Coord::from(rounds)) {
            continue;
        }
        // sorting not needed since counting compression is ordering-invariant
        let rep: Vec<_> = npt.iter().map(|c| c.abs() as Time).collect();
        let comp_neigh_rep = compress(0, 0, &rep, rounds, 0, 0, wzbits);
        *h.entry(comp_neigh_rep)
            .or_insert_with(HashMap::new)
            .entry(comp_pt)
            .or_insert(0) += 1;
    }
}

fn is_representative(pt: &[Time]) -> bool {
    for i in 1..pt.len() {
        if pt[i - 1] > pt[i] {
            return false;
        }
    }
    true
}

fn size(compressed: &[Pos], dimensions: Dim, rounds: Time, wzbits: usize) -> u64 {
    let perms_wz: u64 = (1..=u64::from(dimensions - 2)).product();
    compressed
        .iter()
        .map(|pos| {
            let mut count = 1_u64;
            let mut perms_pos = 1_u64;
            for i in 0..=rounds {
                let shifted = pos >> (wzbits * usize::from(i));
                let count_of_i = shifted & ((1 << wzbits) - 1);
                if i != 0 {
                    count <<= count_of_i;
                }
                perms_pos *= (1_u64..=(count_of_i as u64)).product::<u64>();
            }
            count * perms_wz / perms_pos
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

    let dims = match dim {
        Some(d) => vec![d],
        None => vec![3, 4],
    };

    for dim in dims {
        let wzbits = bit_width((dim as usize) - 2);

        let t1 = Instant::now();
        let weights = neigh_weights(dim, time, wzbits);
        let elapsed_neigh = t1.elapsed();

        let zs = vec![0; usize::from(dim) - 2];
        let mut active: Vec<_> = active2
            .iter()
            .map(|(x, y)| {
                compress(
                    i32::try_from(*x).expect("input too wide"),
                    i32::try_from(*y).expect("input too tall"),
                    &zs,
                    time,
                    i32::from(time),
                    ybits,
                    wzbits,
                )
            })
            .collect();

        let t2 = Instant::now();
        for _t in 1..=time {
            active = step(&active, time, &weights, ybits, wzbits);
            if false {
                println!("t={} {}", _t, size(&active, dim, time, wzbits));
                println!("t={} {:?}", _t, active);
                let coord: Vec<_> = active
                    .iter()
                    .map(|&pos| decompress(pos, dim, time, i32::from(time), ybits, wzbits))
                    .collect();
                println!("{:?}", coord);
            }
        }
        println!("{}", size(&active, dim, time, wzbits));
        let elapsed_tot = t1.elapsed();
        let elapsed_steps = t2.elapsed();
        if dim > 4 || verbose {
            println!("neigh: {} ms", elapsed_neigh.as_millis());
            println!("steps: {} ms", elapsed_steps.as_millis());
            println!("total: {} ms", elapsed_tot.as_millis());
        }
    }
}
