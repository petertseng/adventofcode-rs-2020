// All the `as` casts are regrettable,
// but using Vec<u32> does make the code faster than Vec<usize>
fn game(cups: &[u8], ncups: u32, rounds: usize, after1: u32) -> Vec<u64> {
    let mut right: Vec<_> = (1..=(ncups + 1)).into_iter().collect();
    for (&l, &r) in cups.iter().zip(cups.iter().skip(1)) {
        right[usize::from(l)] = u32::from(r);
    }
    if ncups as usize > cups.len() {
        right[cups[cups.len() - 1] as usize] = cups.len() as u32 + 1;
        right[ncups as usize] = u32::from(cups[0]);
    } else {
        right[cups[cups.len() - 1] as usize] = u32::from(cups[0]);
    }

    let mut current = u32::from(cups[0]);

    for _ in 0..rounds {
        let pickup1 = right[current as usize];
        let pickup2 = right[pickup1 as usize];
        let pickup3 = right[pickup2 as usize];
        let after_pickup = right[pickup3 as usize];

        let mut dest = if current == 1 { ncups } else { current - 1 };
        while dest == pickup1 || dest == pickup2 || dest == pickup3 {
            dest = if dest == 1 { ncups } else { dest - 1 };
        }

        let right_of_dest = right[dest as usize];

        right[current as usize] = after_pickup;
        right[dest as usize] = pickup1;
        right[pickup3 as usize] = right_of_dest;

        current = after_pickup;
    }

    let mut current = 1;
    (0..after1)
        .map(|_| {
            let x = u64::from(right[current as usize]);
            current = x;
            x
        })
        .collect()
}

fn main() {
    // if arg looks like a number, use the number.
    // Otherwise, assume it's a file that will contain a number.
    let maybe_number = std::env::args().nth(1).unwrap_or_else(|| "x".to_string());
    let number = if maybe_number.chars().all(|c| ('0'..='9').contains(&c)) {
        maybe_number
    } else {
        adventofcode::read_input_file()
    };
    let cups: Vec<_> = number
        .chars()
        .filter_map(|c| c.to_digit(10).map(|c| c as u8))
        .collect();

    println!(
        "{}",
        game(&cups, cups.len() as u32, 100, cups.len() as u32 - 1)
            .iter()
            .fold(0, |a, x| a * 10 + x)
    );
    println!(
        "{}",
        game(&cups, 1_000_000, 10_000_000, 2)
            .iter()
            .product::<u64>()
    );
}
