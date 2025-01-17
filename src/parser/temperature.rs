use crate::parser::serializer_helper;
use serde::{Deserialize, Serialize};

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Temperatures {
    /// This determines how long it takes the material to heat up or cool down.
    /// A material with a high specific heat capacity will hold more heat and affect its surroundings more
    /// before cooling down or heating up to equilibrium. The input for this token is not temperature,
    /// but rather the specific heat capacity of the material.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    specific_heat: u32,
    /// This is the temperature at which the material will catch fire.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    ignition_point: u32,
    /// This is the temperature at which a liquid material will freeze, or a solid material will melt.
    /// In Dwarf Fortress the melting point and freezing point coincide exactly; this is contrary to many
    /// real-life materials, which can be supercooled.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    melting_point: u32,
    /// This is the temperature at which the material will boil or condense. Water boils at 10180 °U
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    boiling_point: u32,
    /// This is the temperature above which the material will begin to take heat damage.
    /// Burning items without a heat damage point (or with an exceptionally high one) will take damage very slowly,
    /// causing them to burn for a very long time (9 months and 16.8 days) before disappearing.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    heat_damage_point: u32,
    /// This is the temperature below which the material will begin to take frost damage.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    cold_damage_point: u32,
    /// A material's temperature can be forced to always be a certain value via the `MAT_FIXED_TEMP`
    /// material definition token. The only standard material which uses this is nether-cap wood,
    /// whose temperature is always at the melting point of water. If a material's temperature is fixed
    /// to between its cold damage point and its heat damage point, then items made from that material
    /// will never suffer cold/heat damage. This makes nether-caps fire-safe and magma-safe despite being a type of wood.
    #[serde(skip_serializing_if = "serializer_helper::is_zero")]
    material_fixed_temperature: u32,
}

impl Temperatures {
    pub fn is_empty(&self) -> bool {
        self.specific_heat == 0
            && self.ignition_point == 0
            && self.melting_point == 0
            && self.boiling_point == 0
            && self.heat_damage_point == 0
            && self.cold_damage_point == 0
            && self.material_fixed_temperature == 0
    }
    pub fn update_specific_heat(&mut self, value: u32) {
        self.specific_heat = value;
    }
    pub fn update_ignition_point(&mut self, value: u32) {
        self.ignition_point = value;
    }
    pub fn update_melting_point(&mut self, value: u32) {
        self.melting_point = value;
    }
    pub fn update_boiling_point(&mut self, value: u32) {
        self.boiling_point = value;
    }
    pub fn update_heat_damage_point(&mut self, value: u32) {
        self.heat_damage_point = value;
    }
    pub fn update_cold_damage_point(&mut self, value: u32) {
        self.cold_damage_point = value;
    }
    pub fn update_material_fixed_temperature(&mut self, value: u32) {
        self.material_fixed_temperature = value;
    }
}
