use crate::{Action, GameState, Identity, PlayerID};
use crate::player::traits::Player;
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct RandomPlayer {
    // Not necessarily two?
    id: PlayerID,
    hand: Vec<Identity>,
}

impl RandomPlayer {
    pub fn new(id: PlayerID) -> Self {
        let hand = Vec::new();
        RandomPlayer { id, hand }
    }
    fn choose_random<T: Clone>(options: &mut [T] ) -> T {
	let mut rng = rand::thread_rng();
	options.shuffle(&mut rng);
	return options[0].clone();
    }
}

impl Player for RandomPlayer {
    fn choose_action(&self, state: &GameState) -> Action {
	// TODO -> Lazy static a rng --> Make a rand module
	let mut rng = rand::thread_rng();
	// TODO -> Choose target randomly -> Then refactor choose_forced_coup
	let target = self.choose_forced_coup(state);
	let mut available_actions = vec!(Action::Income, Action::ForeignAid,  Action::Tax, Action::Steal(target.clone()));
	let num_coins = self.count_coins(state);

	// More constants
	if num_coins >= 3 {
	    available_actions.push(Action::Assassinate(target.clone()));
	}

	if num_coins >= 7 {
	    available_actions.push(Action::Coup(target.clone()));
	}

	let num_actions = available_actions.len();
	let mut options = &mut available_actions[0..num_actions];
	return RandomPlayer::choose_random(options);
    }
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
	
        false
    }
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> Option<Action> {
	None
    }
    // Index in hand to replace
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> Option<usize> {
	None
    }

    fn choose_card_to_lose(&self, state: &GameState) -> usize {
	0
    }

    fn choose_forced_coup(&self, state: &GameState) -> PlayerID {
	for player_id in &state.active_players {
            if player_id != self.who_am_i() {
                return player_id.clone();
            }
        }
        panic!("No other players to coup!");
    }

    
    fn get_hand(&self) -> Vec<Identity> {
        self.hand.iter().cloned().collect()
    }

    fn set_hand(&mut self, hand: Vec<Identity>) {
        self.hand = hand;
    }

    fn who_am_i(&self) -> &PlayerID {
        &self.id
    }
    
}
