use serde::{Deserialize, Serialize};

use crate::parser::{
    body_size::BodySize,
    milkable::Milkable,
    names::{Name, SingPlurName},
    ranges::parse_min_max_range,
    searchable::Searchable,
    serializer_helper,
    tile::Tile,
};

use super::{phf_table::CASTE_TOKENS, tokens::CasteTag};

#[derive(ts_rs::TS)]
#[ts(export)]
#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Caste {
    identifier: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<CasteTag>,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
    // String Tokens
    #[serde(skip_serializing_if = "SingPlurName::is_empty")]
    baby_name: SingPlurName,
    #[serde(skip_serializing_if = "Name::is_empty")]
    caste_name: Name,
    #[serde(skip_serializing_if = "SingPlurName::is_empty")]
    child_name: SingPlurName,
    // [min, max] ranges
    /// Default \[0,0\]
    #[serde(skip_serializing_if = "serializer_helper::min_max_is_zeroes")]
    clutch_size: [u16; 2],
    /// Default \[0,0\]
    #[serde(skip_serializing_if = "serializer_helper::min_max_is_zeroes")]
    litter_size: [u16; 2],
    /// Default \[0,0\]
    #[serde(skip_serializing_if = "serializer_helper::min_max_is_zeroes")]
    max_age: [u16; 2],
    // Integer tokens
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    baby: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    child: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    difficulty: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    egg_size: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero_u8")]
    grass_trample: u8,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    grazer: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    low_light_vision: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    pet_value: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    pop_ratio: u32,
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    change_body_size_percentage: u32,
    // Arrays
    #[serde(skip_serializing_if = "Vec::is_empty")]
    creature_class: Vec<String>,
    // Special Tokens
    #[serde(skip_serializing_if = "Vec::is_empty")]
    body_size: Vec<BodySize>,
    #[serde(skip_serializing_if = "Milkable::is_default")]
    milkable: Milkable,
    #[serde(skip_serializing_if = "Tile::is_default")]
    tile: Tile,
}

impl Caste {
    pub fn new(name: &str) -> Caste {
        Caste {
            identifier: String::from(name),
            ..Caste::default()
        }
    }
    pub fn parse_tag(&mut self, key: &str, value: &str) {
        let Some(tag) = CASTE_TOKENS.get(key) else {
            log::warn!(
                "CasteParsing: called `Option::unwrap()` on a `None` value for presumed caste tag: {}",
                key
            );
            return;
        };

        // If value is empty, add the tag to the last caste
        if value.is_empty() {
            self.tags.push(tag.clone());
            return;
        }

        match tag {
            CasteTag::Description => self.description = String::from(value),
            CasteTag::EggSize => self.egg_size = value.parse::<u32>().unwrap_or_default(),
            CasteTag::Baby => self.baby = value.parse::<u32>().unwrap_or_default(),
            CasteTag::Child => self.child = value.parse::<u32>().unwrap_or_default(),
            CasteTag::Difficulty => self.difficulty = value.parse::<u32>().unwrap_or_default(),
            CasteTag::Grazer => self.grazer = value.parse::<u32>().unwrap_or_default(),
            CasteTag::GrassTrample => self.grass_trample = value.parse::<u8>().unwrap_or_default(),
            CasteTag::LowLightVision => {
                self.low_light_vision = value.parse::<u32>().unwrap_or_default();
            }
            CasteTag::PopRatio => self.pop_ratio = value.parse::<u32>().unwrap_or_default(),
            CasteTag::PetValue => self.pet_value = value.parse::<u32>().unwrap_or_default(),
            CasteTag::ClutchSize => {
                self.clutch_size = parse_min_max_range(value).unwrap_or_default();
            }
            CasteTag::LitterSize => {
                self.litter_size = parse_min_max_range(value).unwrap_or_default();
            }
            CasteTag::MaxAge => self.max_age = parse_min_max_range(value).unwrap_or_default(),
            CasteTag::CreatureClass => self.creature_class.push(String::from(value)),
            CasteTag::BodySize => {
                self.body_size.push(BodySize::from_value(value));
            }
            CasteTag::Milkable => self.milkable = Milkable::from_value(value),
            CasteTag::BabyName => self.baby_name = SingPlurName::from_value(value),
            CasteTag::CasteName => self.caste_name = Name::from_value(value),
            CasteTag::ChildName => self.child_name = SingPlurName::from_value(value),
            CasteTag::CasteTile => self.tile.set_character(value),
            CasteTag::CasteAltTile => self.tile.set_alt_character(value),
            CasteTag::CasteColor => self.tile.set_color(value),
            CasteTag::CasteGlowTile => self.tile.set_glow_character(value),
            CasteTag::CasteGlowColor => self.tile.set_glow_color(value),
            CasteTag::ChangeBodySizePercent => {
                self.change_body_size_percentage = value.parse::<u32>().unwrap_or_default();
            }
            _ => self.tags.push(tag.clone()),
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn remove_tag_and_value(&mut self, key: &str, value: &str) {
        let Some(tag) = CASTE_TOKENS.get(key) else {
            log::warn!(
                "CreatureParsing: called `Option::unwrap()` on a `None` value for presumed caste tag: {}",
                key
            );
            return;
        };

        // If value is empty, remove the tag from the last caste
        if value.is_empty() {
            self.tags.retain(|t| t != tag);
            return;
        }

        match tag {
            CasteTag::Description => self.description = String::new(),
            CasteTag::EggSize => self.egg_size = 0,
            CasteTag::Baby => self.baby = 0,
            CasteTag::Child => self.child = 0,
            CasteTag::Difficulty => self.difficulty = 0,
            CasteTag::Grazer => self.grazer = 0,
            CasteTag::GrassTrample => self.grass_trample = 0,
            CasteTag::LowLightVision => self.low_light_vision = 0,
            CasteTag::PopRatio => self.pop_ratio = 0,
            CasteTag::PetValue => self.pet_value = 0,
            CasteTag::ClutchSize => self.clutch_size = [0, 0],
            CasteTag::LitterSize => self.litter_size = [0, 0],
            CasteTag::MaxAge => self.max_age = [0, 0],
            CasteTag::CreatureClass => self.creature_class.retain(|c| c != value),
            CasteTag::BodySize => {
                let body_size_to_remove = BodySize::from_value(value);
                self.body_size.retain(|bs| bs != &body_size_to_remove);
            }
            CasteTag::Milkable => self.milkable = Milkable::default(),
            CasteTag::BabyName => self.baby_name = SingPlurName::default(),
            CasteTag::CasteName => self.caste_name = Name::default(),
            CasteTag::ChildName => self.child_name = SingPlurName::default(),
            CasteTag::CasteTile => self.tile.set_character(""),
            CasteTag::CasteAltTile => self.tile.set_alt_character(""),
            CasteTag::CasteColor => self.tile.set_color(""),
            CasteTag::CasteGlowTile => self.tile.set_glow_character(""),
            CasteTag::CasteGlowColor => self.tile.set_glow_color(""),
            CasteTag::ChangeBodySizePercent => self.change_body_size_percentage = 0,
            _ => self.tags.retain(|t| t != tag),
        }
    }

    pub fn overwrite_caste(&mut self, other: &Caste) {
        // Include any tags from other that aren't in self
        for tag in &other.tags {
            if !self.tags.contains(tag) {
                self.tags.push(tag.clone());
            }
        }
        // For any of the other's values that are not default, overwrite self's values
        if !other.description.is_empty() {
            self.description = other.description.clone();
        }
        if !other.baby_name.is_empty() {
            self.baby_name = other.baby_name.clone();
        }
        if !other.caste_name.is_empty() {
            self.caste_name = other.caste_name.clone();
        }
        if !other.child_name.is_empty() {
            self.child_name = other.child_name.clone();
        }
        if other.clutch_size != [0, 0] {
            self.clutch_size = other.clutch_size;
        }
        if other.litter_size != [0, 0] {
            self.litter_size = other.litter_size;
        }
        if other.max_age != [0, 0] {
            self.max_age = other.max_age;
        }
        if other.baby != 0 {
            self.baby = other.baby;
        }
        if other.child != 0 {
            self.child = other.child;
        }
        if other.difficulty != 0 {
            self.difficulty = other.difficulty;
        }
        if other.egg_size != 0 {
            self.egg_size = other.egg_size;
        }
        if other.grass_trample != 0 {
            self.grass_trample = other.grass_trample;
        }
        if other.grazer != 0 {
            self.grazer = other.grazer;
        }
        if other.low_light_vision != 0 {
            self.low_light_vision = other.low_light_vision;
        }
        if other.pet_value != 0 {
            self.pet_value = other.pet_value;
        }
        if other.pop_ratio != 0 {
            self.pop_ratio = other.pop_ratio;
        }
        if other.change_body_size_percentage != 0 {
            self.change_body_size_percentage = other.change_body_size_percentage;
        }
        if !other.creature_class.is_empty() {
            self.creature_class = other.creature_class.clone();
        }
        if !other.body_size.is_empty() {
            self.body_size = other.body_size.clone();
        }
        if !other.milkable.is_default() {
            self.milkable = other.milkable.clone();
        }
        if !other.tile.is_default() {
            self.tile = other.tile.clone();
        }
    }

    pub fn is_egg_layer(&self) -> bool {
        self.tags.contains(&CasteTag::LaysEggs)
    }
    pub fn is_milkable(&self) -> bool {
        self.tags.contains(&CasteTag::Milkable)
    }
}

impl Searchable for Caste {
    // Used to help extend things that own this caste
    fn get_search_vec(&self) -> Vec<String> {
        let mut vec = Vec::new();

        // Identifier
        vec.push(self.identifier.clone());
        // Name (and child/baby names)
        vec.extend(self.caste_name.as_vec());
        vec.extend(self.child_name.as_vec());
        vec.extend(self.baby_name.as_vec());
        // Creature Class
        vec.extend(self.creature_class.clone());
        // Description
        vec.push(self.description.clone());
        // If egg layer, include egg information
        if self.is_egg_layer() {
            vec.push(String::from("eggs"));
            vec.push(format!("{}", self.egg_size));
        }
        // If milkable, include milk information
        if self.is_milkable() {
            vec.push(String::from("milk"));
            vec.extend(self.milkable.as_vec());
        }
        // If flier, include flyer information
        if self.tags.contains(&CasteTag::Flier) {
            vec.push(String::from("flying flies flier"));
        }
        // If gnawer, include gnawer information
        if self.tags.contains(&CasteTag::Gnawer) {
            vec.push(String::from("gnawer"));
        }
        // If playable/civilized, include playable information
        if self.tags.contains(&CasteTag::OutsiderControllable) {
            vec.push(String::from("playable civilized"));
        }
        // Include difficulty if not 0
        if self.difficulty > 0 {
            vec.push(format!("{}", self.difficulty));
        }
        // Include pet value if not 0
        if self.pet_value > 0 {
            vec.push(format!("{}", self.pet_value));
        }
        // If speaks, include language information
        // If learns, include learn
        // If both, include "intelligent"
        if self.tags.contains(&CasteTag::Intelligent) || self.tags.contains(&CasteTag::CanSpeak) {
            vec.push(String::from("speaks language"));
        }
        if self.tags.contains(&CasteTag::Intelligent) || self.tags.contains(&CasteTag::CanLearn) {
            vec.push(String::from("learns"));
        }
        if self.tags.contains(&CasteTag::Intelligent)
            || (self.tags.contains(&CasteTag::CanSpeak) && self.tags.contains(&CasteTag::CanLearn))
        {
            vec.push(String::from("intelligent"));
        }

        vec
    }
}
