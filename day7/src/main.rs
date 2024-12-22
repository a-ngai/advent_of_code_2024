use std::fs;
use std::time::Instant;

struct Node {
    num_add: u64,
    num_mul: u64,
    num_cat: u64,
    nums: Vec<u64>,
}

impl Node {

    fn yield_add_node(&self) -> Option<Self> {
        assert!(self.nums.len() >= 2);
        let mut nums_iter = self.nums.clone().into_iter();
        let num1: u64 = nums_iter.next().unwrap();
        let num2: u64 = nums_iter.next().unwrap();
        let rest_nums: Vec<u64> = nums_iter.collect();

        if self.num_add == 0 { return None }
        let mut child_add_nums = vec![num1 + num2];
        child_add_nums.extend(rest_nums.iter());
        let child_add_node = Node {
            nums: child_add_nums,
            num_add: self.num_add - 1,
            num_mul: self.num_mul,
            num_cat: self.num_cat,
        };

        Some(child_add_node)
    }

    fn yield_mul_node(&self) -> Option<Self> {
        assert!(self.nums.len() >= 2);
        let mut nums_iter = self.nums.clone().into_iter();
        let num1: u64 = nums_iter.next().unwrap();
        let num2: u64 = nums_iter.next().unwrap();
        let rest_nums: Vec<u64> = nums_iter.collect();

        if self.num_mul == 0 { return None }
        let mut child_mul_nums = vec![num1 * num2];
        child_mul_nums.extend(rest_nums.iter());
        let child_mul_node = Node {
            nums: child_mul_nums,
            num_add: self.num_add,
            num_mul: self.num_mul - 1,
            num_cat: self.num_cat,
        };

        Some(child_mul_node)
    }

    fn yield_cat_node(&self) -> Option<Self> {
        assert!(self.nums.len() >= 2);
        let mut nums_iter = self.nums.clone().into_iter();
        let num1: u64 = nums_iter.next().unwrap();
        let num2: u64 = nums_iter.next().unwrap();
        let rest_nums: Vec<u64> = nums_iter.collect();

        if self.num_cat == 0 { return None }
        let mut child_cat_nums = vec![
            (num1.to_string() + num2.to_string().as_str()).parse::<u64>().unwrap()
        ];
        child_cat_nums.extend(rest_nums.iter());
        let child_cat_node = Node {
            nums: child_cat_nums,
            num_add: self.num_add,
            num_mul: self.num_mul,
            num_cat: self.num_cat - 1,
        };

        Some(child_cat_node)
    }

    fn spawn_nodes(&self) -> Vec<Self> {
        let mut child_nodes: Vec<Node> = Vec::new();
        if let Some(node) = self.yield_add_node() { child_nodes.push(node) }
        if let Some(node) = self.yield_mul_node() { child_nodes.push(node) }
        if let Some(node) = self.yield_cat_node() { child_nodes.push(node) }
        child_nodes
    }

}

fn check_target_line(target: u64, nums: &[u64], cat: bool) -> bool {

    // I can optimize these two numbers!
    let num_mul = (nums.len() - 1) as u64;
    let num_add = (nums.len() - 1) as u64;
    let num_cat = if cat {
        (nums.len() - 1) as u64
    } else { 0u64 };

    // optimization here, decrease the num_mul, num_add, num_cat as much as possible!
    let mut num_add = num_add;
    let mut sorted_nums = nums.to_vec();
    sorted_nums.sort();
    sorted_nums.reverse();
    for num in sorted_nums.iter_mut() {
        if *num == 1 { *num = 2 }
    }

    if !cat {  // for no concatenation, limit max. possible additions
        let mut remove_add = num_mul;
        for test_num_mul in 0..(nums.len()) {
            let mul_part = sorted_nums[..test_num_mul].iter().product::<u64>();
            let add_part = sorted_nums[test_num_mul..].iter().sum::<u64>();
            let upper_bound: u64 = mul_part * add_part;
            if upper_bound >= target { 
                remove_add = test_num_mul as u64;
                break
            }
        }
        num_add -= remove_add;
    }
    
    // note that we could bound the num of adds, muls, and cats independently esp. get a lower
    // bound for num of cats, and upper bound for num of adds. The other bounds still exist, but
    // probably don't additionally restrict the bounds.
    //
    // if no bounds, then num of possibilities is (3*N choose N) / (N!)^3 ) (excluding pruning)
    // (this number is wrong! I'm missing a bunch of symmetry factors)
    //
    // Each time the bound is restricted by a value, the total number of possibilities decreases by
    // a factor of (3*N-N-1)/(3*N) ~ 2/3. So this is worthwhile to find!

    let first_node = Node { nums: nums.to_vec(), num_mul, num_add, num_cat};

    // depth-first search
    let mut stack: Vec<Node> = vec![first_node];
    let mut found_target: bool = false;
    assert!(nums.len() >= 2);
    while let Some(node) = stack.pop() {

        assert!(!node.nums.is_empty());
        let is_end_node = node.nums.len() == 1;
        if is_end_node {
            if node.nums[0] == target { 
            found_target = true;
            break
            } else { continue }
        } 

        assert!(node.nums.len() >= 2);
        if node.nums[0] > target { continue } // prune
        let child_nodes: Vec<Node> = node.spawn_nodes();
        stack.extend(child_nodes);
        }

    found_target
}

fn string_to_target_nums(string: &str) -> (u64, Vec<u64>) {
    let mut iter = string.split_whitespace();
    let mut target = iter.next().unwrap().chars();
    target.next_back();
    let target: u64 = target.as_str().parse().unwrap();

    let nums: Vec<u64> = iter
        .map(|string| string.parse().unwrap())
        .collect();

    (target, nums)
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file {filename}"));

    let before = Instant::now();

    let include_cat = false;
    let calibration_sum: u64 = textdata.lines()
        .map(string_to_target_nums)
        .filter(|(target, num)| check_target_line(*target, num, include_cat))
        .map(|(target, _num)| target)
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part  I): {after:2?}");
    println!("(Part  I): num. of valid cal. sum: {calibration_sum}");

    let include_cat = true;
    let calibration_sum: u64 = textdata.lines()
        .map(string_to_target_nums)
        .filter(|(target, num)| check_target_line(*target, num, include_cat))
        .map(|(target, _num)| target)
        .sum();

    let after = before.elapsed();
    println!("Time elapsed (Part II): {after:2?}");
    println!("(Part II): num. of valid cal. sum: {calibration_sum}");
}

#[test]
fn test_lines() {
    let include_cat = false;
    assert!(check_target_line(190, &[10, 19], include_cat));
    assert!(check_target_line(3267, &[81, 40, 27], include_cat));
    assert!(!check_target_line(83, &[17, 5], include_cat));
    assert!(!check_target_line(156, &[15, 6], include_cat));
    assert!(!check_target_line(7290, &[6, 8, 6, 15], include_cat));
    assert!(!check_target_line(161011, &[16, 10, 13], include_cat));
    assert!(!check_target_line(192, &[17, 8, 14], include_cat));
    assert!(!check_target_line(21037, &[9, 7, 18, 13], include_cat));
    assert!(check_target_line(292, &[11, 6, 16, 20], include_cat));

    assert!(check_target_line(1585066, &[66, 279, 9, 8, 86], include_cat));
}

#[test]
fn test_input() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .expect(format!("Cannot read file {filename}").as_str());

    let include_cat = false;
    let num_calibration = textdata.lines()
        .map(string_to_target_nums)
        .filter(|(target, num)| check_target_line(*target, num, include_cat))
        .count();
    assert_eq!(3, num_calibration)
}

#[test]
fn test_cat_input() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .expect(format!("Cannot read file {filename}").as_str());

    let include_cat = true;
    let num_calibration = textdata.lines()
        .map(string_to_target_nums)
        .filter(|(target, num)| check_target_line(*target, num, include_cat))
        .count();
    assert_eq!(6, num_calibration)
}

