use std::collections::HashSet;

pub fn by_combinations(nums: &[u32], nums_set: &HashSet<u32>) -> Vec<u32> {
    let mut ans = Vec::new();
    for (i, x) in nums.iter().enumerate() {
        for &y in nums.iter().skip(i + 1) {
            let needed = 2020 - x - y;
            if needed < *x || needed < y {
                continue;
            }
            if nums_set.contains(&needed) {
                ans.push(x * y * needed);
            }
        }
    }
    ans
}

pub fn by_combinations_sorted(nums: &[u32], nums_set: &HashSet<u32>) -> Vec<u32> {
    let mut ans = Vec::new();
    let sorted = {
        let mut tmp = nums.to_vec();
        tmp.sort_unstable();
        tmp
    };
    for (i, x) in sorted.iter().enumerate() {
        if x * 3 > 2020 {
            break;
        }
        for &y in sorted.iter().skip(i + 1) {
            let needed = 2020 - x - y;
            if needed < y {
                break;
            }
            if nums_set.contains(&needed) {
                ans.push(x * y * needed);
            }
        }
    }
    ans
}

fn minmax(nums: &[u32]) -> (u32, u32) {
    let mut min = u32::MAX;
    let mut max = u32::MIN;

    for &x in nums {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }

    (min, max)
}

pub fn over_input_range(nums: &[u32], nums_set: &HashSet<u32>) -> Vec<u32> {
    let mut ans = Vec::new();
    let (min, max) = minmax(nums);
    for a in min..=max {
        if a * 3 > 2020 {
            break;
        }

        if !nums_set.contains(&a) {
            continue;
        }

        for b in a..=max {
            let c = 2020 - a - b;
            if c < b {
                break;
            }
            if nums_set.contains(&b) && nums_set.contains(&c) {
                ans.push(a * b * c);
            }
        }
    }
    ans
}
