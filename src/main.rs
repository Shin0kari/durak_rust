use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    io,
    time::Instant,
};

use rand::Rng;

type Id = u8;

struct GameData {
    who_attacker: Id,
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

    let mut deck_of_cards = filled_card_deck(&rand_vec_cards, vec_len, trump_volume);

    let mut players_data: HashMap<u8, PlayerData> = HashMap::new();

    println!("Enter the number of players (up to 8 people): ");
    let mut chosen_num: u8 = read_console().parse::<u8>().unwrap();
    while chosen_num > 8 {
        println!("An incorrect number of players was entered.");
        println!("Enter the number of players (up to 8 people): ");
        chosen_num = read_console().parse::<u8>().unwrap();
    }

    for i in 1..=chosen_num {
        println!("Will player {} be a bot?(true/false): ", chosen_num);
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
    };

    let who_firs_id = who_is_first(deck_data.players_data.clone());
    deck_data.who_attacker = who_firs_id;

    println!("\nLets start the game!\n");
    let win = false;
    let mut transfered_cards = false;
    let mut took_cards = false;
    let mut def_cards = false;

    while !win {
        for whose_turn in 1..=chosen_num {
            let who_id = deck_data.who_attacker + whose_turn - 1;
            println!("Player {}'s turn", who_id);

            match whose_turn {
                1 => {
                    let mut chosen_cards: HashMap<u8, Card> = HashMap::new();

                    if let Some(mut player_data) = deck_data.players_data.get_mut(&who_id) {
                        chose_cards(&mut chosen_cards, &player_data.cards.clone());
                        remove_cards(&mut player_data.cards, &chosen_cards.clone());
                        play_card(&mut deck_data.attacker_deck, chosen_cards);
                    };
                }
                2 => {
                    if let Some(mut player_data) = deck_data.players_data.get_mut(&who_id) {
                        // auto remove cards, after fix attacker
                        println!(
                            "Attacker deck: {:?}",
                            deck_data.attacker_deck.attacking_cards
                        );
                        println!("Your cards: {:?}", player_data.cards.clone());

                        let transferrable_cards = get_transferrable_cards(
                            player_data.cards.clone(),
                            attacker_deck.attacking_cards.clone(),
                            vec_len,
                        );

                        let mut check_transfer: bool =
                            if deck_data.defender_deck.defending_cards.is_empty()
                                && !transferrable_cards.is_empty()
                            {
                                println!(
                                "Do you want to transfer cards to another player?(true/false): "
                            );
                                read_console().parse::<bool>().unwrap()
                            } else {
                                false
                            };

                        if check_transfer {
                            check_transfer = transfer_cards(
                                &mut deck_data.attacker_deck,
                                player_data,
                                vec_len,
                                transferrable_cards,
                            );
                            if check_transfer {}
                        };
                        if check_transfer {
                            beat_cards(
                                &mut deck_data.attacker_deck,
                                &mut deck_data.defender_deck,
                                &mut player_data,
                                vec_len,
                            );
                        }
                    }
                }
                3..=8 => {}
                _ => panic!("PANICAAA!"),
            }
        }
    }

    let elapsed_time = now.elapsed();

    println!(
        "The array was generated in {} micros",
        elapsed_time.as_micros()
    );
    // println!("Bot_start_deck: {:?}", bot1_start_deck);
    // println!("Bot_start_deck: {:?}", bot2_start_deck);
    // println!("New Rand_deck: {:?}", filled_card_deck);
}

// если младшей козырной карты нет ни у кого на руках, то игрок, который будет ходить первым будет выбираться рандомно
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

fn chose_cards(chosen_cards: &mut HashMap<u8, Card>, cards: &HashMap<u8, Card>) {
    // переделать !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // добавить возможность вбросить несколько карт одной стоимости но разных мастей
    println!("Your cards: {:?}", cards);
    println!("Chose cards: ");

    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();

    let chose_card: u8 = buffer.trim_end().parse::<u8>().unwrap();

    if cards.contains_key(&chose_card) {
        let (k, v) = cards.get_key_value(&chose_card).unwrap();
        chosen_cards.insert(*k, v.clone());
    } else {
        println!("Error: card not found");
    }
    // chosen_cards
}

fn play_card(attacker_deck: &mut AttackerDeck, mut chosen_cards: HashMap<u8, Card>) {
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
    trump_card_location: usize,
) -> HashMap<u8, Card> {
    let allowed_cards_to_throw = {
        let mut cards: HashMap<u8, Card> = HashMap::new();
        cards.extend(attacker_deck.attacking_cards);
        cards.extend(defending_deck.defending_cards);
        cards
    };

    let div_from_volume = count_div(trump_card_location);

    let allowed_volume: HashSet<u8> = allowed_cards_to_throw
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

fn what_card_to_throw(
    attacker_deck: AttackerDeck,
    defending_deck: DefenderDeck,
    players_data: PlayerData,
    trump_card_location: usize,
) {
    if players_data.bot {
        todo!()
    } else {
        println!("Your cards: {:?}", players_data.cards);
        println!("Would you like to throw some cards? (true/false)");

        let mut throw_bool: bool = read_console().parse::<bool>().unwrap_or_default();
        let mut num_throwable_cards = defending_deck.how_much_cards_defender_have
            - (defending_deck.pairs_beaten_cards.len() as u8);

        let mut chosen_cards: HashMap<u8, Card> = HashMap::new();
        let mut playable_cards: HashMap<u8, Card> = HashMap::new();
        if throw_bool {
            playable_cards = throwable_cards(
                attacker_deck.clone(),
                defending_deck.clone(),
                players_data.clone(),
                trump_card_location,
            );
        }

        while throw_bool || (num_throwable_cards > 0) {
            println!("You can play with these cards: {:?}", playable_cards);
            println!("Choose the card you want to throw: ");

            let choosen_card: u8 = read_console().parse::<u8>().unwrap();

            if playable_cards.contains_key(&choosen_card) {
                chosen_cards.insert(
                    choosen_card,
                    playable_cards.get(&choosen_card).unwrap().clone(),
                );
                playable_cards.remove(&choosen_card);
                num_throwable_cards -= 1;
                println!("Do you want to throw another card? (true/false)");
                throw_bool = read_console().parse::<bool>().unwrap_or_default();
            } else {
                println!("You chose a card that you don't have");
            }
        }

        if !chosen_cards.is_empty() {
            play_card(attacker_deck, chosen_cards);
        }
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
    trump_card_location: usize,
) {
    if players_data.bot {
        todo!()
    } else {
        let mut not_beat_bool = true;
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
                    trump_card_location,
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
            }
        }
    }
}

fn transfer_cards(
    attacker_deck: &mut AttackerDeck,
    players_data: &mut PlayerData,
    trump_card_location: usize,
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
    trump_card_location: usize,
) -> HashSet<u8> {
    let div_from_volume = count_div(trump_card_location);
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
        54 => 15,
        _ => panic!("A PANIC!"),
    }
}

// добавить переводную функцию
fn beatable_cards(
    players_cards: HashMap<u8, Card>,
    attacker_card: (&u8, &Card),
    trump_card_location: usize,
) -> HashSet<u8> {
    let div_from_volume = count_div(trump_card_location);
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

fn take_cards_from_deck(filled_card_deck: DeckCards, mut players_data: PlayerData) {
    let num_drawn_cards = 7 - players_data.cards.len();

    let del_data = if (filled_card_deck.cards.len() - num_drawn_cards) > 0 {
        num_drawn_cards
    } else {
        filled_card_deck.cards.len()
    };
    filled_card_deck
        .cards
        .iter()
        .take(del_data)
        .for_each(|(volume, card_info)| {
            players_data.cards.insert(*volume, card_info.clone());
        });
}

// убирает карты из колоды которые были отданы игроку
fn remove_cards(card_deck: &mut HashMap<u8, Card>, cards: &HashMap<u8, Card>) {
    cards.clone().into_iter().map(|(card_volume, _)| {
        if card_deck.contains_key(&card_volume) {
            card_deck.remove(&card_volume);
        }
    });
}

// переделать, чтобы была не 54 карта, а 15 карта (т.е. туз для удобства игрока) !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
fn filled_card_deck(rand_vec: &[u8], vec_len: usize, trump_volume: u8) -> DeckCards {
    let mut filled_card_deck = DeckCards {
        cards: HashMap::new(),
    };
    rand_vec.iter().for_each(|card| {
        filled_card_deck
            .cards
            .insert(*card, filling_in_card_data(*card, vec_len, trump_volume));
    });
    filled_card_deck
}

fn filling_in_card_data(card_volume: u8, trump_card_location: usize, trump_volume: u8) -> Card {
    let div_from_volume = count_div(trump_card_location);
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
