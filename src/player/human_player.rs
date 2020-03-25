use crate::{Action, GameState, Identity, PlayerID};
use crate::player::traits::Player;
use anyhow::{anyhow, Result};
use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use std::fmt::Display;

pub struct HumanPlayer {
    // Not necessarily two?
    id: PlayerID,
    prompter: LocalPrompter,
    name: String,
    hand: Vec<Identity>,
}

pub trait Prompter {
    fn prompt_player(&self, question: &str) -> Result<String>;
    fn prompt_player_choice<T: Display + FromStr + Clone>(&self, question: &str, possible_choices: Vec<T>) -> Result<T>;
}

struct LocalPrompter {}

impl LocalPrompter {
    fn new() -> Self {
	LocalPrompter{}
    }
    
    fn get_response(&self) -> Result<String> {
	let mut response = String::new();
	let _ = stdout().flush();
	stdin().read_line(&mut response).expect("Did not enter a correct string");
	if let Some('\n')= response.chars().next_back() {
            response.pop();
	}
	if let Some('\r')= response.chars().next_back() {
            response.pop();
	}
	Ok(response)
    }
}

impl Prompter for LocalPrompter {
    fn prompt_player(&self, question: &str) -> Result<String> {
	print!("{}", question);
	self.get_response()
    }

    fn prompt_player_choice<T: Display + FromStr + Clone>(&self, question: &str, possible_choices: Vec<T>) -> Result<T> {
	println!("{}", question);
	println!("Choices are: ");
	for choice in possible_choices {
	    print!("{}", choice);
	}
	println!("");
	match self.prompt_player(question) {
	    Ok(response) => {
		match T::from_str(&response) {
		    Ok(response) => Ok(response),
		    Err(e) => Err(anyhow!("Unable to convert {} ", response))
		}
	    },
	    Err(e) => Err(e)
	}
    }
}


impl HumanPlayer {
    pub fn new(id: PlayerID) -> Self {
        let hand = Vec::new();
	let prompter = LocalPrompter::new();
	let name = prompter.prompt_player("Please enter your name: ").unwrap();
	HumanPlayer { id, name, prompter, hand}
    }


    // Need to impl Player now!
    // Needs to be recursive if fail
    pub fn choose_action(&self, state: &GameState) -> Action {
	let available_actions = self.get_available_actions(state);
	for action in available_actions {
	    println!("{:?}", action)
	}
	Action::Income
    }
}



/*impl Player for HumanPlayer {
    fn choose_action(&self, state: &GameState) -> Action;
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> usize;

    // Utility functions on player state
    fn get_hand(&self) -> Vec<Identity>;
    fn set_hand(&mut self, hand: Vec<Identity>);
    fn who_am_i(&self) -> &PlayerID;
    fn discard_identity(&mut self, state: &GameState) -> Identity;
}*/

