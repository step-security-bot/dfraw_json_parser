use crate::parser::refs::{DF_ENCODING, NON_DIGIT_RE, RAW_TOKEN_RE};

use super::parsing;
use super::raws::info::DFInfoFile;
use super::raws::tags::{CasteTag, CreatureTag};
use super::raws::{biomes, creature, info, names, tags};

use encoding_rs_io::DecodeReaderBytesBuilder;
use slug::slugify;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub enum RawObjectKind {
    Creature,
    None,
}

/// It reads a file, line by line, and checks the first line for the filename, reads lines until it encounters the
/// [OBJECT:<type>] tag in the file.
///
/// Arguments:
///
/// * `input_path`: Path to the file to be read
/// * `module_info`: Information about the raw module `input_path` is within
///
/// Returns:
///
/// RawObjectKind for the type of [OBJECT] tag encountered, and RawObjectKind::None if it is unsupported.
pub fn read_raw_file_type(input_path: &Path) -> RawObjectKind {
    let caller = "Raw File Type Checker";
    // Validate file exists
    if !input_path.exists() {
        log::error!("{} - Path doesn't exist {}", caller, input_path.display());
        return RawObjectKind::None;
    }
    if !input_path.is_file() {
        log::error!(
            "{} - Path does not point to a file {}",
            caller,
            input_path.display(),
        );
        return RawObjectKind::None;
    }

    // Open the file
    let Ok(file) = File::open(input_path) else {
        log::error!("{} - Unable to open file {}",caller, input_path.display());
        return RawObjectKind::None;
    };

    // Setup a file reader for the encoding used by DF
    let decoding_reader = DecodeReaderBytesBuilder::new()
        .encoding(*DF_ENCODING)
        .build(file);
    let reader = BufReader::new(decoding_reader);

    // String to store the parsed filename in
    let mut raw_filename = String::new();

    // Read in lines until we encounter the [OBJECT tag] or complete the file.
    for (index, line) in reader.lines().enumerate() {
        if line.is_err() {
            log::error!(
                "{} - Error processing {}:{}",
                caller,
                input_path.display(),
                index
            );
            continue;
        }
        // Match the line so we can check the pieces of it
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                log::error!("Line-reading error\n{:?}", e);
                continue;
            }
        };
        // The filename is always the top line of a DF raw file
        if index == 0 {
            raw_filename = String::from(&line);
            continue;
        }
        // Multiple matches can occur in a single line, so we loop over all captures within the match
        // for this line.
        for cap in RAW_TOKEN_RE.captures_iter(&line) {
            log::trace!("{} - Key: {} Value: {}", caller, &cap[2], &cap[3]);
            // Match the front part of the tag
            match &cap[2] {
                // We are only concerned with the [OBJECT] key
                "OBJECT" => {
                    log::trace!("{} - {} is a {} raw file", caller, raw_filename, &cap[3]);
                    match &cap[3] {
                        "CREATURE" => {
                            return RawObjectKind::Creature;
                        }
                        // Currently, any other type of raw object is unsupported
                        &_ => {
                            log::debug!("{} - Currently no support for OBJECT:{}", caller, &cap[3]);
                            return RawObjectKind::None;
                            // current_object = RawObjectKind::None;
                        }
                    }
                }
                &_ => (),
            }
        }
    }

    // Reading through the entire file and not finding an [OBJECT] tag means the raw file is invalid
    log::warn!("{} - no [OBJECT] tag in {}", caller, input_path.display());
    RawObjectKind::None
}

pub fn parse_creature_file(input_path: &Path, info_text: &DFInfoFile) -> Vec<creature::DFCreature> {
    let caller = "Parse Creature Raw";
    let mut results: Vec<creature::DFCreature> = Vec::new();

    let file = match File::open(&input_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("{} - Error opening raw file for parsing!\n{:?}", caller, e);
            return results;
        }
    };

    let decoding_reader = DecodeReaderBytesBuilder::new()
        .encoding(*DF_ENCODING)
        .build(file);
    let reader = BufReader::new(decoding_reader);

    // let mut creatures = 0;
    let mut raw_filename = String::new();
    let mut current_object = RawObjectKind::None;
    let mut started = false;
    let mut creature_temp = creature::DFCreature::new("None", "None", info_text);

    let mut caste_tags: Vec<CasteTag> = Vec::new();
    let mut creature_tags: Vec<CreatureTag> = Vec::new();
    let mut temp_caste_vec: Vec<creature::DFCreatureCaste> = Vec::new();

    let mut caste_temp = creature::DFCreatureCaste::new("ALL");

    for (index, line) in reader.lines().enumerate() {
        if line.is_err() {
            log::error!(
                "{} - Error processing {}:{}",
                caller,
                input_path.display(),
                index
            );
            continue;
        }
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                log::error!("{} - Line-reading error\n{:?}", caller, e);
                continue;
            }
        };
        if index == 0 {
            raw_filename = String::from(&line);
            continue;
        }
        for cap in RAW_TOKEN_RE.captures_iter(&line) {
            log::trace!("{} - Key: {} Value: {}", caller, &cap[2], &cap[3]);
            match &cap[2] {
                "OBJECT" => match &cap[3] {
                    "CREATURE" => {
                        // Discovered raws for creatures.
                        current_object = RawObjectKind::Creature;
                    }
                    &_ => {
                        log::debug!("{} - Wrong type of raw ({})", caller, &cap[3]);
                        return Vec::new();
                        // current_object = RawObjectKind::None;
                    }
                },
                "CREATURE" => {
                    // We are starting a creature object capture
                    match current_object {
                        RawObjectKind::Creature => {
                            if started {
                                // If we already *were* capturing a creature, export it.
                                //1. Save caste tags
                                caste_temp.tags = caste_tags.clone();
                                //2. Save caste
                                temp_caste_vec.push(caste_temp.clone());
                                //3. Save creature tags
                                creature_temp.tags = creature_tags.clone();
                                //4. Save tamp_castes to creature
                                creature_temp.castes = temp_caste_vec.clone();
                                //5. Save creature
                                results.push(creature_temp);
                            } else {
                                started = true;
                            }
                            //Reset all temp values
                            //1. Make new creature from [CREATURE:<NAME>]
                            creature_temp =
                                creature::DFCreature::new(&raw_filename, &cap[3], info_text);
                            //2. Make new caste
                            caste_temp = creature::DFCreatureCaste::new("ALL");
                            //3. Reset/empty caste tags
                            caste_tags = Vec::new();
                            //4. Reset/empty creature tags
                            creature_tags = Vec::new();
                            //5. Reset/empty caste vector
                            temp_caste_vec = Vec::new();
                        }
                        RawObjectKind::None => (),
                    }
                }
                "CASTE" => {
                    //1. Save caste tags
                    caste_temp.tags = caste_tags.clone();
                    //2. Save caste
                    temp_caste_vec.push(caste_temp.clone());
                    //3. Make new caste from [CASTE:<NAME>]
                    caste_temp = creature::DFCreatureCaste::new(&cap[3]);
                    //4. Reset/empty caste tags
                    caste_tags = Vec::new();
                }
                "BIOME" => match biomes::BIOMES.get(&cap[3]) {
                    Some(biome_name) => creature_temp.biomes.push((*biome_name).to_string()),
                    None => log::warn!("{} is not in biome dictionary!", &cap[3]),
                },
                "BODY_SIZE" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    if split.len() == 3 {
                        match parsing::parse_body_size(&split) {
                            Ok(size) => caste_temp.body_size.push(size),
                            Err(e) => {
                                log::error!(
                                    "{}:{}:Unable to parse BODYSIZE\n{:?}",
                                    creature_temp.get_identifier(),
                                    caste_temp.name,
                                    e
                                );
                                break;
                            }
                        }
                    }
                }
                "MILKABLE" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    if split.len() == 3 {
                        let freq: u32 = match split[2].parse() {
                            Ok(n) => n,
                            Err(e) => {
                                log::error!(
                                    "{}:{}:Unable to parse frequency from MILKABLE\n{:?}",
                                    creature_temp.get_identifier(),
                                    caste_temp.name,
                                    e
                                );
                                break;
                            }
                        };
                        caste_temp.milkable =
                            tags::DFMilkable::new(&format!("{}:{}", split[0], split[1]), freq);
                    }
                }
                "PREFSTRING" => {
                    creature_temp.pref_string.push(String::from(&cap[3]));
                }
                "CREATURE_CLASS" => {
                    caste_temp.creature_class.push(String::from(&cap[3]));
                }
                "NAME" => {
                    creature_temp.name = names::Name::new(&cap[3]);
                }
                "CASTE_NAME" => {
                    caste_temp.caste_name = names::Name::new(&cap[3]);
                }
                "GENERAL_BABY_NAME" => {
                    creature_temp.general_baby_name = names::SingPlurName::new(&cap[3]);
                }
                "GENERAL_CHILD_NAME" => {
                    creature_temp.general_child_name = names::SingPlurName::new(&cap[3]);
                }
                "LAYS_EGGS" => {
                    caste_tags.push(tags::CasteTag::LaysEggs);
                }
                "EGG_SIZE" => match cap[3].parse() {
                    Ok(n) => caste_temp.egg_size = n,
                    Err(e) => log::error!(
                        "{}:{}:EGG_SIZE parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "BABY" => match cap[3].parse() {
                    Ok(n) => caste_temp.baby = n,
                    Err(e) => log::error!(
                        "{}:{}:BABY parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "CHILD" => match cap[3].parse() {
                    Ok(n) => caste_temp.child = n,
                    Err(e) => log::error!(
                        "{}:{}:CHILD parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "DIFFICULTY" => match cap[3].parse() {
                    Ok(n) => caste_temp.difficulty = n,
                    Err(e) => log::error!(
                        "{}:{}:DIFFICULTY parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "GRASSTRAMPLE" => match cap[3].parse() {
                    Ok(n) => caste_temp.grass_trample = n,
                    Err(e) => log::error!(
                        "{}:{}:GRASSTRAMPLE parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "GRAZER" => match cap[3].parse() {
                    Ok(n) => caste_temp.grazer = n,
                    Err(e) => log::error!(
                        "{}:{}:GRAZER parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "LOW_LIGHT_VISION" => match cap[3].parse() {
                    Ok(n) => caste_temp.low_light_vision = n,
                    Err(e) => log::error!(
                        "{}:{}:LOW_LIGHT_VISION parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "PETVALUE" => match cap[3].parse() {
                    Ok(n) => caste_temp.pet_value = n,
                    Err(e) => log::error!(
                        "{}:{}:PETVALUE parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "POP_RATIO" => match cap[3].parse() {
                    Ok(n) => caste_temp.pop_ratio = n,
                    Err(e) => log::error!(
                        "{}:{}:POP_RATIO parsing error\n{:?}",
                        creature_temp.get_identifier(),
                        caste_temp.name,
                        e
                    ),
                },
                "CLUTCH_SIZE" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            caste_temp.clutch_size[0] = range[0];
                            caste_temp.clutch_size[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:{}:Unable to parse range for CLUTCH_SIZE",
                            creature_temp.get_identifier(),
                            caste_temp.name
                        ),
                    }
                }
                "LITTERSIZE" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            caste_temp.litter_size[0] = range[0];
                            caste_temp.litter_size[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:{}:Unable to parse range for LITTERSIZE",
                            creature_temp.get_identifier(),
                            caste_temp.name
                        ),
                    }
                }
                "DESCRIPTION" => {
                    caste_temp.description = String::from(&cap[3]);
                }
                "MAXAGE" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            caste_temp.max_age[0] = range[0];
                            caste_temp.max_age[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:{}:Unable to parse range for MAXAGE",
                            creature_temp.get_identifier(),
                            caste_temp.name
                        ),
                    }
                }
                "COPY_TAGS_FROM" => {
                    creature_temp.copy_tags_from.push(format!(
                        "{}-{}-{}",
                        raw_filename,
                        "CREATURE",
                        slugify(&cap[3])
                    ));
                }
                "ALL_ACTIVE" => {
                    caste_temp.active_time = caste_temp.active_time
                        | tags::ACTIVE_DIURNAL
                        | tags::ACTIVE_NOCTURNAL
                        | tags::ACTIVE_CREPUSCULAR;
                }
                "DIURNAL" => {
                    caste_temp.active_time |= tags::ACTIVE_DIURNAL;
                }
                "CREPUSCULAR" => {
                    caste_temp.active_time |= tags::ACTIVE_CREPUSCULAR;
                }
                "MATUTINAL" => {
                    caste_temp.active_time |= tags::ACTIVE_MATUTINAL;
                }
                "VESPERTINE" => {
                    caste_temp.active_time |= tags::ACTIVE_VESPERTINE;
                }
                "NOCTURNAL" => {
                    caste_temp.active_time |= tags::ACTIVE_NOCTURNAL;
                }
                "AMBUSHPREDATOR" | "AMBUSH_PREDATOR" => {
                    caste_tags.push(tags::CasteTag::AmbushPredator);
                }
                "AMPHIBIOUS" => {
                    caste_tags.push(tags::CasteTag::Amphibious);
                }
                "CURIOUSBEAST_EATER" => {
                    caste_temp.curious_beast |= tags::CURIOUS_EATER;
                }
                "CURIOUSBEAST_GUZZLER" => {
                    caste_temp.curious_beast |= tags::CURIOUS_GUZZLER;
                }
                "CURIOUSBEAST_ITEM" => {
                    caste_temp.curious_beast |= tags::CURIOUS_ITEM;
                }
                "NO_SPRING" => {
                    caste_temp.no_season |= tags::NO_SPRING;
                }
                "NO_SUMMER" => {
                    caste_temp.no_season |= tags::NO_SUMMER;
                }
                "NO_AUTUMN" => {
                    caste_temp.no_season |= tags::NO_FALL;
                }
                "NO_WINTER" => {
                    caste_temp.no_season |= tags::NO_WINTER;
                }
                "TRAINABLE_HUNTING" => {
                    caste_temp.trainable |= tags::TRAINABLE_HUNTING;
                }
                "TRAINABLE_WAR" => {
                    caste_temp.trainable |= tags::TRAINABLE_WAR;
                }
                "TRAINABLE" => {
                    caste_temp.trainable =
                        caste_temp.trainable | tags::TRAINABLE_WAR | tags::TRAINABLE_HUNTING;
                }
                "ARTIFICIAL_HIVEABLE" => {
                    creature_tags.push(tags::CreatureTag::ArtificialHiveable);
                }
                "CLUSTER_NUMBER" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            creature_temp.cluster_number[0] = range[0];
                            creature_temp.cluster_number[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:Unable to parse range for CLUSTER_NUMBER",
                            creature_temp.get_identifier()
                        ),
                    }
                }
                "POPULATION_NUMBER" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            creature_temp.population_number[0] = range[0];
                            creature_temp.population_number[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:Unable to parse range for POPULATION_NUMBER",
                            creature_temp.get_identifier()
                        ),
                    }
                }
                "DOES_NOT_EXIST" => {
                    creature_tags.push(tags::CreatureTag::DoesNotExist);
                }
                "EVIL" => {
                    creature_tags.push(tags::CreatureTag::Evil);
                }
                "FANCIFUL" => {
                    creature_tags.push(tags::CreatureTag::Fanciful);
                }
                "GOOD" => {
                    creature_tags.push(tags::CreatureTag::Good);
                }
                "SAVAGE" => {
                    creature_tags.push(tags::CreatureTag::Savage);
                }
                "GENERATED" => {
                    creature_tags.push(tags::CreatureTag::Generated);
                }
                "UBIQUITOUS" => {
                    creature_tags.push(tags::CreatureTag::Ubiquitous);
                }
                "VERMIN_SOIL" => {
                    creature_tags.push(tags::CreatureTag::VerminSoil);
                }
                "VERMIN_SOIL_COLONY" => {
                    creature_tags.push(tags::CreatureTag::VerminSoilColony);
                }
                "VERMIN_ROTTER" => {
                    creature_tags.push(tags::CreatureTag::VerminRotter);
                }
                "VERMIN_GROUNDER" => {
                    creature_tags.push(tags::CreatureTag::VerminGrounder);
                }
                "VERMIN_FISH" => {
                    creature_tags.push(tags::CreatureTag::VerminFish);
                }
                "VERMIN_EATER" => {
                    creature_tags.push(tags::CreatureTag::VerminEater);
                }
                "FREQUENCY" => match cap[3].parse() {
                    Ok(n) => creature_temp.frequency = n,
                    Err(_e) => log::error!(
                        "{}:Unable to parse FREQUENCY",
                        creature_temp.get_identifier()
                    ),
                },
                "UNDERGROUND_DEPTH" => {
                    let split = cap[3].split(':').collect::<Vec<&str>>();
                    match parsing::parse_min_max_range(&split) {
                        Ok(range) => {
                            creature_temp.underground_depth[0] = range[0];
                            creature_temp.underground_depth[1] = range[1];
                        }
                        Err(_e) => log::error!(
                            "{}:Unable to parse range for UNDERGROUND_DEPTH",
                            creature_temp.get_identifier()
                        ),
                    }
                }
                "LARGE_ROAMING" => {
                    creature_tags.push(tags::CreatureTag::LargeRoaming);
                }
                "LOCAL_POPS_CONTROLLABLE" => {
                    creature_tags.push(tags::CreatureTag::LocalPopsControllable);
                }
                "LOCAL_POPS_PRODUCE_HEROES" => {
                    creature_tags.push(tags::CreatureTag::LocalPopsProduceHeroes);
                }
                "LOOSE_CLUSTERS" => {
                    creature_tags.push(tags::CreatureTag::LooseClusters);
                }
                "ADOPTS_OWNER" => {
                    caste_tags.push(tags::CasteTag::AdoptsOwner);
                }
                "BENIGN" => {
                    caste_tags.push(tags::CasteTag::Benign);
                }
                "AQUATIC" => {
                    caste_tags.push(tags::CasteTag::Aquatic);
                }
                "ARENA_RESTRICTED" => {
                    caste_tags.push(tags::CasteTag::ArenaRestricted);
                }
                "AT_PEACE_WITH_WILDLIFE" => {
                    caste_tags.push(tags::CasteTag::AtPeaceWithWildlife);
                }
                "BONECARN" => {
                    caste_tags.push(tags::CasteTag::BoneCarn);
                    caste_tags.push(tags::CasteTag::Carnivore);
                }
                "CAN_LEARN" => {
                    caste_tags.push(tags::CasteTag::CanLearn);
                }
                "CAN_SPEAK" => {
                    caste_tags.push(tags::CasteTag::CanSpeak);
                }
                "CARNIVORE" => {
                    caste_tags.push(tags::CasteTag::Carnivore);
                }
                "COMMON_DOMESTIC" => {
                    caste_tags.push(tags::CasteTag::CommonDomestic);
                }
                "COOKABLE_LIVE" => {
                    caste_tags.push(tags::CasteTag::CookableLive);
                }
                "DEMON" => {
                    caste_tags.push(tags::CasteTag::Demon);
                }
                "DIE_WHEN_VERMIN_BITE" => {
                    caste_tags.push(tags::CasteTag::DieWhenVerminBite);
                }
                "EQUIPS" => {
                    caste_tags.push(tags::CasteTag::Equips);
                }
                "EXTRAVISION" => {
                    caste_tags.push(tags::CasteTag::Extravision);
                }
                "FEATURE_BEAST" => {
                    caste_tags.push(tags::CasteTag::FeatureBeast);
                }
                "FEMALE" => {
                    caste_tags.push(tags::CasteTag::Female);
                }
                "FIREIMMUNE" => {
                    caste_tags.push(tags::CasteTag::FireImmune);
                }
                "FIREIMMUNE_SUPER" => {
                    caste_tags.push(tags::CasteTag::FireimmuneSuper);
                }
                "FISHITEM" => {
                    caste_tags.push(tags::CasteTag::FishItem);
                }
                "FLIER" => {
                    caste_tags.push(tags::CasteTag::Flier);
                }
                "GNAWER" => {
                    caste_tags.push(tags::CasteTag::Gnawer);
                }
                "HAS_NERVES" => {
                    caste_tags.push(tags::CasteTag::HasNerves);
                }
                "HUNTS_VERMIN" => {
                    caste_tags.push(tags::CasteTag::HuntsVermin);
                }
                "IMMOBILE" => {
                    caste_tags.push(tags::CasteTag::Immobile);
                }
                "IMMOBILE_LAND" => {
                    caste_tags.push(tags::CasteTag::ImmobileLand);
                }
                "IMMOLATE" => {
                    caste_tags.push(tags::CasteTag::Immolate);
                }
                "INTELLIGENT" => {
                    caste_tags.push(tags::CasteTag::Intelligent);
                }
                "LIGHT_GEN" => {
                    caste_tags.push(tags::CasteTag::LightGen);
                }
                "LOCKPICKER" => {
                    caste_tags.push(tags::CasteTag::LockPicker);
                }
                "MAGMA_VISION" => {
                    caste_tags.push(tags::CasteTag::MagmaVision);
                }
                "MALE" => {
                    caste_tags.push(tags::CasteTag::Male);
                }
                "MEANDERER" => {
                    caste_tags.push(tags::CasteTag::Meanderer);
                }
                "MEGABEAST" => {
                    caste_tags.push(tags::CasteTag::Megabeast);
                }
                "MISCHIEVIOUS" | "MISCHIEVOUS" => {
                    caste_tags.push(tags::CasteTag::Mischievous);
                }
                "MOUNT" => {
                    caste_tags.push(tags::CasteTag::Mount);
                }
                "MOUNT_EXOTIC" => {
                    caste_tags.push(tags::CasteTag::MountExotic);
                }
                "MULTIPART_FULL_VISION" => {
                    caste_tags.push(tags::CasteTag::MultipartFullVision);
                }
                "MULTIPLE_LITTER_RARE" => {
                    caste_tags.push(tags::CasteTag::MultipleLitterRare);
                }
                "MUNDANE" => {
                    creature_tags.push(tags::CreatureTag::Mundane);
                }
                "NATURAL" => {
                    caste_tags.push(tags::CasteTag::Natural);
                }
                "NO_CONNECTIONS_FOR_MOVEMENT" => {
                    caste_tags.push(tags::CasteTag::NoConnectionsForMovement);
                }
                "NO_DIZZINESS" => {
                    caste_tags.push(tags::CasteTag::NoDizziness);
                }
                "NO_DRINK" => {
                    caste_tags.push(tags::CasteTag::NoDrink);
                }
                "NO_EAT" => {
                    caste_tags.push(tags::CasteTag::NoEat);
                }
                "NO_FEVERS" => {
                    caste_tags.push(tags::CasteTag::NoFevers);
                }
                "NO_GENDER" => {
                    caste_tags.push(tags::CasteTag::NoGender);
                }
                "NO_SLEEP" => {
                    caste_tags.push(tags::CasteTag::NoSleep);
                }
                "NOBONES" => {
                    caste_tags.push(tags::CasteTag::NoBones);
                }
                "NOBREATHE" => {
                    caste_tags.push(tags::CasteTag::NoBreathe);
                }
                "NOEMOTION" => {
                    caste_tags.push(tags::CasteTag::NoEmotion);
                }
                "NOEXERT" => {
                    caste_tags.push(tags::CasteTag::NoExert);
                }
                "NOFEAR" => {
                    caste_tags.push(tags::CasteTag::NoFear);
                }
                "NOMEAT" => {
                    caste_tags.push(tags::CasteTag::NoMeat);
                }
                "NONAUSEA" => {
                    caste_tags.push(tags::CasteTag::NoNausea);
                }
                "NOPAIN" => {
                    caste_tags.push(tags::CasteTag::NoPain);
                }
                "NOSKIN" => {
                    caste_tags.push(tags::CasteTag::NoSkin);
                }
                "NOSKULL" => {
                    caste_tags.push(tags::CasteTag::NoSkull);
                }
                "NOSMELLYROT" => {
                    caste_tags.push(tags::CasteTag::NoSmellyRot);
                }
                "NOSTUCKINS" => {
                    caste_tags.push(tags::CasteTag::NoStuckIns);
                }
                "NOSTUN" => {
                    caste_tags.push(tags::CasteTag::NoStrun);
                }
                "NOT_BUTCHERABLE" => {
                    caste_tags.push(tags::CasteTag::NotButcherable);
                }
                "NOT_LIVING" => {
                    caste_tags.push(tags::CasteTag::NotLiving);
                }
                "NOTHOUGHT" => {
                    caste_tags.push(tags::CasteTag::NoThought);
                }
                "OPPOSED_TO_LIFE" => {
                    caste_tags.push(tags::CasteTag::OpposedToLife);
                }
                "OUTSIDER_CONTROLLABLE" => {
                    caste_tags.push(tags::CasteTag::OutsiderControllable);
                }
                "PACK_ANIMAL" => {
                    caste_tags.push(tags::CasteTag::PackAnimal);
                }
                "PARALYZEIMMUNE" => {
                    caste_tags.push(tags::CasteTag::ParalyzeImmune);
                }
                "PET" => {
                    caste_tags.push(tags::CasteTag::Pet);
                }
                "PET_EXOTIC" => {
                    caste_tags.push(tags::CasteTag::PetExotic);
                }
                "POWER" => {
                    caste_tags.push(tags::CasteTag::Power);
                }
                "SEMIMEGABEAST" => {
                    caste_tags.push(tags::CasteTag::SemiMegabeast);
                }
                "SLOW_LEARNER" => {
                    caste_tags.push(tags::CasteTag::SlowLearner);
                }
                "SMALL_REMAINS" => {
                    caste_tags.push(tags::CasteTag::SmallRemains);
                }
                "STANDARD_GRAZER" => {
                    caste_tags.push(tags::CasteTag::StandardGrazer);
                }
                "SUPERNATURAL" => {
                    caste_tags.push(tags::CasteTag::Supernatural);
                }
                "SWIMS_INNATE" => {
                    caste_tags.push(tags::CasteTag::SwimsInnate);
                }
                "SWIMS_LEARNED" => {
                    caste_tags.push(tags::CasteTag::SwimsLearned);
                }
                "THICKWEB" => {
                    caste_tags.push(tags::CasteTag::ThickWeb);
                }
                "TITAN" => {
                    caste_tags.push(tags::CasteTag::Titan);
                }
                "TRANCES" => {
                    caste_tags.push(tags::CasteTag::Trances);
                }
                "TRAPAVOID" => {
                    caste_tags.push(tags::CasteTag::TrapAvoid);
                }
                "UNIQUE_DEMON" => {
                    caste_tags.push(tags::CasteTag::UniqueDemon);
                }
                "VEGETATION" => {
                    caste_tags.push(tags::CasteTag::Vegetation);
                }
                "VERMIN_HATEABLE" => {
                    caste_tags.push(tags::CasteTag::VerminHateable);
                }
                "VERMIN_MICRO" => {
                    caste_tags.push(tags::CasteTag::VerminMicro);
                }
                "VERMIN_NOFISH" => {
                    caste_tags.push(tags::CasteTag::VerminNofish);
                }
                "VERMIN_NOROAM" => {
                    caste_tags.push(tags::CasteTag::VerminNoroam);
                }
                "VERMIN_NOTRAP" => {
                    caste_tags.push(tags::CasteTag::VerminNotrap);
                }
                "WAGON_PULLER" => {
                    caste_tags.push(tags::CasteTag::WagonPuller);
                }
                "WEBIMMUNE" => {
                    caste_tags.push(tags::CasteTag::WebImmune);
                }
                "SELECT_CASTE" => {
                    //SELECT_CASTE:<CASTE_NAME> --> retrieve tags for this
                    let target_caste_name = &cap[3];
                    log::trace!(
                        "{}: selecting caste {}",
                        creature_temp.get_identifier(),
                        target_caste_name
                    );
                    //1. Save current tags
                    caste_temp.tags = caste_tags.clone();
                    //2. Save caste
                    temp_caste_vec.push(caste_temp.clone());
                    // (Assume we didn't find a matching caste)
                    //4. Make new caste from [CASTE:<NAME>]
                    caste_temp = creature::DFCreatureCaste::new(&cap[3]);
                    //5. Reset/empty caste tags
                    caste_tags = Vec::new();
                    //3. Find and get the caste we select
                    let mut caste_found = false;
                    let mut target_caste_index: usize = 0;
                    for (index, val) in temp_caste_vec.iter().enumerate() {
                        if val.name.eq(target_caste_name) {
                            //Save index
                            target_caste_index = index;
                            //Set caste found
                            caste_found = true;
                            // Break loop
                            break;
                        }
                    }
                    if caste_found {
                        //4. Grab the target caste from the array
                        caste_temp = temp_caste_vec.swap_remove(target_caste_index);
                        //5. Set the tag array to its tags
                        caste_tags = caste_temp.tags.clone();
                    }
                }
                &_ => (),
            }
        }
    }
    match current_object {
        RawObjectKind::Creature => {
            // If we already *were* capturing a creature, export it.
            //1. Save caste tags
            caste_temp.tags = caste_tags.clone();
            //2. Save caste
            temp_caste_vec.push(caste_temp.clone());
            //3. Save creature tags
            creature_temp.tags = creature_tags.clone();
            //4. Save tamp_castes to creature
            creature_temp.castes = temp_caste_vec.clone();
            //5. Save creature
            results.push(creature_temp);
        }
        RawObjectKind::None => (),
    }
    log::info!(
        "{} creatures defined in {} ({} {} in {})",
        results.len(),
        &raw_filename,
        info_text.get_identifier(),
        info_text.displayed_version,
        info_text.get_sourced_directory(),
    );
    results
}

pub fn parse_dfraw_module_info_file(info_file_path: &Path, source_dir: &str) -> info::DFInfoFile {
    let file = match File::open(&info_file_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Error opening raw file for parsing!\n{:?}", e);
            return info::DFInfoFile::new("error", source_dir);
        }
    };

    let decoding_reader = DecodeReaderBytesBuilder::new()
        .encoding(*DF_ENCODING)
        .build(file);
    let reader = BufReader::new(decoding_reader);

    // info.txt details
    let mut info_file_data: info::DFInfoFile = info::DFInfoFile::new("", source_dir);

    for (index, line) in reader.lines().enumerate() {
        if line.is_err() {
            log::error!(
                "DFInfoFile - Error processing {:?}:{}",
                &info_file_path,
                index
            );
            continue;
        }
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                log::error!("DFInfoFile - Line-reading error\n{:?}", e);
                continue;
            }
        };
        for cap in RAW_TOKEN_RE.captures_iter(&line) {
            log::trace!("Key: {} Value: {}", &cap[2], &cap[3]);
            match &cap[2] {
                // SECTION FOR MATCHING info.txt DATA
                "ID" => {
                    // the [ID:identifier] tag should be the top of the info.txt file
                    info_file_data = info::DFInfoFile::new(&cap[3], source_dir);
                }
                "NUMERIC_VERSION" => match cap[3].parse() {
                    Ok(n) => info_file_data.numeric_version = n,
                    Err(_e) => {
                        log::warn!(
                            "DFInfoFile - 'numeric_version' value in {}: '{}' is not integer! file: {}",
                            info_file_data.get_identifier(),
                            &cap[3],
                            info_file_path.display()
                        );
                        // match on \D to replace any non-digit characters with empty string
                        let digits_only = NON_DIGIT_RE.replace_all(&cap[3], "").to_string();
                        match digits_only.parse() {
                            Ok(n) => info_file_data.numeric_version = n,
                            Err(_e) => {
                                log::error!(
                                    "DFInfoFile - Unable to parse numerals from {}",
                                    digits_only
                                );
                            }
                        }
                    }
                },
                "EARLIEST_COMPATIBLE_NUMERIC_VERSION" => match cap[3].parse() {
                    Ok(n) => info_file_data.earliest_compatible_numeric_version = n,
                    Err(_e) => {
                        log::warn!(
                            "DFInfoFile - 'earliest_compatible_numeric_version' value in {}: '{}' is not integer! file: {}",
                            info_file_data.get_identifier(),
                            &cap[3],
                            info_file_path.display()
                        );
                        // match on \D to replace any non-digit characters with empty string
                        let digits_only = NON_DIGIT_RE.replace_all(&cap[3], "").to_string();
                        match digits_only.parse() {
                            Ok(n) => info_file_data.earliest_compatible_numeric_version = n,
                            Err(_e) => {
                                log::error!(
                                    "DFInfoFile - Unable to parse numerals from {}",
                                    digits_only
                                );
                            }
                        }
                    }
                },
                "DISPLAYED_VERSION" => {
                    info_file_data.displayed_version = String::from(&cap[3]);
                }
                "EARLIEST_COMPATIBLE_DISPLAYED_VERSION" => {
                    info_file_data.earliest_compatible_displayed_version = String::from(&cap[3]);
                }
                "AUTHOR" => {
                    info_file_data.author = String::from(&cap[3]);
                }
                "NAME" => {
                    info_file_data.name = String::from(&cap[3]);
                }
                "DESCRIPTION" => {
                    info_file_data.description = String::from(&cap[3]);
                }
                &_ => (),
            }
        }
    }
    info_file_data
}
