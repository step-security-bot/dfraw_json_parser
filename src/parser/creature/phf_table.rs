use super::tokens::CreatureTag;
pub static CREATURE_TOKENS: phf::Map<&'static str, CreatureTag> = phf::phf_map! {
    "ARTIFICIAL_HIVEABLE" => CreatureTag::ArtificialHiveable,
    "DOES_NOT_EXIST" => CreatureTag::DoesNotExist,
    "EVIL" => CreatureTag::Evil,
    "GOOD" => CreatureTag::Good,
    "FANCIFUL" => CreatureTag::Fanciful,
    "SAVAGE" => CreatureTag::Savage,
    "GENERATED" => CreatureTag::Generated,
    "UBIQUITOUS" => CreatureTag::Ubiquitous,
    "VERMIN_FISH" => CreatureTag::VerminFish,
    "VERMIN_SOIL" => CreatureTag::VerminSoil,
    "VERMIN_SOIL_COLONY" => CreatureTag::VerminSoilColony,
    "VERMIN_ROTTER" => CreatureTag::VerminRotter,
    "VERMIN_GROUNDER" => CreatureTag::VerminGrounder,
    "VERMIN_EATER" => CreatureTag::VerminEater,
    "FREQUENCY" => CreatureTag::Frequency,
    "UNDERGROUND_DEPTH" => CreatureTag::UndergroundDepth,
    "LARGE_ROAMING" => CreatureTag::LargeRoaming,
    "LOCAL_POPS_CONTROLLABLE" => CreatureTag::LocalPopsControllable,
    "LOCAL_POPS_PRODUCE_HEROES" => CreatureTag::LocalPopsProduceHeroes,
    "LOOSE_CLUSTERS" => CreatureTag::LooseClusters,
    "MUNDANE" => CreatureTag::Mundane,
    "BIOME" => CreatureTag::Biome,
    "PREFSTRING" => CreatureTag::PrefString,
    "NAME" => CreatureTag::Name,
    "GENERAL_BABY_NAME" => CreatureTag::GeneralBabyName,
    "GENERAL_CHILD_NAME" => CreatureTag::GeneralChildName,
    "POPULATION_NUMBER" => CreatureTag::PopulationNumber,
    "COPY_TAGS_FROM" => CreatureTag::CopyTagsFrom,
    "APPLY_CREATURE_VARIATION" => CreatureTag::ApplyCreatureVariation,
    "CREATURE_TILE" => CreatureTag::CreatureTile,
    "ALTTILE" => CreatureTag::AltTile,
    "COLOR" => CreatureTag::Color,
    "GLOWCOLOR" => CreatureTag::GlowColor,
    "GLOWTILE" => CreatureTag::GlowTile,
    "CHANGE_FREQUENCY_PERC" => CreatureTag::ChangeFrequencyPercent,
    "CLUSTER_NUMBER" => CreatureTag::ClusterNumber,
};
