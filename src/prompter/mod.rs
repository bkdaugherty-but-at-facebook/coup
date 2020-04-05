use crate::action::Action;
use crate::{GameState, Identity};
use anyhow::{anyhow, Result};
use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;

// Defines a temporary struct used to model a players state when prompted
pub struct PromptInfo<'a> {
    public_state: Option<&'a GameState>,
    player_hand: Vec<Identity>,
}

const YES: &[&str] = &[
    "yes",
    "yep",
    "y",
    "ya",
    "yer",
    "yar",
    "yessir",
    "yah",
    "yeah",
    "yup",
    "yupperino",
    "yy",
    "yyy",
];
const NO: &[&str] = &[
    "no",
    "nope",
    "n",
    "nah",
    "no fucking way",
    "nada",
    "no thank you",
    "no way jose",
    "nein",
    "niet",
    "nn",
    "nnn",
    "nah guy",
];

pub trait Prompter {
    fn prompt_player(&self, state: Option<&GameState>) -> Result<String>;
    fn prompt_player_choice<T: Display + FromStr + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
        state: Option<&GameState>,
    ) -> Result<T>;
    fn prompt_player_for_action(
        &self,
        question: &str,
        possible_actions: Vec<Action>,
        state: &GameState,
    ) -> Result<Action>;
    fn prompt_player_yes_no(&self, question: &str, state: Option<&GameState>) -> Result<bool>;
    // TODO --> Decide where you want thi
    // Should this just be on game state? Or a utility function?
    fn display_action(state: &GameState, action: Action) -> String {
        match action.clone() {
            Action::Assassinate(target) | Action::Coup(target) => {
                format!("{} {}", action.clone(), state.get_player_name(&target))
            }
            Action::Steal(target) => format!("{} from {}", action.clone(), state.get_player_name(&target)),
            _ => format!("{}", action),
        }
    }
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
        let response = self.get_response();
        // TODO flip this?
        match response {
            Ok(response) => {
                let response_value = &response.to_lowercase()[0..response.len()];
                match state {
                    Some(state_value) => match response_value {
                        "show" => {
                            println!("{}", state_value);
                            self.prompt_player(state)
                        }
                        _ => Ok(response),
                    },
                    None => Ok(response),
                }
            }
            Err(e) => Err(e),
        }
    }

    // TODO this is garbage
    // This could take an acceptance function as a new parameter.
    fn prompt_player_choice<T: Display + FromStr + Clone>(
        &self,
        question: &str,
        possible_choices: Vec<T>,
        state: Option<&GameState>,
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

    // Generic was just too tough :(
    fn prompt_player_for_action(
        &self,
        question: &str,
        possible_choices: Vec<Action>,
        state: &GameState,
    ) -> Result<Action> {
	println!("{}", question);
        println!("Choices are: [");
        for (idx, choice) in possible_choices.iter().enumerate() {
            println!("\t{} => {}", idx, LocalPrompter::display_action(state, choice.clone()));
        }
        println!(" ]");
        let choice = match self.prompt_player(Some(state)) {
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
        println!("{} (y/n)", question);
        let choice = self.prompt_player(state);
        match choice {
            Ok(choice) => {
                let choice_value = &choice[0..choice.len()];
                if YES.to_vec().contains(&choice_value) {
                    Ok(true)
                } else if NO.to_vec().contains(&choice_value) {
                    Ok(false)
                } else {
                    panic!("No response!")
                }
            }
            Err(e) => panic!("oh god"),
        }
    }
}
