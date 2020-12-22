use std::cmp::Ordering;
use std::collections::VecDeque;

fn game(deck1: &[u8], deck2: &[u8]) -> VecDeque<u8> {
    let mut deck1: VecDeque<_> = deck1.iter().cloned().collect();
    let mut deck2: VecDeque<_> = deck2.iter().cloned().collect();
    loop {
        let card1 = deck1.pop_front().unwrap();
        let card2 = deck2.pop_front().unwrap();
        if card1 > card2 {
            deck1.push_back(card1);
            deck1.push_back(card2);
            if deck2.is_empty() {
                return deck1;
            }
        } else {
            deck2.push_back(card2);
            deck2.push_back(card1);
            if deck1.is_empty() {
                return deck2;
            }
        }
    }
}

fn recgame(
    mut deck1: VecDeque<u8>,
    mut deck2: VecDeque<u8>,
    not_known_to_loop: &[bool],
    toplevel: bool,
    verbose: bool,
) -> (Ordering, VecDeque<u8>) {
    let max1 = deck1.iter().max().unwrap();
    let max2 = deck2.iter().max().unwrap();
    let max_card = *std::cmp::max(max1, max2);

    if !toplevel {
        if max1 > max2 {
            return (Ordering::Less, VecDeque::new());
        }
        if not_known_to_loop[deck1.len() + deck2.len()] {
            return (max2.cmp(max1), VecDeque::new());
        }
    }

    let mut cache = std::collections::HashSet::new();

    loop {
        let cache_key: Vec<_> = deck1
            .iter()
            .cloned()
            .chain(std::iter::once(0))
            .chain(deck2.iter().cloned())
            .collect();
        if deck1[0] == max_card || deck2[0] == max_card {
            if cache.contains(&cache_key) {
                return (Ordering::Less, deck1);
            }
            cache.insert(cache_key);
        }
        let card1 = deck1.pop_front().unwrap();
        let card2 = deck2.pop_front().unwrap();

        let winner = if deck1.len() >= usize::from(card1) && deck2.len() >= usize::from(card2) {
            let subdeck1 = deck1.iter().take(usize::from(card1)).cloned().collect();
            let subdeck2 = deck2.iter().take(usize::from(card2)).cloned().collect();
            recgame(subdeck1, subdeck2, not_known_to_loop, false, verbose).0
        } else {
            card2.cmp(&card1)
        };

        match winner {
            Ordering::Less => {
                deck1.push_back(card1);
                deck1.push_back(card2);
                if deck2.is_empty() {
                    return (winner, deck1);
                }
            }
            Ordering::Greater => {
                deck2.push_back(card2);
                deck2.push_back(card1);
                if deck1.is_empty() {
                    return (winner, deck2);
                }
            }
            Ordering::Equal => unreachable!(),
        }
    }
}

fn score(deck: &[u8]) -> u16 {
    deck.iter()
        .rev()
        .enumerate()
        .map(|(i, &c)| ((i + 1) as u16) * u16::from(c))
        .sum()
}

fn main() {
    let s = adventofcode::read_input_file();
    let lines: Vec<_> = s.lines().collect();
    let deck_size = (lines.len() - 3) / 2;

    let mut not_known_to_loop = [false; 51];
    for &i in &[
        0, 1, 2, 3, 4, 6, 8, 12, 24, 32, 38, 40, 42, 44, 46, 48, 49, 50,
    ] {
        not_known_to_loop[i] = true;
    }

    let parse = |n: usize| {
        lines[n..(n + deck_size)]
            .iter()
            .map(|n| n.parse::<u8>().expect("can't parse integer"))
            .collect::<Vec<_>>()
    };
    let deck1 = parse(1);
    let deck2 = parse(deck_size + 3);
    let winner1 = game(&deck1, &deck2);
    println!("{}", score(&winner1.iter().cloned().collect::<Vec<_>>()));

    let deck1: VecDeque<_> = deck1.iter().cloned().collect();
    let deck2: VecDeque<_> = deck2.iter().cloned().collect();
    let (_, winner2) = recgame(deck1, deck2, &not_known_to_loop, true, false);
    println!("{}", score(&winner2.iter().cloned().collect::<Vec<_>>()));
}
