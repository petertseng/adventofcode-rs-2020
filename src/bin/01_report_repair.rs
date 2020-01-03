use std::collections::HashSet;

fn pair(nums: &[u32], nums_set: &HashSet<u32>) -> Vec<u32> {
    let mut ans = Vec::new();
    for num in nums {
        let needed = 2020 - num;
        if needed < *num {
            continue;
        }
        if nums_set.contains(&needed) {
            ans.push(num * needed);
        }
    }
    ans
}

fn main() {
    let nums =
        adventofcode::read_input_lines(|line| line.parse::<u32>().expect("can't parse integer"));
    let nums_set = nums.iter().cloned().collect();

    for x in pair(&nums, &nums_set) {
        println!("{}", x);
    }
    let try1 = adventofcode::day01::by_combinations(&nums, &nums_set);
    let try2 = adventofcode::day01::over_input_range(&nums, &nums_set);
    let try3 = adventofcode::day01::by_combinations_sorted(&nums, &nums_set);
    assert_eq!(try1, try2);
    assert_eq!(try1, try3);
    for x in try1 {
        println!("{}", x);
    }
}
