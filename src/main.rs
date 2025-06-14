use std::{collections::VecDeque, io::{self, Write}};
use rand::{seq::SliceRandom, rng};


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

impl MorseTutor {
    fn new() -> Self {
        let progression = ProgressionSystem::new();
        
        let mut app = MorseTutor {
            progression,
            practice_queue: VecDeque::new(),
            correct_answers: 0,
            total_answers: 0,
        };
        
        app.generate_practice_queue();
        app
    }

    fn generate_practice_queue(&mut self) {
        let mut rng = rng();
        let mut chars = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 
            'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 
            'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            '0', '1', '2', '3', '4', '5', 
            '6', '7', '8', '9'
        ].to_vec();
        chars.shuffle(&mut rng);
        
        self.practice_queue = chars.into_iter().collect();
    }

    fn char_to_morse(c: char) -> Option<&'static str> {
        MORSE_MAPPING.iter()
            .find(|(ch, _)| *ch == c.to_ascii_uppercase())
            .map(|(_, code)| *code)
    }

    fn practice_char(&mut self, c: char) -> bool {
        let morse_code = Self::char_to_morse(c).unwrap_or("");
        println!("\n--- New charater ---");
        println!("Character: {}", c);
        
        print!("Morse code (use . and -): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input");
        
        let input = input.trim().to_uppercase();
        let correct = input == morse_code;
        
        self.total_answers += 1;
        
        if correct {
            self.correct_answers += 1;
            println!("✓ Correct!");
        } else {
            println!("✗ Incorrect! Correst code: {} (your: {})", morse_code, input);
        }
        
        correct
    }
}

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

    let mut app = MorseTutor::new();
    app.practice_char('A');
}
