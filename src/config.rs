use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Config {
    ints: HashMap<String, isize>,
    floats: HashMap<String, f64>,
    strings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            ints: HashMap::new(),
            strings: HashMap::new(),
            floats: HashMap::new(),
        }
    }

    pub fn new_from_config_file<P: std::convert::AsRef<std::path::Path>>(
        path: P,
    ) -> std::io::Result<Config> {
        use std::fs;
        let s = fs::read_to_string(path)?;

        Ok(Config::new_from_config_string(&s))
    }

    pub fn new_from_config_string(s: &str) -> Config {
        let mut ints: HashMap<String, isize> = HashMap::new();
        let mut floats: HashMap<String, f64> = HashMap::new();
        let mut strings: HashMap<String, String> = HashMap::new();

        for (i, line) in s.lines().enumerate() {
            let trim_line = line.trim_start();
            // Ignore lines where the first non-whitespace char is `#` since that is reserved for comments
            if trim_line.get(0..1) != Some("#") {
                let parts = trim_line.split('=').collect::<Vec<&str>>();

                // TODO: Come up with error strategy (maybe callback)
                if parts.len() != 2 {
                    println!(
                        "Invalid format on line - too many sections - (skipping) {}: \"{}\"",
                        i, line
                    );
                    break;
                }

                let key = parts[0].trim();

                if key == "" {
                    println!(
                        "Invalid format on line - no key - (skipping) {}: \"{}\"",
                        i, line
                    );
                    break;
                }

                if let Ok(value) = parts[1].parse::<isize>() {
                    ints.insert(key.to_string(), value);
                } else if let Ok(value) = parts[1].parse::<f64>() {
                    floats.insert(key.to_string(), value);
                } else {
                    strings.insert(key.to_string(), parts[1].to_string());
                }
            }
        }

        Config {
            ints,
            floats,
            strings,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<isize> {
        self.ints.get(key).cloned()
    }

    // Will use this at some point, keeping for consistency. Although perhaps this can be moved out of this crate?
    #[allow(dead_code)]
    pub fn get_float<'a>(&'a self, key: &str) -> Option<&'a f64> {
        self.floats.get(key)
    }

    pub fn get_string<'a>(&'a self, key: &str) -> Option<&'a String> {
        self.strings.get(key)
    }
}
