use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use std::fmt::Display;
use anyhow::{anyhow, Result};
use crate::{GameState};

pub trait Prompter {
    fn prompt_player(&self, state: Option<&GameState>) -> Result<String>;
    fn prompt_player_choice<T: Display + FromStr + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
	state: Option<&GameState>
    ) -> Result<T>;
    fn prompt_player_choice_index<T: Display  + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
	state: Option<&GameState>
    ) -> Result<T>;
    fn prompt_player_yes_no(&self, question: &str, state: Option<&GameState>) -> Result<bool>;
}


pub struct LocalPrompter {}

impl LocalPrompter {
    pub fn new() -> Self {
        LocalPrompter {}
    }

    fn get_response(&self) -> Result<String> {
        let mut response = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut response)
            .expect("Did not enter a correct string");
        if let Some('\n') = response.chars().next_back() {
            response.pop();
        }
        if let Some('\r') = response.chars().next_back() {
            response.pop();
        }
        Ok(response)
    }
}

impl Prompter for LocalPrompter {
    fn prompt_player(&self, state: Option<&GameState>) -> Result<String> {
        self.get_response()
    }

    // TODO this is garbage
    // This could take an acceptance function as a new parameter.
    fn prompt_player_choice<T: Display + FromStr + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
	state: Option<&GameState>
    ) -> Result<T> {
        println!("{}", question);
        print!("Choices are: [");
        for choice in possible_choices {
            print!(" {}", choice);
        }
        println!(" ]");
        match self.prompt_player(state) {
            Ok(response) => match T::from_str(&response) {
                Ok(response) => Ok(response),
                Err(e) => Err(anyhow!("Unable to convert {} ", response)),
            },
            Err(e) => Err(e),
        }
    }
    fn prompt_player_choice_index<T: Display + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
	state: Option<&GameState>
    ) -> Result<T> {
        println!("Choices are: [");
        for (idx, choice) in possible_choices.iter().enumerate() {
            println!("\t{} ({})", choice, idx);
        }
        println!(" ]");
	println!("{}", question);
        let choice = match self.prompt_player(state) {
            Ok(response) => match usize::from_str(&response) {
                Ok(response) => Ok(response),
                Err(e) => Err(anyhow!("Unable to convert {} ", response)),
            },
            Err(e) => Err(e),
        };
        match choice {
            Ok(choice_idx) if choice_idx < possible_choices.len() => {
                Ok(possible_choices[choice_idx].clone())
            }
            _ => Err(anyhow!("invalid choice {:?}", choice)),
        }
    }
    fn prompt_player_yes_no(&self, question: &str, state: Option<&GameState>) -> Result<bool> {
        let choice = self
            .prompt_player_choice(question, vec!["Yes".to_string(), "No".to_string()], state);
        match choice {
            Ok(choice) => match choice {
                _ if choice == "Yes" => Ok(true),
                _ if choice == "No" => Ok(false),
                _ => panic!("No response!"),
            },
            Err(e) => panic!("oh god"),
        }
    }
}
