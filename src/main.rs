use std::{
    collections::{VecDeque, HashMap},
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};
use rand::{seq::SliceRandom, rng};
use directories::ProjectDirs;
use serde_derive::{Serialize, Deserialize};
use serde::{Deserialize, Serialize};


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

#[derive(Debug, Serialize, Deserialize, Default)]
struct UserStats {
    sessions_completed: u32,
    chars_learned: u32,
    accuracy: f32,
    #[serde(serialize_with = "serialize_response_times")]
    #[serde(deserialize_with = "deserialize_response_times")]
    response_times: HashMap<char, f32>,
    session_history: Vec<LearningSession>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LearningSession {
    timestamp: String,
    duration: u32,
    chars_practiced: Vec<char>,
    accuracy: f32,
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
    stats: UserStats,
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

fn serialize_response_times<S>(
    map: &HashMap<char, f32>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string_map: HashMap<String, f32> = map
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
    string_map.serialize(serializer)
}

fn deserialize_response_times<'de, D>(
    deserializer: D,
) -> Result<HashMap<char, f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string_map = HashMap::<String, f32>::deserialize(deserializer)?;
    let char_map = string_map
        .into_iter()
        .map(|(k, v)| (k.chars().next().unwrap(), v))
        .collect();
    Ok(char_map)
}

impl MorseTutor {
    fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let stats = UserStats::load().unwrap_or_default();
        let progression = ProgressionSystem::new();
        
        let mut app = MorseTutor {
            config: config.clone(),
            stats,
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
        let duration = self.session_start.elapsed().as_secs() as u32;
        let accuracy = if self.total_answers > 0 {
            self.correct_answers as f32 / self.total_answers as f32
        } else {
            0.0
        };
        
        if let Some(session) = self.stats.session_history.last_mut() {
            session.duration = duration;
            session.accuracy = accuracy;
        }
        
        self.stats.sessions_completed += 1;
        self.stats.accuracy = (self.stats.accuracy * (self.stats.sessions_completed - 1) as f32 + accuracy) / 
                            self.stats.sessions_completed as f32;

        if let Err(e) = self.config.save() {
            eprintln!("Error saving configuration: {}", e);
        }

        if let Err(e) = self.stats.save() {
            eprintln!("Error saving stats: {}", e);
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
        self.stats.response_times.insert(c, response_time);

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
        self.stats.session_history.push(LearningSession {
            timestamp: chrono::Local::now().to_rfc3339(),
            duration: 0,
            chars_practiced: self.config.known_chars.clone(),
            accuracy: 0.0,
            difficulty: self.config.difficulty_level,
        });

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

            let avg_time = if !self.stats.response_times.is_empty() {
                self.stats.response_times.values().sum::<f32>() / 
                self.stats.response_times.len() as f32
            } else {
                0.0
            };
            
            println!("\nLevel requirements {}:", current_level);
            println!("- Accuracy: {:.1}% (required: {:.1}%)", 
                accuracy * 100.0, level.accuracy_requirement * 100.0);

            println!("- Average time: {:.1}s (required: {:.1}s)", 
                avg_time, level.speed_requirement);

            if avg_time <= level.speed_requirement && accuracy >= level.accuracy_requirement {
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

impl UserStats {
    fn stats_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "MorseTutor", "Morse Tutor") {
            proj_dirs.data_dir().join("stats.toml")
        } else {
            PathBuf::from("stats.toml")
        }
    }

    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::stats_path();
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            toml::from_str(&data).map_err(|e| e.into())
        } else {
            Ok(UserStats::default())
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::stats_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = toml::to_string(self)?;
        fs::write(path, data)?;
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
