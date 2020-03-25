

// A logger logs information about the game
pub trait Logger {
    // Used to log internal details
    fn log(&self, message: String);
    
    // Used to present information to the user
    fn present(&self, message: String);
}
