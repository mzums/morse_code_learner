use std::{
    collections::VecDeque,
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};
use rand::{seq::SliceRandom, rng};
use directories::ProjectDirs;
use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppConfig {
    difficulty_level: u8,
    known_chars: Vec<char>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            difficulty_level: 1,
            known_chars: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LearningSession {
    timestamp: String,
    chars_practiced: Vec<char>,
    difficulty: u8,
}

#[derive(Debug)]
struct ProgressionSystem {
    levels: Vec<ProgressionLevel>,
}

#[derive(Debug)]
struct ProgressionLevel {
    level: u8,
    chars_to_learn: Vec<char>,
    speed_requirement: f32,
    accuracy_requirement: f32,
}

struct MorseTutor {
    config: AppConfig,
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

impl MorseTutor {
    fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let progression = ProgressionSystem::new();
        
        let mut app = MorseTutor {
            config: config.clone(),
            progression,
            practice_queue: VecDeque::new(),
            session_start: Instant::now(),
            correct_answers: 0,
            total_answers: 0,
        };
        
        app.generate_practice_queue();
        app
    }

    fn generate_practice_queue(&mut self) {
        let mut rng = rng();
        let mut chars = self.config.known_chars.clone();
        chars.shuffle(&mut rng);
        
        if let Some(level) = self.progression.levels.iter().find(|l| l.level == self.config.difficulty_level) {
            for c in &level.chars_to_learn {
                if !chars.contains(c) {
                    chars.push(*c);
                }
            }
        }
        
        self.practice_queue.clear();
        for c in &chars {
            self.practice_queue.push_back(*c);
        }
    }

    fn end_session(&mut self) {
        if let Err(e) = self.config.save() {
            eprintln!("Error saving configuration: {}", e);
        }

        self.update_progression();
    }

    fn practice_char(&mut self, c: char) -> bool {
        let morse_code = Self::char_to_morse(c).unwrap_or("");
        println!("\n--- New char ---");
        println!("Level: {} | 'Exercises left': {}", 
            self.config.difficulty_level,
            self.practice_queue.len()
        );
        println!("Character: {}", c);
        
        print!("Your Morse code: {} (press Enter to submit): ", morse_code);
        io::stdout().flush().unwrap();
        
        let start_time = Instant::now();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input");
        let response_time = start_time.elapsed().as_secs_f32();
        
        let input = input.trim().to_uppercase();
        let correct = input == morse_code;
        
        self.total_answers += 1;
        
        if correct {
            self.correct_answers += 1;
            println!("✓ Correct! (time: {:.1}s)", response_time);
        } else {
            println!("✗ Incorrect! Correct code: {} (your: {})", morse_code, input);
        }
        
        correct
    }

    fn char_to_morse(c: char) -> Option<&'static str> {
        MORSE_MAPPING.iter()
            .find(|(ch, _)| *ch == c.to_ascii_uppercase())
            .map(|(_, code)| *code)
    }

    fn start_session(&mut self) {
        println!("\nNew session started!");
        println!("Difficulty level: {}", self.config.difficulty_level);
        println!("Characters to learn: {}", self.config.known_chars.iter().collect::<String>());
        println!("Exercise number: {}", self.practice_queue.len());
        println!("------------------------------------------------");

        self.session_start = Instant::now();
        self.correct_answers = 0;
        self.total_answers = 0;
    }

    fn run(&mut self) {
        self.start_session();       
        while let Some(&current_char) = self.practice_queue.front() {
            let correct = self.practice_char(current_char);
            
            if correct {
                self.practice_queue.pop_front();
            } else {
                if let Some(c) = self.practice_queue.pop_front() {
                    self.practice_queue.push_back(c);
                }
            }
            
            print!("Press 'q' to quit or Enter to continue: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Error reading input");
            
            if input.trim().eq_ignore_ascii_case("q") {
                println!("\nSession interrupted");
                break;
            }
        }
        
        self.end_session();
    }

    fn update_progression(&mut self) {
        let current_level = self.config.difficulty_level;
        if let Some(level) = self.progression.levels.iter().find(|l| l.level == current_level) {
            let accuracy = if self.total_answers > 0 {
                self.correct_answers as f32 / self.total_answers as f32
            } else {
                0.0
            };
            
            println!("\nLevel requirements {}:", current_level);
            println!("- Accuracy: {:.1}% (required: {:.1}%)", 
                accuracy * 100.0, level.accuracy_requirement * 100.0);

            if accuracy >= level.accuracy_requirement {
                self.config.difficulty_level += 1;
                println!("\n🎉 Next level: {}!", self.config.difficulty_level);
                
                if let Some(next_level) = self.progression.levels.iter().find(|l| l.level == self.config.difficulty_level) {
                    for c in &next_level.chars_to_learn {
                        if !self.config.known_chars.contains(c) {
                            self.config.known_chars.push(*c);
                            println!("+ New character added: {}", c);
                        }
                    }
                }
                
                self.generate_practice_queue();
            } else {
                println!("\nℹ️ Continue on current level.");
            }

            if let Err(e) = self.config.save() {
                eprintln!("Error saving configuration: {}", e);
            }
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "MorseTutor", "Morse Tutor") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            toml::from_str(&data).map_err(|e| e.into())
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        println!("Saving config to: {:?}", path);
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = toml::to_string_pretty(self)?;
        fs::write(&path, data)?;
        
        println!("Config saved");
        Ok(())
    }
}

impl ProgressionSystem {
    fn new() -> Self {
        let levels = vec![
            ProgressionLevel {
                level: 1,
                chars_to_learn: vec!['E', 'T'],
                speed_requirement: 5.0,
                accuracy_requirement: 0.8,
            },
            ProgressionLevel {
                level: 2,
                chars_to_learn: vec!['A', 'I', 'M', 'N'],
                speed_requirement: 4.0,
                accuracy_requirement: 0.85,
            },
            ProgressionLevel {
                level: 3,
                chars_to_learn: vec!['D', 'G', 'K', 'O', 'R', 'S', 'U', 'W'],
                speed_requirement: 3.5,
                accuracy_requirement: 0.9,
            },
            ProgressionLevel {
                level: 4,
                chars_to_learn: vec!['B', 'C', 'F', 'H', 'J', 'L', 'P', 'Q', 'V', 'X', 'Y', 'Z'],
                speed_requirement: 3.0,
                accuracy_requirement: 0.95,
            },
            ProgressionLevel {
                level: 5,
                chars_to_learn: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
                speed_requirement: 2.5,
                accuracy_requirement: 0.95,
            },
        ];
        
        ProgressionSystem {
            levels
        }
    }
}

fn main() {
    println!("================================================");
    println!("               MORSE CODE LEARNER");
    println!("================================================");
    
    let mut app = MorseTutor::new();
    app.run();
}