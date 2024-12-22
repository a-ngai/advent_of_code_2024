use std::fs;

// Part II possible issues:
// - infinite loop

struct Register {
    a: u64,
    b: u64,
    c: u64,
}

fn combo(operand: u8, registers: &Register) -> u64 {
    match operand {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => registers.a,
        5 => registers.b,
        6 => registers.c,
        7 => panic!("reserved, is not a combo!"),
        other => panic!("combo operand ({other}) should not appear!")
    }
}

fn initialize_computer(textdata: &String) -> (usize, Register, Vec<u8>) {
    let mut text_iter = textdata.lines();

    let reg_a_text: &str = text_iter.next().unwrap();
    let reg_b_text: &str = text_iter.next().unwrap();
    let reg_c_text: &str = text_iter.next().unwrap();
    text_iter.next();
    let program_text: &str = text_iter.next().unwrap();

    let reg_a: u64 = reg_a_text.split_whitespace().last().unwrap().parse::<u64>().unwrap();
    let reg_b: u64 = reg_b_text.split_whitespace().last().unwrap().parse::<u64>().unwrap();
    let reg_c: u64 = reg_c_text.split_whitespace().last().unwrap().parse::<u64>().unwrap();
    let reg: Register = Register { a: reg_a, b: reg_b, c: reg_c };
    let pointer: usize = 0;
    let program: Vec<u8> = program_text.split_whitespace().last().unwrap().split(",").map(|string| string.parse::<u8>().unwrap()).collect();
    (pointer, reg, program)
}

fn operation(opcode: u8, operand: u8, reg: &mut Register, pointer: &mut usize) -> Option<u64> {
    let result = match opcode {
        0 => {
            reg.a = reg.a / (2u64.pow(combo(operand, reg) as u32));
            *pointer += 2;
            None
        },
        1 => {
            reg.b = reg.b ^ (operand as u64);
            *pointer += 2;
            None
        },
        2 => {
            reg.b = combo(operand, reg) % 8;
            *pointer += 2;
            None
        },
        3 => {
            if reg.a != 0 { *pointer = operand as usize }
            else { *pointer += 2 }
            None
        },
        4 => {
            reg.b = reg.b ^ reg.c;
            *pointer += 2;
            None
        },
        5 => {
            *pointer += 2;
            Some(combo(operand, &reg) % 8)
        },
        6 => {
            reg.b = reg.a / (2u64.pow(combo(operand, &reg) as u32));
            *pointer += 2;
            None
        },
        7 => {
            // reg.c = reg.a / (2u32.pow(combo(operand, &reg)));
            reg.c = reg.a >> combo(operand, &reg);
            *pointer += 2;
            None
        },
        other => panic!("opcode must satisfy 0 <= ({other}) < 8")
    };
    // println!("opcode: {opcode}, operand: {operand}, a: {:06b}, b: {:06b}, c: {:06b}", reg.a, reg.b, reg.c);
    result
}

fn reset(reg: &mut Register, pointer: &mut usize) {
    reg.a = 0;
    reg.b = 0;
    reg.c = 0;
    *pointer = 0;
}

fn main() {
    let filename: &str = "input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let (mut pointer, mut reg, program): (usize, Register, Vec<u8>) = initialize_computer(&textdata);
    let mut output: Vec<Option<u64>> = Vec::new();
    while pointer < program.len()  {
        let opcode = program[pointer];
        let operand = program[pointer+1];
        let result = operation(opcode, operand, &mut reg, &mut pointer);
        output.push(result);
    }

    let output_string: String = output.into_iter()
        .filter_map(|result| result)
        .map(|num| num.to_string())
        .collect::<Vec<String>>()
        .join(",");
    println!("(Part  I) program output: {output_string}");

    let program: [u8; 16] = [2,4,1,2,7,5,4,7,1,3,5,5,0,3,3,0]; // algorithm specific to program!
    let start_register_digits: u64  = 0;
    let start_depth = 15;

    let mut results: Vec<u64> = Vec::new();
    let mut stack: Vec<(u64, usize)> = Vec::from([(start_register_digits, start_depth),]);

    while let Some((register_digits, depth)) = stack.pop() {
        let program_digits: u8 = program[depth];
        // println!("depth: {depth:02}, program_digit: {program_digits:02}, register_digits: {register_digits:048b}");
        for test in 0..8u8 {
            let shift = test ^ 0b010;
            // println!("    shift: {shift}");
            //let shifted_register = register_digits >> (shift as usize + 3*depth);
            let shifted_register = ((register_digits >> 3*depth) | (test as u64)) >> shift ;
            let shifted: u8 = (shifted_register as u8) & 0b111;
            // println!("    register as u8: {:03b}", shifted);
            // println!("    test as u8:     {:03b}", test);
            // println!("    digit as u8:    {:03b}", program_digits);
            
            let digit_1 = (test ^ shifted) & 0b100; // a_{3m+2} XOR a_{3m+2+s}
            let digit_2 = (!(!test ^ shifted)) & 0b010; // a_{3m+1} XOR a_{3m+1+s}
            let digit_3 = (!(test ^ shifted)) & 0b001; // a_{3m} NXOR a_{3m+s}
            let test_compare: u8 = digit_1 | digit_2 | digit_3 ;
            // println!("    result:         {:03b}", test_compare);

            if test_compare == program_digits {
                let next_register_digits: u64 = ((test as u64) << (3*depth)) | register_digits;
                if depth == 0 {
                    results.push(next_register_digits);
                    continue
                }
                let next_depth = depth - 1;
                stack.push((next_register_digits, next_depth));
                // println!("    -> good combination");
            }
        }
        // panic!("stop after comparing first digit {program_digits}");
    }
    println!("number of possibilities: {}", results.len());
    println!("smallest possibility: {}", results.iter().min().unwrap());
}

#[test]
#[ignore]
fn small_test() {
    let filename: &str = "test_input.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let (mut pointer, mut reg, program): (usize, Register, Vec<u8>) = initialize_computer(&textdata);
    let mut output: Vec<Option<u64>> = Vec::new();
    while pointer < program.len()  {
        let opcode = program[pointer];
        let operand = program[pointer+1];
        let result = operation(opcode, operand, &mut reg, &mut pointer);
        output.push(result);
    }

    let output_string: String = output.into_iter().filter_map(|result| result).map(|num| num.to_string()).collect::<Vec<String>>().join(",");
    assert_eq!("4,6,3,5,6,3,5,2,1,0", output_string);
}

#[test]
#[ignore]
fn test_part_two() {
    let filename: &str = "test_input_part_two.txt";
    let textdata: String = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Cannot read file ({filename})"));

    let (mut pointer, mut reg, program): (usize, Register, Vec<u8>) = initialize_computer(&textdata);
    reg.a = 117440;
    let mut output: Vec<u8> = Vec::new();
    while pointer < program.len()  {
        let opcode = program[pointer];
        let operand = program[pointer+1];
        let result = operation(opcode, operand, &mut reg, &mut pointer);

        match result {
            Some(val) => {
                //if val >= 8 { break }
                output.push(val as u8);
            },
            None => (),
        }
    }
    let output_string: String = output.iter().map(|&num| num.to_string()).collect::<Vec<String>>().join(",");
    assert_eq!(program.into_iter().map(|num| num.to_string()).collect::<Vec<String>>().join(","), output_string);
}

#[test]
#[ignore]
fn algorithm_identity_test() {

    let program: Vec<u8> = "2,4,1,2,7,5,4,7,1,3,5,5,0,3,3,0".split(",").map(|item| item.parse::<u8>().unwrap()).collect();
    let mut pointer = 0;
    let mut reg = Register { a : u64::from_str_radix("010101", 2).unwrap(), b: 0, c: 0};
    let mut output: Vec<Option<u64>> = Vec::new();
    while pointer < program.len()  {
        let opcode = program[pointer];
        let operand = program[pointer+1];
        let result = operation(opcode, operand, &mut reg, &mut pointer);
        output.push(result);
    }

    let output_string: String = output.into_iter().filter_map(|result| result).map(|num| num.to_string()).collect::<Vec<String>>().join(",");

    assert_eq!("4,1", output_string);
}

#[test]
fn smaller_test() {
    let program: [u8; 16] = [2,4,1,2,7,5,4,7,1,3,5,5,0,3,3,0]; // algorithm specific to program!
    let mut reg = Register { a: 37221274271216, b: 0, c: 0 };

    let mut output: Vec<Option<u64>> = Vec::new();
    let mut pointer = 0;
    while pointer < program.len()  {
        let opcode = program[pointer];
        let operand = program[pointer+1];
        let result = operation(opcode, operand, &mut reg, &mut pointer);
        output.push(result);
    }
        

    let output_string: String = output.into_iter()
        .filter_map(|result| result)
        .map(|num| num.to_string())
        .collect::<Vec<String>>()
        .join(",");
    println!("program output: {output_string}");
}

