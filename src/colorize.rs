use std::string::String;

/// Trait to colorize strings for console output
pub trait Colorize {
    fn warning(self) -> String;
    fn error(self) -> String;
    fn info(self) -> String;
    fn success(self) -> String;
    fn update(self) -> String;
}

impl Colorize for String {
    fn warning(mut self) -> String {
        self = "\x1b[1;33m".to_string() + &self + "\x1b[0m";
        self
    }
    fn error(mut self) -> String {
        self = "\x1b[1;31m".to_string() + &self + "\x1b[0m";
        self
    }
    fn info(mut self) -> String {
        self = "\x1b[0;34m".to_string() + &self + "\x1b[0m";
        self
    }
    fn success(mut self) -> String {
        self = "\x1b[1;32m".to_string() + &self + "\x1b[0m";
        self
    }
    fn update(mut self) -> String {
        self = "\x1b[1A\r".to_string() + &self;
        self
    }
}

impl Colorize for &str {
    fn warning(self) -> String {
        let result = self.to_string();
        result.warning()
    }
    fn error(self) -> String {
        let result = self.to_string();
        result.error()
    }
    fn info(self) -> String {
        let result = self.to_string();
        result.info()
    }
    fn success(self) -> String {
        let result = self.to_string();
        result.success()
    }
    fn update(self) -> String {
        let result = self.to_string();
        result.update()
    }
}
