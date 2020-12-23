fn game(cups: &[usize], ncups: usize, rounds: usize, after1: usize) -> Vec<usize> {
    let mut right: Vec<_> = (1..=(ncups + 1)).into_iter().collect();
    for (&l, &r) in cups.iter().zip(cups.iter().skip(1)) {
        right[l] = r;
    }
    if ncups > cups.len() {
        right[cups[cups.len() - 1]] = cups.len() + 1;
        right[ncups] = cups[0];
    } else {
        right[cups[cups.len() - 1]] = cups[0];
    }

    let mut current = cups[0];

    for _ in 0..rounds {
        let pickup1 = right[current];
        let pickup2 = right[pickup1];
        let pickup3 = right[pickup2];
        let after_pickup = right[pickup3];

        let mut dest = if current == 1 { ncups } else { current - 1 };
        while dest == pickup1 || dest == pickup2 || dest == pickup3 {
            dest = if dest == 1 { ncups } else { dest - 1 };
        }

        let right_of_dest = right[dest];

        right[current] = after_pickup;
        right[dest] = pickup1;
        right[pickup3] = right_of_dest;

        current = after_pickup;
    }

    let mut current = 1;
    (0..after1)
        .map(|_| {
            let x = right[current];
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
        .filter_map(|c| c.to_digit(10).map(|c| c as usize))
        .collect();

    println!(
        "{}",
        game(&cups, cups.len(), 100, cups.len() - 1)
            .iter()
            .fold(0, |a, x| a * 10 + x)
    );
    println!(
        "{}",
        game(&cups, 1_000_000, 10_000_000, 2)
            .iter()
            .product::<usize>()
    );
}
