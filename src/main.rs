use std::{
    collections::{HashMap, HashSet},
    io,
    time::Instant,
};

use rand::Rng;

type Id = u8;

fn main() {
    let now = Instant::now();
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

    let mut filled_card_hash = filled_card_hash(&rand_vec_cards, vec_len, trump_volume);

    let bot1_start_deck = gen_deck_in_hand(&filled_card_hash, false);
    filled_card_hash = remove_hand_cards(filled_card_hash, &bot1_start_deck);

    let bot2_start_deck = gen_deck_in_hand(&filled_card_hash, false);
    filled_card_hash = remove_hand_cards(filled_card_hash, &bot2_start_deck);

    let elapsed_time = now.elapsed();

    println!(
        "The array was generated in {} micros",
        elapsed_time.as_micros()
    );
    // println!("Bot_start_deck: {:?}", bot1_start_deck);
    // println!("Bot_start_deck: {:?}", bot2_start_deck);
    // println!("New Rand_deck: {:?}", filled_card_hash);
}

struct GameData {
    who_attacker: Id,
    attacker_deck: AttackerDeck,
    defending_deck: DefendingDeck,
    deck_of_cards: DeckCards,
    players_data: HashMap<Id, PlayersData>,
}

// если младшей козырной карты нет ни у кого на руках, то игрок, который будет ходить первым будет выбираться рандомно
fn who_is_first(players_data: HashMap<Id, PlayersData>) -> Id {
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

fn chose_cards(mut chosen_cards: HashMap<u8, Card>, players_data: PlayersData) {
    // переделать !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // добавить возможность вбросить несколько карт одной стоимости но разных мастей
    println!("Your cards: {:?}", players_data.cards);
    println!("Chose cards: ");

    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();

    let chosen_card: u8 = match buffer.trim_end().parse::<u8>() {
        Ok(chosen_cards) => chosen_cards,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    if players_data.cards.contains_key(&chosen_card) {
        let (k, v) = players_data.cards.get_key_value(&chosen_card).unwrap();
        chosen_cards.insert(*k, v.clone());
    } else {
        println!("Error: card not found");
    }
}

fn play_card(mut attacker_deck: AttackerDeck, mut chosen_cards: HashMap<u8, Card>) {
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

// нападающий
#[derive(Debug, Clone)]
struct AttackerDeck {
    attacking_cards: HashMap<u8, Card>,
}

////////
#[derive(Debug, Clone)]
struct PlayersData {
    cards: HashMap<u8, Card>,
    // 1 нападение, 2 защита, 3, 4 и тд по ре
    player_state: u8,
    bot: bool,
}

#[derive(Debug, Clone)]
struct Card {
    suit: char,
    trump: bool,
}

fn throwable_cards(
    attacker_deck: AttackerDeck,
    defending_deck: DefendingDeck,
    players_data: PlayersData,
    trump_card_location: usize,
) -> HashMap<u8, Card> {
    let allowed_cards_to_throw = {
        let mut cards: HashMap<u8, Card> = HashMap::new();
        cards.extend(attacker_deck.attacking_cards);
        cards.extend(defending_deck.defending_cards);
        cards
    };

    let div_from_volume = match trump_card_location {
        24 => 6,
        36 => 9,
        52 => 13,
        54 => 15,
        _ => panic!("A PANIC!"),
    };

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
    defending_deck: DefendingDeck,
    players_data: PlayersData,
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
    mut attacker_deck: AttackerDeck,
    mut defending_deck: DefendingDeck,
    mut players_data: PlayersData,
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
    attacker_deck: AttackerDeck,
    mut defending_deck: DefendingDeck,
    mut players_data: PlayersData,
    trump_card_location: usize,
) {
    if players_data.bot {
        todo!()
    } else {
        let mut not_beat_bool = true;
        let copy_attacker_deck = attacker_deck.attacking_cards.clone();
        while not_beat_bool {
            println!("Your cards: {:?}", players_data.cards);
            println!("Attacking cards: {:?}", copy_attacker_deck);
            println!("Choose the card you want to beat: ");

            let chosen_card: u8 = read_console().parse::<u8>().unwrap();
            if copy_attacker_deck.contains_key(&chosen_card) {
                let defenderable_card = beatable_cards(
                    players_data.cards.clone(),
                    copy_attacker_deck.get_key_value(&chosen_card).unwrap(),
                    trump_card_location,
                );

                if defenderable_card.is_empty() {
                    take_all_card(attacker_deck, defending_deck, players_data);
                    break;
                }

                println!(
                    "Choose the card you want to beat {:?} card with: ",
                    copy_attacker_deck.get_key_value(&chosen_card).unwrap()
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

// отбивающийся
#[derive(Debug, Clone)]
struct DefendingDeck {
    pairs_beaten_cards: HashMap<u8, u8>,
    defending_cards: HashMap<u8, Card>,
    how_much_cards_defender_have: u8,
}

// добавить переводную функцию
fn beatable_cards(
    players_cards: HashMap<u8, Card>,
    attacker_card: (&u8, &Card),
    trump_card_location: usize,
) -> HashSet<u8> {
    let div_from_volume = match trump_card_location {
        24 => 6,
        36 => 9,
        52 => 13,
        54 => 15,
        _ => panic!("A PANIC!"),
    };
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

#[derive(Debug, Clone)]
struct DeckCards {
    cards: HashMap<u8, Card>,
    // нужно для присваивания id игрокам
    id: Id,
}

fn take_cards_from_deck(filled_card_hash: DeckCards, mut players_data: PlayersData) {
    let num_drawn_cards = 7 - players_data.cards.len();

    let del_data = if (filled_card_hash.cards.len() - num_drawn_cards) > 0 {
        num_drawn_cards
    } else {
        filled_card_hash.cards.len()
    };
    filled_card_hash
        .cards
        .iter()
        .take(del_data)
        .for_each(|(volume, card_info)| {
            players_data.cards.insert(*volume, card_info.clone());
        });
}

fn remove_hand_cards(mut filled_card_hash: DeckCards, card_hash: &PlayersData) -> DeckCards {
    card_hash
        .cards
        .clone()
        .into_iter()
        .for_each(|(card_volume, _)| {
            if filled_card_hash.cards.contains_key(&card_volume) {
                filled_card_hash.cards.remove(&card_volume);
            }
        });

    filled_card_hash
}

fn filled_card_hash(rand_vec: &[u8], vec_len: usize, trump_volume: u8) -> DeckCards {
    let mut filled_card_hash = DeckCards {
        cards: HashMap::new(),
        id: 0,
    };
    rand_vec.iter().for_each(|card| {
        filled_card_hash
            .cards
            .insert(*card, filling_in_card_data(*card, vec_len, trump_volume));
    });
    filled_card_hash
}

fn filling_in_card_data(card_volume: u8, trump_card_location: usize, trump_volume: u8) -> Card {
    // println!(
    //     "Card volume: {:?}, location: {:?}, trump_volume: {:?}",
    //     card_volume, trump_card_location, trump_volume
    // );
    let div_from_volume = match trump_card_location {
        24 => 6,
        36 => 9,
        52 => 13,
        54 => 15,
        _ => panic!("A PANIC!"),
    };
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

fn gen_deck_in_hand(filled_card_hash: &DeckCards, bot: bool) -> PlayersData {
    let mut card_in_hand = PlayersData {
        cards: HashMap::new(),
        player_state: 0,
        bot,
    };
    let del_data = if (filled_card_hash.cards.len() - 7) > 0 {
        7
    } else {
        filled_card_hash.cards.len()
    };
    filled_card_hash
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
