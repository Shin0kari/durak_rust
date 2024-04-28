use std::{
    collections::{HashMap, HashSet},
    io,
    time::Instant,
};

use rand::Rng;

type Id = u8;

struct GameData {
    who_attacker: Id,
    who_defender: Id,
    attacker_deck: AttackerDeck,
    defender_deck: DefenderDeck,
    deck_of_cards: DeckCards,
    players_data: HashMap<Id, PlayerData>,
}

// нападающий
#[derive(Debug, Clone)]
struct AttackerDeck {
    attacking_cards: HashMap<u8, Card>,
}

////////
#[derive(Debug, Clone)]
struct PlayerData {
    cards: HashMap<u8, Card>,
    bot: bool,
}

#[derive(Debug, Clone)]
struct Card {
    suit: char,
    trump: bool,
}

// отбивающийся
#[derive(Debug, Clone)]
struct DefenderDeck {
    pairs_beaten_cards: HashMap<u8, u8>,
    defending_cards: HashMap<u8, Card>,
    how_much_cards_defender_have: u8,
}

#[derive(Debug, Clone)]
struct DeckCards {
    cards: HashMap<u8, Card>,
}

fn main() {
    let now = Instant::now();

    let attacker_deck = AttackerDeck {
        attacking_cards: HashMap::new(),
    };
    let defender_deck = DefenderDeck {
        defending_cards: HashMap::new(),
        pairs_beaten_cards: HashMap::new(),
        how_much_cards_defender_have: 0,
    };

    // hows the size of a deck of cards
    let vec_len = 36;
    let mut rand_vec_cards: Vec<u8> = vec![0; vec_len];
    // generating a random deck of cards
    rand_vec_cards = gen_rand_deck(rand_vec_cards);
    // println!("Rand_deck: {:?}", rand_vec_cards);

    // let suits = "♣♠♥♦".chars();
    let trump_volume = *rand_vec_cards.first().unwrap(); // last volume in array
    rand_vec_cards = rand_vec_cards[1..].to_vec(); // removing a trump card from the queue
    rand_vec_cards.push(trump_volume); // adding a trump card to the queue

    let div_from_volume = count_div(vec_len);
    let mut deck_of_cards = filled_card_deck(&rand_vec_cards, div_from_volume, trump_volume);

    let mut players_data: HashMap<u8, PlayerData> = HashMap::new();

    println!("Enter the number of players (up to 8 people): ");
    let mut chosen_num: u8 = read_console().parse::<u8>().unwrap();
    while chosen_num > 8 {
        println!("An incorrect number of players was entered.");
        println!("Enter the number of players (up to 8 people): ");
        chosen_num = read_console().parse::<u8>().unwrap();
    }

    for i in 1..=chosen_num {
        println!("Will player {} be a bot?(true/false): ", i);
        let is_bot: bool = read_console().parse::<bool>().unwrap();

        let new_player_data = gen_deck_in_hand(&deck_of_cards, is_bot);
        remove_cards(&mut deck_of_cards.cards, &new_player_data.cards);
        players_data.insert(i, new_player_data);
    }

    let deck_data = &mut GameData {
        who_attacker: 0,
        attacker_deck,
        defender_deck,
        deck_of_cards,
        players_data,
        who_defender: 0,
    };

    deck_data.who_attacker = who_is_first(deck_data.players_data.clone());
    deck_data.who_defender = deck_data.who_attacker + 1;
    deck_data.defender_deck.how_much_cards_defender_have = deck_data
        .players_data
        .clone()
        .get(&deck_data.who_defender)
        .unwrap()
        .cards
        .len() as u8;

    println!("\nLet`s start the game!\n");
    let mut win = false;
    let mut transfered_cards = Some(false);
    let mut def_cards = Some(false);
    let mut attack_cards = Some(false);
    let mut throw_bool = Some(false);
    let mut whose_turn: (u8, u8) = (1, deck_data.who_attacker);
    let mut who_last: u8 = deck_data.who_attacker;
    let mut chosen_cards: HashMap<u8, Card> = HashMap::new();

    while !win {
        let who_id = whose_turn.1;
        println!("Player {}'s turn", who_id);

        // 1 - attacker/thrower, 2 - defender, 3.. - throwers
        match whose_turn.0 {
            1 => {
                if let Some(player_data) = deck_data.players_data.get_mut(&who_id) {
                    // if !attack_cards.is_some_and(|check| !check) {           ???
                    if attack_cards.is_some_and(|check| !check) {
                        chose_cards(
                            &mut chosen_cards,
                            &mut player_data.cards,
                            &deck_data.defender_deck,
                            div_from_volume,
                        );
                        remove_cards(&mut player_data.cards, &chosen_cards.clone());
                        play_card(&mut deck_data.attacker_deck, &mut chosen_cards);
                        attack_cards = Some(true);
                    } else {
                        let mut allowed_throwable_cards = throwable_cards(
                            deck_data.attacker_deck.clone(),
                            deck_data.defender_deck.clone(),
                            player_data.clone(),
                            div_from_volume,
                        );

                        if !allowed_throwable_cards.is_empty() {
                            throw_bool = Some(throw_cards(
                                &mut deck_data.attacker_deck,
                                &deck_data.defender_deck,
                                player_data,
                                &mut allowed_throwable_cards,
                            ));
                        }
                    }
                };
            }
            2 => {
                if let Some(player_data) = deck_data.players_data.get_mut(&who_id) {
                    // auto remove cards, later I'll change the attacker to have autoremove too
                    println!(
                        "Attacker deck: {:?}",
                        deck_data.attacker_deck.attacking_cards
                    );
                    println!("Your cards: {:?}", player_data.cards.clone());

                    let transferrable_cards = get_transferrable_cards(
                        player_data.cards.clone(),
                        deck_data.attacker_deck.attacking_cards.clone(),
                        div_from_volume,
                    );

                    let check_transfer: bool = if deck_data.defender_deck.defending_cards.is_empty()
                        && !transferrable_cards.is_empty()
                    {
                        println!("Do you want to transfer cards to another player?(true/false): ");
                        read_console().parse::<bool>().unwrap()
                    } else {
                        false
                    };

                    if check_transfer {
                        transfered_cards = Some(transfer_cards(
                            &mut deck_data.attacker_deck,
                            player_data,
                            transferrable_cards,
                        ));
                    } else {
                        def_cards = Some(beat_cards(
                            &mut deck_data.attacker_deck,
                            &mut deck_data.defender_deck,
                            player_data,
                            div_from_volume,
                        ));
                    }
                }
            }
            3..=8 => {
                if let Some(player_data) = deck_data.players_data.get_mut(&who_id) {
                    let mut allowed_throwable_cards = throwable_cards(
                        deck_data.attacker_deck.clone(),
                        deck_data.defender_deck.clone(),
                        player_data.clone(),
                        div_from_volume,
                    );

                    if !allowed_throwable_cards.is_empty() {
                        throw_bool = Some(throw_cards(
                            &mut deck_data.attacker_deck,
                            &deck_data.defender_deck,
                            player_data,
                            &mut allowed_throwable_cards,
                        ));
                    }
                }
            }
            _ => panic!("PANICAAA!"),
        }

        let mut check_player = false;
        who_last = whose_turn.1;
        // attack_cards
        if whose_turn.0 == 1 {
            if attack_cards.is_some_and(|check| check) {
                while !check_player {
                    let mut def_id = whose_turn.1 + 1;
                    if def_id > chosen_num {
                        def_id %= chosen_num;
                    }
                    if def_id == whose_turn.1 {
                        win = true;
                        break;
                    }
                    if !deck_data
                        .players_data
                        .get(&def_id)
                        .unwrap()
                        .cards
                        .is_empty()
                    {
                        check_player = true;
                        deck_data.who_defender = def_id;
                        whose_turn = (2, def_id);
                    };
                }
            } else if throw_bool.is_some_and(|check| check) {
                while !check_player {
                    let mut def_id = whose_turn.1 + 1;
                    if def_id > chosen_num {
                        def_id %= chosen_num;
                    }
                    if !deck_data
                        .players_data
                        .get(&def_id)
                        .unwrap()
                        .cards
                        .is_empty()
                    {
                        check_player = true;
                        deck_data.who_defender = def_id;
                        whose_turn = (2, def_id);
                    };
                }
            } else {
                let mut next_thrower_id = if whose_turn.1 + 2 < 9 {
                    whose_turn.1 + 2
                } else {
                    (whose_turn.1 + 2) % chosen_num
                };

                while !check_player {
                    if !deck_data
                        .players_data
                        .get(&next_thrower_id)
                        .unwrap()
                        .cards
                        .is_empty()
                    {
                        check_player = true;
                        whose_turn = (3, next_thrower_id);
                    };
                    next_thrower_id += 1;
                    if next_thrower_id > chosen_num {
                        next_thrower_id %= chosen_num;
                    }
                }
            }
        }
        if whose_turn.0 == 2 {
            if transfered_cards.is_some_and(|check| check) {
                while !check_player {
                    let mut def_id = whose_turn.1 + 1;
                    if def_id > chosen_num {
                        def_id %= chosen_num;
                    }
                    if def_id == whose_turn.1 {
                        win = true;
                        break;
                    }
                    if !deck_data
                        .players_data
                        .get(&def_id)
                        .unwrap()
                        .cards
                        .is_empty()
                    {
                        check_player = true;
                        deck_data.who_attacker = whose_turn.1;
                        deck_data.who_defender = def_id;
                        whose_turn = (2, def_id);
                    };
                }
            } else if def_cards.is_some_and(|check| check) {
                while !check_player {
                    let mut attacker_id = whose_turn.1 - 1;
                    if attacker_id == 0 {
                        attacker_id = chosen_num;
                    };
                    if deck_data.players_data.get(&attacker_id).is_some() {
                        whose_turn = (1, attacker_id);
                        check_player = true;
                    } else if attacker_id == whose_turn.1 {
                        win = true;
                        check_player = true;
                    }
                }
            } else {
                while !check_player {
                    let mut attacker_id = whose_turn.1 + 1;
                    if attacker_id > chosen_num {
                        attacker_id %= chosen_num;
                    }
                    if attacker_id == whose_turn.1 {
                        win = true;
                        break;
                    }
                    if deck_data.players_data.get(&attacker_id).is_some() {
                        whose_turn = (1, attacker_id);
                        check_player = true;
                    }
                }
            }
        }
        if whose_turn.0 > 2 {
            if throw_bool.is_some_and(|check| check) {
                whose_turn = (2, deck_data.who_defender);
            } else {
                while !check_player {
                    let mut attacker_id = whose_turn.1 + 1;
                    if attacker_id > chosen_num {
                        attacker_id %= chosen_num;
                    }
                    if attacker_id == whose_turn.1 {
                        win = true;
                        break;
                    }
                    if deck_data.players_data.get(&attacker_id).is_some() {
                        whose_turn = (whose_turn.0 + 1, attacker_id);
                        check_player = true;
                    }
                }
            }
        }

        let mut filling_player_id = deck_data.who_attacker;
        if (transfered_cards.is_some_and(|check| check) || def_cards.is_some())
            && !deck_data.deck_of_cards.cards.is_empty()
        {
            if let Some(player_data) = deck_data.players_data.get_mut(&filling_player_id) {
                fill_player_deck(&mut deck_data.deck_of_cards, &mut player_data.cards);
                filling_player_id += 1;
                while filling_player_id == deck_data.who_attacker
                    && !deck_data.deck_of_cards.cards.is_empty()
                {
                    if let Some(player_data) = deck_data.players_data.get_mut(&filling_player_id) {
                        fill_player_deck(&mut deck_data.deck_of_cards, &mut player_data.cards);
                    }
                    filling_player_id += 1;
                }
            }
        }
    }

    println!("Durak is {}", who_last);
    let elapsed_time = now.elapsed();
    println!("Игра завершена за {} minutes", elapsed_time.as_secs() / 60);
}

// Replenishing the missing player cards
fn fill_player_deck(deck_of_cards: &mut DeckCards, player_cards: &mut HashMap<u8, Card>) {
    let num_cards_to_fill = {
        if deck_of_cards.cards.len() < 7 - player_cards.len() {
            deck_of_cards.cards.len()
        } else {
            7
        }
    };

    deck_of_cards
        .cards
        .iter()
        .take(num_cards_to_fill)
        .for_each(|(card_value, card_data)| {
            player_cards.insert(*card_value, card_data.clone());
        });

    let cards_id: HashSet<u8> = player_cards
        .keys()
        .filter(|card_value| deck_of_cards.cards.contains_key(card_value))
        .copied()
        .collect();

    cards_id.iter().for_each(|card_value| {
        deck_of_cards.cards.remove(card_value);
    });
}

// struct GameData {
//     who_attacker: Id,
//     who_defender: Id,
//     attacker_deck: AttackerDeck,
//     defender_deck: DefenderDeck,
//     deck_of_cards: DeckCards,
//     players_data: HashMap<Id, PlayerData>,
// }

// if no one has the lowest trump card in their hands
// then the player who goes first will be chosen randomly
fn who_is_first(players_data: HashMap<Id, PlayerData>) -> Id {
    // search by lowest trump card
    let lowest_id: Id = {
        let mut lowest_card = 55;
        let mut lowest_id = 0;
        let players_data_with_lowest_card = players_data
            .into_iter()
            .map(|(player_id, cards_data)| {
                (
                    player_id,
                    cards_data
                        .cards
                        .iter()
                        .filter(|(_card_value, card_data)| card_data.trump)
                        .map(|(card_value, _)| (*card_value))
                        .collect::<HashSet<u8>>(),
                )
            })
            .collect::<HashMap<Id, HashSet<u8>>>();

        if players_data_with_lowest_card.is_empty() {
            let max_num_players = players_data_with_lowest_card.keys().max().unwrap();

            let mut rng = rand::thread_rng();
            lowest_id = rng.gen_range(1..=(*max_num_players as i32)) as u8;
            lowest_id
        } else {
            players_data_with_lowest_card
                .into_iter()
                .for_each(|(id, cards)| {
                    cards.into_iter().for_each(|card| {
                        lowest_card = card;
                        lowest_id = id;
                    });
                });

            lowest_id
        }
    };

    lowest_id
}

fn check_duplicate_cards(player_cards: &HashMap<u8, Card>, div_from_volume: u8) -> bool {
    let mut check_duplicate_bool = false;
    let mut check_duplicate_vec: Vec<u8> = vec![0; div_from_volume as usize];

    player_cards.iter().for_each(|(card_value, _card_data)| {
        check_duplicate_vec[((card_value - 1) / div_from_volume) as usize] += 1
    });

    check_duplicate_vec.iter().for_each(|card_value| {
        if *card_value > 1 {
            check_duplicate_bool = true;
        }
    });

    check_duplicate_bool
}

fn get_playable_cards_with_duplicate(
    player_cards: &mut HashMap<u8, Card>,
    playable_cards: &mut HashMap<u8, Card>,
    div_from_volume: u8,
) {
    if !playable_cards.is_empty() {
        player_cards.iter().for_each(|(card_value, card_data)| {
            if (card_value - 1) / div_from_volume
                == (playable_cards.iter().clone().last().unwrap().0 - 1) / div_from_volume
            {
                playable_cards.insert(*card_value, card_data.clone());
            }
        })
    } else {
        playable_cards.extend(player_cards.clone())
    }
}

fn chose_cards(
    chosen_cards: &mut HashMap<u8, Card>,
    player_cards: &mut HashMap<u8, Card>,
    defending_deck: &DefenderDeck,
    div_from_volume: u8,
) -> bool {
    let mut num_throwable_cards = defending_deck.how_much_cards_defender_have
        - (defending_deck.pairs_beaten_cards.len() as u8);

    if num_throwable_cards == 0 {
        return false;
    }

    let check_duplicate = check_duplicate_cards(player_cards, div_from_volume);
    let mut playable_cards: HashMap<u8, Card> = HashMap::new();
    let mut chosen_card: u8;
    loop {
        if check_duplicate {
            get_playable_cards_with_duplicate(player_cards, &mut playable_cards, div_from_volume);

            let mut throw_more = true;

            while throw_more || num_throwable_cards > 0 {
                println!("You can play with these cards: {:?}", playable_cards);
                println!("Choose the card you want to throw: ");

                chosen_card = read_console().parse::<u8>().unwrap();

                if playable_cards.contains_key(&chosen_card) {
                    chosen_cards.insert(
                        chosen_card,
                        playable_cards.get(&chosen_card).unwrap().clone(),
                    );
                    playable_cards.remove(&chosen_card);
                    num_throwable_cards -= 1;
                    println!("Do you want to throw another card? (true/false)");
                    throw_more = read_console().parse::<bool>().unwrap_or_default();
                } else {
                    println!("You chose a card that you don't have");
                }
            }
            return true;
        } else {
            println!("Your cards: {:?}", player_cards);
            println!("Choose the card you want to throw: ");

            chosen_card = read_console().trim_end().parse::<u8>().unwrap();

            if player_cards.contains_key(&chosen_card) {
                let (k, v) = player_cards.get_key_value(&chosen_card).unwrap();
                chosen_cards.insert(*k, v.clone());

                return true;
            } else {
                println!("Card not found");
                continue;
            }
        }
    }
}

fn play_card(attacker_deck: &mut AttackerDeck, chosen_cards: &mut HashMap<u8, Card>) {
    chosen_cards
        .clone()
        .into_iter()
        .for_each(|(card_volume, card_data)| {
            attacker_deck
                .attacking_cards
                .insert(card_volume, card_data)
                .unwrap();
        });

    chosen_cards.clear();
}

fn throwable_cards(
    attacker_deck: AttackerDeck,
    defending_deck: DefenderDeck,
    players_data: PlayerData,
    div_from_volume: u8,
) -> HashMap<u8, Card> {
    let allowed_volume: HashSet<u8> = {
        let mut allowed_cards: HashMap<u8, Card> = HashMap::new();
        allowed_cards.extend(attacker_deck.attacking_cards);
        allowed_cards.extend(defending_deck.defending_cards);
        allowed_cards
    }
    .keys()
    .map(|card_volume| (card_volume % div_from_volume))
    .collect();

    players_data
        .cards
        .iter()
        .filter(|(card_volume, _card_data)| {
            allowed_volume.contains(&(*card_volume % div_from_volume))
        })
        .map(|(card_volume, card_data)| (*card_volume, card_data.clone()))
        .collect()
}

fn throw_cards(
    attacker_deck: &mut AttackerDeck,
    defending_deck: &DefenderDeck,
    players_data: &PlayerData,
    playable_cards: &mut HashMap<u8, Card>,
) -> bool {
    if players_data.bot {
        todo!()
    } else {
        let mut num_throwable_cards = defending_deck.how_much_cards_defender_have
            - (defending_deck.pairs_beaten_cards.len() as u8);

        if num_throwable_cards == 0 {
            return false;
        }
        println!("Your cards: {:?}", players_data.cards);
        println!("Would you like to throw some cards? (true/false)");

        let throw_bool: bool = read_console().parse::<bool>().unwrap_or_default();

        let mut chosen_cards: HashMap<u8, Card> = HashMap::new();

        let mut throw_more = true;

        while throw_bool || throw_more {
            println!("You can play with these cards: {:?}", playable_cards);
            println!("Choose the card you want to throw: ");

            let chosen_card: u8 = read_console().parse::<u8>().unwrap();

            if playable_cards.contains_key(&chosen_card) {
                chosen_cards.insert(
                    chosen_card,
                    playable_cards.get(&chosen_card).unwrap().clone(),
                );
                playable_cards.remove(&chosen_card);
                num_throwable_cards -= 1;
                println!("Do you want to throw another card? (true/false)");
                throw_more = read_console().parse::<bool>().unwrap_or_default();
            } else {
                println!("You chose a card that you don't have");
            }
        }

        if !chosen_cards.is_empty() {
            play_card(attacker_deck, &mut chosen_cards);
        }
        throw_bool
    }
}

fn read_console() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap_or_default();
    buffer.trim_end().to_string()
}

fn take_all_card(
    attacker_deck: &mut AttackerDeck,
    defending_deck: &mut DefenderDeck,
    players_data: &mut PlayerData,
) {
    attacker_deck
        .attacking_cards
        .iter()
        .for_each(|(card_volume, card_data)| {
            players_data.cards.insert(*card_volume, card_data.clone());
        });

    defending_deck
        .defending_cards
        .iter()
        .for_each(|(card_volume, card_data)| {
            players_data.cards.insert(*card_volume, card_data.clone());
        });

    attacker_deck.attacking_cards.clear();
    defending_deck.defending_cards.clear();
    defending_deck.pairs_beaten_cards.clear();
}

// имеется косяк, к примеру если вы не можете побить карты оппонента, то вы возьмёте
// все карты только в тот момент, когда наткнётесь на карту которую не можете побить
// т.е. вам не ссразу об этом скажет
fn beat_cards(
    attacker_deck: &mut AttackerDeck,
    defending_deck: &mut DefenderDeck,
    players_data: &mut PlayerData,
    div_from_volume: u8,
) -> bool {
    if players_data.bot {
        todo!()
    } else {
        let mut not_beat_bool = true;
        let mut def_cards = false;
        // let attacker_deck.attacking_cards = attacker_deck.attacking_cards.clone();
        while not_beat_bool {
            println!("Your cards: {:?}", players_data.cards);
            println!("Attacking cards: {:?}", attacker_deck.attacking_cards);
            println!("Choose the card you want to beat: ");

            let chosen_card: u8 = read_console().parse::<u8>().unwrap();
            if attacker_deck.attacking_cards.contains_key(&chosen_card) {
                let defenderable_card = beatable_cards(
                    players_data.cards.clone(),
                    attacker_deck
                        .attacking_cards
                        .get_key_value(&chosen_card)
                        .unwrap(),
                    div_from_volume,
                );

                if defenderable_card.is_empty() {
                    take_all_card(attacker_deck, defending_deck, players_data);
                    break;
                }

                println!(
                    "Choose the card you want to beat {:?} card with: ",
                    attacker_deck
                        .attacking_cards
                        .get_key_value(&chosen_card)
                        .unwrap()
                );

                let chosen_def_card: u8 = read_console().parse::<u8>().unwrap();

                if defenderable_card.contains(&chosen_def_card) {
                    defending_deck
                        .pairs_beaten_cards
                        .insert(chosen_card, chosen_def_card);
                    defending_deck.defending_cards.insert(
                        chosen_def_card,
                        players_data.cards.get(&chosen_def_card).unwrap().clone(),
                    );
                    players_data.cards.remove(&chosen_def_card);
                }
            }

            if attacker_deck.attacking_cards.len() == defending_deck.defending_cards.len() {
                not_beat_bool = false;
                def_cards = true;
            }
        }

        def_cards
    }
}

fn transfer_cards(
    attacker_deck: &mut AttackerDeck,
    players_data: &mut PlayerData,
    transferrable_cards: HashSet<u8>,
) -> bool {
    let mut complete_transfer = false;
    if players_data.bot {
        todo!()
    } else {
        println!("Your cards: {:?}", players_data.cards);
        println!("Attacking cards: {:?}", attacker_deck.attacking_cards);
        println!("Available cards for transfer: {:?}", transferrable_cards);
        println!("What card do you want to transfer with? Enter card number: ");

        let chosen_card: u8 = read_console().parse::<u8>().unwrap();
        while !complete_transfer {
            if players_data.cards.contains_key(&chosen_card) {
                attacker_deck.attacking_cards.insert(
                    chosen_card,
                    players_data
                        .cards
                        .get_key_value(&chosen_card)
                        .unwrap()
                        .1
                        .clone(),
                );
                players_data.cards.remove(&chosen_card);
                complete_transfer = true;
            } else {
                println!("You entered an incorrect card number");
            }
        }
        complete_transfer
    }
}

fn get_transferrable_cards(
    players_cards: HashMap<u8, Card>,
    attacker_cards: HashMap<u8, Card>,
    div_from_volume: u8,
) -> HashSet<u8> {
    let mut allowed_volume: HashSet<u8> = HashSet::new();

    if attacker_cards.len() < 4 {
        let mut check_volume: u8 = 0;
        let mut check_bool: bool = false;
        for (attacker_card, _) in attacker_cards {
            if check_volume == 0 {
                check_volume = attacker_card % div_from_volume;
                check_bool = true;
            } else if (attacker_card % div_from_volume) != check_volume {
                check_bool = false;
                break;
            }
        }

        if check_bool {
            allowed_volume.extend(
                players_cards
                    .iter()
                    .filter(|(card_value, _card_data)| {
                        *card_value % div_from_volume == check_volume
                    })
                    .map(|(card_value, _card_data)| card_value),
            )
        }
    }

    allowed_volume
}

fn count_div(trump_card_location: usize) -> u8 {
    match trump_card_location {
        24 => 6,
        36 => 9,
        52 => 13,
        // здесь баг, считается, что в игре тогда 4 джокера
        // 54 => 14,
        _ => panic!("A PANIC!"),
    }
}

fn beatable_cards(
    players_cards: HashMap<u8, Card>,
    attacker_card: (&u8, &Card),
    div_from_volume: u8,
) -> HashSet<u8> {
    let mut allowed_volume: HashSet<u8> = HashSet::new();

    if attacker_card.1.trump {
        allowed_volume.extend(
            players_cards
                .into_iter()
                .filter(|(card_value, card_data)| card_data.trump && (card_value > attacker_card.0))
                .map(|(card_value, _card_data)| card_value),
        )
    } else {
        allowed_volume.extend(
            players_cards
                .iter()
                .filter(|(card_value, card_data)| {
                    ((*card_value % div_from_volume == attacker_card.0 % div_from_volume)
                        && *card_value > attacker_card.0)
                        || card_data.trump
                })
                .map(|(card_value, _card_data)| card_value),
        )
    }
    allowed_volume
}

// fn take_cards_from_deck(filled_card_deck: DeckCards, mut players_data: PlayerData) {
//     let num_drawn_cards = 7 - players_data.cards.len();

//     let del_data = if (filled_card_deck.cards.len() - num_drawn_cards) > 0 {
//         num_drawn_cards
//     } else {
//         filled_card_deck.cards.len()
//     };
//     filled_card_deck
//         .cards
//         .iter()
//         .take(del_data)
//         .for_each(|(volume, card_info)| {
//             players_data.cards.insert(*volume, card_info.clone());
//         });
// }

// убирает карты из колоды которые были отданы игроку
fn remove_cards(card_deck: &mut HashMap<u8, Card>, cards: &HashMap<u8, Card>) {
    cards.clone().into_keys().for_each(|card_volume| {
        if card_deck.contains_key(&card_volume) {
            card_deck.remove(&card_volume);
        }
    });
}

// переделать, чтобы была не 54 карта, а 15 карта (т.е. туз для удобства игрока) !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
fn filled_card_deck(rand_vec: &[u8], div_from_volume: u8, trump_volume: u8) -> DeckCards {
    let mut filled_card_deck = DeckCards {
        cards: HashMap::new(),
    };
    rand_vec.iter().for_each(|card| {
        filled_card_deck.cards.insert(
            *card,
            filling_in_card_data(*card, div_from_volume, trump_volume),
        );
    });
    filled_card_deck
}

fn filling_in_card_data(card_volume: u8, div_from_volume: u8, trump_volume: u8) -> Card {
    // Checking if the card is a trump card
    let trump_data = (((card_volume - 1) / div_from_volume) as i32)
        == ((trump_volume - 1) / div_from_volume) as i32;
    match (card_volume - 1) / div_from_volume {
        0 => Card {
            suit: 'c',
            trump: trump_data,
        },
        1 => Card {
            suit: 's',
            trump: trump_data,
        },
        2 => Card {
            suit: 'h',
            trump: trump_data,
        },
        3 => Card {
            suit: 'd',
            trump: trump_data,
        },
        _ => panic!("B PANIC!"),
    }
}

fn gen_deck_in_hand(filled_card_deck: &DeckCards, bot: bool) -> PlayerData {
    // player_state 1 - def, 2 - attack, 3 - throw cards, ...
    let mut card_in_hand = PlayerData {
        cards: HashMap::new(),
        bot,
    };
    let del_data = if (filled_card_deck.cards.len() - 7) > 0 {
        7
    } else {
        filled_card_deck.cards.len()
    };
    filled_card_deck
        .cards
        .iter()
        .take(del_data)
        .for_each(|(volume, card_info)| {
            card_in_hand.cards.insert(*volume, card_info.clone());
        });
    card_in_hand
}

// это можно оптимизировать, но ~120 микро секунд для 54 карт терпимо
fn gen_rand_deck(mut rand_vec: Vec<u8>) -> Vec<u8> {
    let vec_len = rand_vec.len();
    for i in 1..=vec_len {
        let mut rng = rand::thread_rng();
        let mut rand_int: usize = rng.gen_range(0..vec_len);
        while rand_vec[rand_int] != 0 {
            rand_int = rng.gen_range(0..vec_len);
        }
        rand_vec[rand_int] = i as u8;
    }
    rand_vec
}
