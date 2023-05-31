use std::{
    collections::BTreeSet,
    iter::once,
    ops::{Index, IndexMut},
};

use color_eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serialize_hierarchy::{Error, SerializeHierarchy};
use spl_network_messages::{Penalty, Player, PlayerNumber, TeamState};

pub const PLAYERS: usize = 7;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Players<const D: usize, T: Serialize + Deserialize> {
    // pub player_number: PlayerNumber,
    #[serde(with = "serde_arrays")]
    pub players: [T; D],
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

impl<const D: usize, T: Serialize> Players<D, T> {
    fn index(&self, index: PlayerNumber) -> &T {
        match index {
            PlayerNumber::One => self.players.get(0).unwrap(),
            PlayerNumber::Two => &self.players.get(1).unwrap(),
            PlayerNumber::Three => &self.players.get(2).unwrap(),
            PlayerNumber::Four => &self.players.get(3).unwrap(),
            PlayerNumber::Five => &self.players.get(4).unwrap(),
            PlayerNumber::Six => &self.players.get(5).unwrap(),
            PlayerNumber::Seven => &self.players.get(6).unwrap(),
        }
    }

    fn index_mut(&mut self, index: PlayerNumber) -> &mut Self {
        match index {
            PlayerNumber::One => &mut self.players.get(0),
            PlayerNumber::Two => &mut self.players.get(1),
            PlayerNumber::Three => &mut self.players.get(2),
            PlayerNumber::Four => &mut self.players.get(3),
            PlayerNumber::Five => &mut self.players.get(4),
            PlayerNumber::Six => &mut self.players.get(5),
            PlayerNumber::Seven => &mut self.players.get(6),
        }
    }
}

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

impl From<TeamState> for Players<PLAYERS, Penalty> {
    fn from(team_state: TeamState) -> Self {
        Self {
            players: team_state
                .players
                .iter()
                .map(|player| player.penalty)
                .collect(),
        }
    }
}

// impl<'a, T> Iterator for Players<PLAYERS, T>

impl<T> SerializeHierarchy for Players<PLAYERS, T>
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
                "one" => self.get(0).serialize_path(suffix, serializer),
                "two" => self.get(1).serialize_path(suffix, serializer),
                "three" => self.get(2).serialize_path(suffix, serializer),
                "four" => self.get(3).serialize_path(suffix, serializer),
                "five" => self.get(4).serialize_path(suffix, serializer),
                "six" => self.get(5).serialize_path(suffix, serializer),
                "seven" => self.get(6).serialize_path(suffix, serializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => self
                    .get(0)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "two" => self
                    .get(1)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "three" => self
                    .get(2)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "four" => self
                    .get(3)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "five" => self
                    .get(4)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "six" => self
                    .get(5)
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "seven" => self
                    .get(6)
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
                "one" => self.get(0).deserialize_path(suffix, deserializer),
                "two" => self.get(1).deserialize_path(suffix, deserializer),
                "three" => self.get(2).deserialize_path(suffix, deserializer),
                "four" => self.get(3).deserialize_path(suffix, deserializer),
                "five" => self.get(4).deserialize_path(suffix, deserializer),
                "six" => self.get(5).deserialize_path(suffix, deserializer),
                "seven" => self.get(6).deserialize_path(suffix, deserializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => {
                    self.get(0) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "two" => {
                    self.get(1) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "three" => {
                    self.get(2) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "four" => {
                    self.get(3) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "five" => {
                    self.get(4) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "six" => {
                    self.get(5) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "seven" => {
                    self.get(6) =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
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
