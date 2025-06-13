#[derive(Debug)]
struct ProgressionLevel {
    level: u8,
    chars_to_learn: Vec<char>,
    speed_requirement: f32,
    accuracy_requirement: f32,
}

const LEVELS_DATA: &[(u8, &[char], f32, f32)] = &[
    (1, &['E', 'T'], 10.0, 0.7),
    (2, &['A', 'I', 'M', 'N'], 9.0, 0.75),
    (3, &['D', 'G', 'K', 'O', 'R', 'S', 'U', 'W'], 8.5, 0.8),
    (4, &['B', 'C', 'F', 'H', 'J', 'L', 'P', 'Q', 'V', 'X', 'Y', 'Z'], 7.0, 0.85),
    (5, &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'], 7.0, 0.9),
];

fn get_levels() -> Vec<ProgressionLevel> {
    LEVELS_DATA
        .iter()
        .map(|(level, chars, speed, accuracy)| ProgressionLevel {
            level: *level,
            chars_to_learn: chars.to_vec(),
            speed_requirement: *speed,
            accuracy_requirement: *accuracy,
        })
        .collect()
}

fn main() {
    println!("================================================");
    println!("              MORSE CODE LEARNER");
    println!("================================================");

    let levels = get_levels();
    print!("{:?}", levels);
}