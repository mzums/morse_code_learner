use std::{collections::VecDeque, time::Instant};


#[derive(Debug)]
struct ProgressionLevel {
    level: u8,
    chars_to_learn: Vec<char>,
    speed_requirement: f32,
    accuracy_requirement: f32,
}

#[derive(Debug)]
struct ProgressionSystem {
    levels: Vec<ProgressionLevel>,
}

struct MorseTutor {
    progression: ProgressionSystem,
    practice_queue: VecDeque<char>,
    session_start: Instant,
    correct_answers: u32,
    total_answers: u32,
}

const MORSE_MAPPING: [(char, &str); 36] = [
    ('A', ".-"), ('B', "-..."), ('C', "-.-."), ('D', "-.."), ('E', "."), ('F', "..-."),
    ('G', "--."), ('H', "...."), ('I', ".."), ('J', ".---"), ('K', "-.-"), ('L', ".-.."),
    ('M', "--"), ('N', "-."), ('O', "---"), ('P', ".--."), ('Q', "--.-"), ('R', ".-."),
    ('S', "..."), ('T', "-"), ('U', "..-"), ('V', "...-"), ('W', ".--"), ('X', "-..-"),
    ('Y', "-.--"), ('Z', "--.."), ('1', ".----"), ('2', "..---"), ('3', "...--"),
    ('4', "....-"), ('5', "....."), ('6', "-...."), ('7', "--..."), ('8', "---.."),
    ('9', "----."), ('0', "-----"),
];


impl ProgressionSystem {
    fn new() -> Self {
        let levels = vec![
            ProgressionLevel {
                level: 1,
                chars_to_learn: vec!['E', 'T'],
                speed_requirement: 5.0,
                accuracy_requirement: 0.7,
            },
            ProgressionLevel {
                level: 2,
                chars_to_learn: vec!['A', 'I', 'M', 'N'],
                speed_requirement: 4.0,
                accuracy_requirement: 0.75,
            },
            ProgressionLevel {
                level: 3,
                chars_to_learn: vec!['D', 'G', 'K', 'O', 'R', 'S', 'U', 'W'],
                speed_requirement: 3.5,
                accuracy_requirement: 0.8,
            },
            ProgressionLevel {
                level: 4,
                chars_to_learn: vec!['B', 'C', 'F', 'H', 'J', 'L', 'P', 'Q', 'V', 'X', 'Y', 'Z'],
                speed_requirement: 3.0,
                accuracy_requirement: 0.85,
            },
            ProgressionLevel {
                level: 5,
                chars_to_learn: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
                speed_requirement: 2.5,
                accuracy_requirement: 0.9,
            },
        ];
        
        ProgressionSystem {
            levels
        }
    }
}

fn main() {
    println!("================================================");
    println!("              MORSE CODE LEARNER");
    println!("================================================");

    ProgressionSystem::new();
    println!("{:?}", ProgressionSystem::new());
}
