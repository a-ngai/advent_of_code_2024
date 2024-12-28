use std::fs;
use std::time::Instant;

// Optimizations possible:
// - The multiplying/dividing/pruning/mixing can all be done using bitwise operations. This makes
// their "pseudorandom" sequences easy to analyze, and I suspect that the outcome in terms of the
// step differences has some nice behaviour.

#[derive(Clone, Copy)]
struct Secret {
    num: usize,
}

impl Secret {
    fn result_1(&self) -> usize { self.num * 64 }
    fn result_2(&self) -> usize { self.num / 32 }
    fn result_3(&self) -> usize { self.num * 2048 }

    fn mix(&mut self, result: usize) -> &mut Self {
        self.num = result ^ self.num;
        self
    }

    fn prune(&mut self) -> &mut Self { 
        self.num = self.num % 16777216;
        self
    }

    fn next_step_diff(&self) -> i8 {
        let next_secret = make_next_secret(self.clone());
        (next_secret.num % 10) as i8 - (self.num % 10) as i8
    }
}

fn make_next_secret(mut secret: Secret) -> Secret {
    secret.mix(secret.result_1()).prune();
    secret.mix(secret.result_2()).prune();
    secret.mix(secret.result_3()).prune();

    secret
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let before = Instant::now();

    let secret_nums: Vec<Secret> = textdata.lines()
        .map(|line| Secret { num: line.parse::<usize>().unwrap() } )
        .collect();

    let secret_sum: usize = secret_nums.iter()
        .map(|&secret| (0..2000).fold(secret, |val, _| make_next_secret(val)))
        .map(|Secret { num } | num)
        .sum();

    let after = before.elapsed();
    println!("(Part  I) secret sum: {secret_sum}");
    println!("(Part  I) time elapsed: {after:2?}");
    let before = Instant::now();

    let price_and_diff: Vec<Vec<(u8, i8)>> = secret_nums.iter()
        .map(|&secret| (0..2000)
            .scan(secret, |curr_secret, _| {
                let val = (curr_secret.num % 10) as u8;
                let diff = curr_secret.next_step_diff();
                *curr_secret = make_next_secret(*curr_secret);
                Some((val, diff))
            })
            .collect()
        )
        .collect();

    // another approach using array as memory; hashmaps/hashsets are too slow!
    let mut price_sums: [[[[u16; 19]; 19]; 19]; 19] = [[[[0; 19]; 19]; 19]; 19];
    for prices in price_and_diff {
        let mut history: [[[[bool; 19]; 19]; 19]; 19] = [[[[false; 19]; 19]; 19]; 19];
        for loc in 4..2000 {
            // range is from -9 into +9
            let seq: [usize; 4] = core::array::from_fn(|i| (prices[loc-4+i].1 + 9) as usize);
            if !history[seq[0]][seq[1]][seq[2]][seq[3]] {
                price_sums[seq[0]][seq[1]][seq[2]][seq[3]] += prices[loc].0 as u16;
                history[seq[0]][seq[1]][seq[2]][seq[3]] = true;
            }
        }
    }
    let max_price: u16 = price_sums.into_iter().map(
        |array2| array2.into_iter().map(
            |array3| array3.into_iter().map(
                |array4| array4.into_iter().max().unwrap())
            .max().unwrap())
        .max().unwrap()).max().unwrap();

    let after = before.elapsed();
    println!("(Part II) max price: {max_price}");
    println!("(Part II) time elapsed: {after:2?}");
}

#[test]
fn small_test() {

    let secret_nums: Vec<Secret> = Vec::from([
        Secret{ num: 1 },
        Secret{ num: 10 },
        Secret{ num: 100 },
        Secret{ num: 2024 },
    ]);

    let secret_sum: usize = secret_nums.into_iter()
        .map(|secret| (0..2000).fold(secret, |val, _| make_next_secret(val)))
        .map(|Secret { num } | num)
        .sum();

    assert_eq!(37327623, secret_sum);
}

#[test]
fn test_mix() {
    let mut secret = Secret{ num: 42 };
    secret.mix(15);
    assert_eq!(37, secret.num)
}

#[test]
fn test_prune() {
    let mut secret = Secret{ num: 100000000 };
    secret.prune();
    assert_eq!(16113920, secret.num)
}

#[test]
fn test_diff() {
    let price_changes: Vec<i8> = (0..9) .scan( Secret{num:123}, |curr_secret, _| {
            let diff = curr_secret.next_step_diff();
            *curr_secret = make_next_secret(*curr_secret);
            Some(diff)
        })
        .collect();
    assert_eq!(vec![-3,6,-1,-1,0,2,-2,0,-2], price_changes)
}

