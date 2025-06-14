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
}

struct MorseTutor {
    config: AppConfig,
    progression: ProgressionSystem,
    practice_queue: VecDeque<char>,
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
            eprintln!("Error saving config: {}", e);
        }
        
        self.update_progression();
    }

    fn practice_char(&mut self, c: char) -> bool {
        let morse_code = Self::char_to_morse(c).unwrap_or("");
        println!("\n--- New char ---");
        println!("Level: {} | Exercises left: {}", 
            self.config.difficulty_level,
            self.practice_queue.len()
        );
        println!("Character: {}", c);
        
        print!("Morse code (use . and -): ");
        io::stdout().flush().unwrap();
        
        let start_time = Instant::now();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading input");
        let response_time = start_time.elapsed().as_secs_f32();
        
        let input = input.trim().to_uppercase();
        let correct = input == morse_code;
        
        if correct {
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

    fn run(&mut self) {        
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
        self.config.difficulty_level += 1;
        println!("\n🎉 Next level {}!", self.config.difficulty_level);
        
        if let Some(next_level) = self.progression.levels.iter().find(|l| l.level == self.config.difficulty_level) {
            for c in &next_level.chars_to_learn {
                if !self.config.known_chars.contains(c) {
                    self.config.known_chars.push(*c);
                    println!("+ New char added: {}", c);
                }
            }
        }
        
        self.generate_practice_queue();
        if let Err(e) = self.config.save() {
            eprintln!("Error saving config: {}", e);
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
            },
            ProgressionLevel {
                level: 2,
                chars_to_learn: vec!['A', 'I', 'M', 'N'],
            },
            ProgressionLevel {
                level: 3,
                chars_to_learn: vec!['D', 'G', 'K', 'O', 'R', 'S', 'U', 'W'],
            },
            ProgressionLevel {
                level: 4,
                chars_to_learn: vec!['B', 'C', 'F', 'H', 'J', 'L', 'P', 'Q', 'V', 'X', 'Y', 'Z'],
            },
            ProgressionLevel {
                level: 5,
                chars_to_learn: vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'],
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