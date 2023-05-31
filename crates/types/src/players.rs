use std::{
    collections::BTreeSet,
    iter::once,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use color_eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serialize_hierarchy::{Error, SerializeHierarchy};
use spl_network_messages::{Penalty, Player, PlayerNumber, TeamState};

pub const PLAYERS: usize = 7;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Players<'a, const N: usize, T: ?Sized + Serialize + Deserialize<'a>> {
    // pub player_number: PlayerNumber,
    #[serde(with = "serde_arrays")]
    #[serde(bound(deserialize = "[T; N]: Deserialize<'a>"))]
    pub players: [T; N],
    _phantom: PhantomData<&'a T>,
}
// pub struct Players<T> {
//     pub one: T,
//     pub two: T,
//     pub three: T,
//     pub four: T,
//     pub five: T,
//     pub six: T,
//     pub seven: T,
// }

//  imply functions for generic Players struct
impl<'a, const N: usize, T: ?Sized + Serialize + Deserialize<'a>> Players<'a, N, T> {
    // pub fn new(players: [u32; N]) -> Self {
    //     Self {
    //         players,
    //         _phantom: PhantomData,
    //     }
    // }

    fn index(&self, index: usize) -> &T {
        self.players.get(index).expect("Player index out of bounds")
    }

    fn index_mut(&mut self, index: usize) -> &mut T {
        self.players
            .get_mut(index)
            .expect("Player index out of bounds")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.players.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.players.get_mut(index)
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.players[index] = value;
    }
}

// impl Default for Players
impl<'a, const N: usize, T: ?Sized + Default + Copy + Serialize + Deserialize<'a>> Default
    for Players<'a, N, T>
{
    fn default() -> Self {
        Self {
            players: [Default::default(); N],
            _phantom: PhantomData,
        }
    }
}

// impl<const N: usize, T: Serialize> Players<N, T> {
//     fn index(&self, index: PlayerNumber) -> &T {
//         match index {
//             PlayerNumber::One => self.players.get(0).unwrap(),
//             PlayerNumber::Two => &self.players.get(1).unwrap(),
//             PlayerNumber::Three => &self.players.get(2).unwrap(),
//             PlayerNumber::Four => &self.players.get(3).unwrap(),
//             PlayerNumber::Five => &self.players.get(4).unwrap(),
//             PlayerNumber::Six => &self.players.get(5).unwrap(),
//             PlayerNumber::Seven => &self.players.get(6).unwrap(),
//         }
//     }

//     fn index_mut(&mut self, index: PlayerNumber) -> &mut Self {
//         match index {
//             PlayerNumber::One => &mut self.players.get(0),
//             PlayerNumber::Two => &mut self.players.get(1),
//             PlayerNumber::Three => &mut self.players.get(2),
//             PlayerNumber::Four => &mut self.players.get(3),
//             PlayerNumber::Five => &mut self.players.get(4),
//             PlayerNumber::Six => &mut self.players.get(5),
//             PlayerNumber::Seven => &mut self.players.get(6),
//         }
//     }
// }

// impl<T> Index<PlayerNumber> for Players<T> {
//     type Output = T;
// }

// impl<T> IndexMut<PlayerNumber> for Players<T> {}

trait PlayerPenalty {
    fn get_penalty(&self, player_index: usize) -> Penalty;
}

impl PlayerPenalty for Vec<Player> {
    fn get_penalty(&self, player_index: usize) -> Penalty {
        if self.len() > player_index {
            self[player_index].penalty
        } else {
            Penalty::None
        }
    }
}

impl From<TeamState> for Players<'_, PLAYERS, Penalty> {
    fn from(team_state: TeamState) -> Self {
        Self {
            players: [
                team_state.players.get_penalty(0),
                team_state.players.get_penalty(1),
                team_state.players.get_penalty(2),
                team_state.players.get_penalty(3),
                team_state.players.get_penalty(4),
                team_state.players.get_penalty(5),
                team_state.players.get_penalty(6),
            ],
            _phantom: PhantomData,
        }
    }
}

// impl<'a, T> Iterator for Players<PLAYERS, T>

impl<T> SerializeHierarchy for Players<'_, PLAYERS, T>
where
    T: Serialize + DeserializeOwned + SerializeHierarchy,
{
    fn serialize_path<S>(&self, path: &str, serializer: S) -> Result<S::Ok, Error<S::Error>>
    where
        S: Serializer,
    {
        let split = path.split_once('.');
        match split {
            Some((name, suffix)) => match name {
                // "one" => self.get(0).serialize_path(suffix, serializer),
                "one" => self.index(0).serialize_path(suffix, serializer),
                "two" => self.index(1).serialize_path(suffix, serializer),
                "three" => self.index(2).serialize_path(suffix, serializer),
                "four" => self.index(3).serialize_path(suffix, serializer),
                "five" => self.index(4).serialize_path(suffix, serializer),
                "six" => self.index(5).serialize_path(suffix, serializer),
                "seven" => self.index(6).serialize_path(suffix, serializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => self
                    .index(0)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "two" => self
                    .index(1)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "three" => self
                    .index(2)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "four" => self
                    .index(3)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "five" => self
                    .index(4)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "six" => self
                    .index(5)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "seven" => self
                    .index(6)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
        }
    }

    fn deserialize_path<'de, D>(
        &mut self,
        path: &str,
        deserializer: D,
    ) -> Result<(), Error<D::Error>>
    where
        D: Deserializer<'de>,
    {
        let split = path.split_once('.');
        match split {
            Some((name, suffix)) => match name {
                "one" => self.index_mut(0).deserialize_path(suffix, deserializer),
                "two" => self.index_mut(1).deserialize_path(suffix, deserializer),
                "three" => self.index_mut(2).deserialize_path(suffix, deserializer),
                "four" => self.index_mut(3).deserialize_path(suffix, deserializer),
                "five" => self.index_mut(4).deserialize_path(suffix, deserializer),
                "six" => self.index_mut(5).deserialize_path(suffix, deserializer),
                "seven" => self.index_mut(6).deserialize_path(suffix, deserializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => {
                    self.set(
                        0,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "two" => {
                    self.set(
                        1,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "three" => {
                    self.set(
                        2,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "four" => {
                    self.set(
                        3,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "five" => {
                    self.set(
                        4,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "six" => {
                    self.set(
                        5,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                "seven" => {
                    self.set(
                        6,
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?,
                    );
                    Ok(())
                }
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
        }
    }

    fn exists(path: &str) -> bool {
        let split = path.split_once('.');
        match split {
            Some((name, suffix)) => match name {
                "one" => T::exists(suffix),
                "two" => T::exists(suffix),
                "three" => T::exists(suffix),
                "four" => T::exists(suffix),
                "five" => T::exists(suffix),
                "six" => T::exists(suffix),
                "seven" => T::exists(suffix),
                _ => false,
            },
            None => matches!(
                path,
                "one" | "two" | "three" | "four" | "five" | "six" | "seven"
            ),
        }
    }

    fn get_fields() -> BTreeSet<String> {
        once(String::new())
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("one.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("two.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("three.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("four.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("five.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("six.{name}")),
            )
            .chain(
                T::get_fields()
                    .into_iter()
                    .map(|name| format!("seven.{name}")),
            )
            .collect()
    }
}
