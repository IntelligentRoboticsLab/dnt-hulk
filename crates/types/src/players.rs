use std::{
    collections::BTreeSet,
    iter::once,
    ops::{Index, IndexMut},
};

use color_eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serialize_hierarchy::{Error, SerializeHierarchy};
use spl_network_messages::{Penalty, Player, PlayerNumber, TeamState};

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize)]
pub struct Players<T> {
    pub one: T,
    pub two: T,
    pub three: T,
    pub four: T,
    pub five: T,
}

impl<T> Index<PlayerNumber> for Players<T> {
    type Output = T;

    fn index(&self, index: PlayerNumber) -> &Self::Output {
        match index {
            PlayerNumber::One => &self.one,
            PlayerNumber::Two => &self.two,
            PlayerNumber::Three => &self.three,
            PlayerNumber::Four => &self.four,
            PlayerNumber::Five => &self.five,
        }
    }
}

impl<T> IndexMut<PlayerNumber> for Players<T> {
    fn index_mut(&mut self, index: PlayerNumber) -> &mut Self::Output {
        match index {
            PlayerNumber::One => &mut self.one,
            PlayerNumber::Two => &mut self.two,
            PlayerNumber::Three => &mut self.three,
            PlayerNumber::Four => &mut self.four,
            PlayerNumber::Five => &mut self.five,
        }
    }
}

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

impl From<TeamState> for Players<Penalty> {
    fn from(team_state: TeamState) -> Self {
        Self {
            one: team_state.players.get_penalty(0),
            two: team_state.players.get_penalty(1),
            three: team_state.players.get_penalty(2),
            four: team_state.players.get_penalty(3),
            five: team_state.players.get_penalty(4),
        }
    }
}

pub struct PlayersIterator<'a, T> {
    data: &'a Players<T>,
    player_number: Option<PlayerNumber>,
}

impl<'a, T> PlayersIterator<'a, T> {
    fn new(data: &'a Players<T>) -> Self {
        Self {
            data,
            player_number: Some(PlayerNumber::One),
        }
    }
}

impl<'a, T> Iterator for PlayersIterator<'a, T> {
    type Item = (PlayerNumber, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.player_number.map(|number| match number {
            PlayerNumber::One => (PlayerNumber::One, &self.data.one),
            PlayerNumber::Two => (PlayerNumber::Two, &self.data.two),
            PlayerNumber::Three => (PlayerNumber::Three, &self.data.three),
            PlayerNumber::Four => (PlayerNumber::Four, &self.data.four),
            PlayerNumber::Five => (PlayerNumber::Five, &self.data.five),
        });
        self.player_number = match self.player_number {
            Some(PlayerNumber::One) => Some(PlayerNumber::Two),
            Some(PlayerNumber::Two) => Some(PlayerNumber::Three),
            Some(PlayerNumber::Three) => Some(PlayerNumber::Four),
            Some(PlayerNumber::Four) => Some(PlayerNumber::Five),
            Some(PlayerNumber::Five) => None,
            None => None,
        };
        result
    }
}

impl<T> Players<T> {
    pub fn iter(&self) -> PlayersIterator<'_, T> {
        PlayersIterator::new(self)
    }
}

impl<T> SerializeHierarchy for Players<T>
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
                "one" => self.one.serialize_path(suffix, serializer),
                "two" => self.two.serialize_path(suffix, serializer),
                "three" => self.three.serialize_path(suffix, serializer),
                "four" => self.four.serialize_path(suffix, serializer),
                "five" => self.five.serialize_path(suffix, serializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => self
                    .one
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "two" => self
                    .two
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "three" => self
                    .three
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "four" => self
                    .four
                    .serialize(serializer)
                    .map_err(Error::SerializationFailed),
                "five" => self
                    .five
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
                "one" => self.one.deserialize_path(suffix, deserializer),
                "two" => self.two.deserialize_path(suffix, deserializer),
                "three" => self.three.deserialize_path(suffix, deserializer),
                "four" => self.four.deserialize_path(suffix, deserializer),
                "five" => self.five.deserialize_path(suffix, deserializer),
                name => Err(Error::UnexpectedPathSegment {
                    segment: name.to_string(),
                }),
            },
            None => match path {
                "one" => {
                    self.one =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "two" => {
                    self.two =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "three" => {
                    self.three =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "four" => {
                    self.four =
                        T::deserialize(deserializer).map_err(Error::DeserializationFailed)?;
                    Ok(())
                }
                "five" => {
                    self.five =
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
                _ => false,
            },
            None => matches!(path, "one" | "two" | "three" | "four" | "five"),
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
            .collect()
    }
}
