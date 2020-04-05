use crate::player::traits::Player;
use crate::prompter::{LocalPrompter, PromptInfo, Prompter};
use crate::{Action, GameState, Identity, PlayerID};
use anyhow::{anyhow, Result};

pub struct HumanPlayer<P: Prompter> {
    // Not necessarily two?
    id: PlayerID,
    prompter: P,
    hand: Vec<Identity>,
}

impl<P: Prompter> HumanPlayer<P> {
    pub fn new(id: PlayerID, prompter: P) -> Self {
        let hand = Vec::new();
        HumanPlayer { id, prompter, hand }
    }
}

impl<P: Prompter> Player for HumanPlayer<P> {
    fn choose_action(&self, state: &GameState) -> Action {
        let available_actions = self.get_available_actions(state);
        let action = self.prompter.prompt_player_for_action(
            "What will you do?",
            available_actions,
            PromptInfo {
                state,
                player_hand: self.get_hand(),
            },
        );
        match action {
            Ok(action) => action,
            Err(e) => {
                println!("Hm. I didn't get that...");
                self.choose_action(state)
            }
        }
    }

    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        let question = &format!(
            "Would you like to challenge {}'s {}?",
            state.get_player_name(player_id),
            // TODO fix this! I hate that I genericized this but then it doesn't work.
            // This should maybe be on game state?
            LocalPrompter::display_action(state, action.clone())
        );

        match self.prompter.prompt_player_yes_no(
            question,
            Some(PromptInfo {
                state,
                player_hand: self.get_hand(),
            }),
        ) {
            Ok(x) => x,
            Err(e) => {
                // To do --> errors handled in prompter?
                println!("Hm. I didn't get that.");
                self.will_challenge(state, player_id, action)
            }
        }
    }
    fn will_block(
        &self,
        state: &GameState,
        player_id: &PlayerID,
        action: &Action,
    ) -> Option<Action> {
        let possible_actions = action.blockable(self.who_am_i());
        match possible_actions {
            Some(actions) => {
                if actions.len() > 0 {
                    let prompt_info = PromptInfo {
                        state,
                        player_hand: self.get_hand(),
                    };
                    let human_readable_action =
                        LocalPrompter::display_action(state, action.clone());
                    let question = &format!(
                        "Would you like to block {}'s {}?",
                        state.get_player_name(player_id),
                        &human_readable_action,
                    );

                    if self.prompter.prompt_player_yes_no(question, Some(prompt_info.clone())).unwrap() {
                        let choice = self
                            .prompter
                            .prompt_player_for_action(
                                &format!("How will you block?"),
				actions,
                                prompt_info.clone(),
                            )
                            .unwrap();
                        Some(choice)
                    } else {
                        None
                    }
                } else {
		    None
		}
            }
            None => None,
        }
    }
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> Option<usize> {
        let prompt_info = Some(PromptInfo {
            state,
            player_hand: self.get_hand(),
        });

        if self
            .prompter
            .prompt_player_yes_no(
                &format!(
                    "You drew a {}. Would you like to exchange it with one of your current cards?",
                    card
                ),
                prompt_info,
            )
            .unwrap()
        {
            let prompt_info = Some(PromptInfo {
                state,
                player_hand: self.get_hand(),
            });
            let chosen_idx = self
                .prompter
                .prompt_player_choice("Which one?", self.get_hand(), prompt_info)
                .unwrap();
            Some(chosen_idx)
        } else {
            None
        }
    }
    fn choose_card_to_lose(&self, state: &GameState) -> usize {
        // TODO Don't give choice on one card
        let prompt_info = Some(PromptInfo {
            state,
            player_hand: self.get_hand(),
        });
        // TODO don't unwrap
        let chosen_idx = self
            .prompter
            .prompt_player_choice(
                "Which identity will you discard?",
                self.get_hand(),
                prompt_info,
            )
            .unwrap();
        chosen_idx
    }
    fn choose_forced_coup(&self, state: &GameState) -> PlayerID {
        let other_players = self.get_other_active_players(state);
        /*let prompt_info = Some(PromptInfo {
                state,
                player_hand: self.get_hand(),
            });
        let chosen_idx = self.prompter.prompt_player_choice("Which identity will you discard?", other_players, prompt_info);*/
        return other_players[0].clone();
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
