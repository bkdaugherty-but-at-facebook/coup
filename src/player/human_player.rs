use crate::{Action, GameState, Identity, PlayerID};
use crate::player::traits::Player;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use std::fmt::Display;

pub struct HumanPlayer {
    // Not necessarily two?
    id: PlayerID,
    name: String,
    hand: Vec<Identity>,
}

impl HumanPlayer {
    pub fn new(id: PlayerID) -> Self {
        let hand = Vec::new();
	let name = HumanPlayer::prompt_player("Please enter your name: ");
	HumanPlayer { id, name, hand}
    }

    fn prompt_player(question: &str) -> String {
	print!("{}", question);
	let mut response = String::new();
	let _ = stdout().flush();
	stdin().read_line(&mut response).expect("Did not enter a correct string");
	if let Some('\n')= response.chars().next_back() {
            response.pop();
	}
	if let Some('\r')= response.chars().next_back() {
            response.pop();
	}
	return response;
    }

    fn prompt_player_choice<T: Display + FromStr + Clone>(question: &str, possible_choices: Vec<T>) -> T {
	possible_choices[0].clone()
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

