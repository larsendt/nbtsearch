mod chunk;
mod region;
mod structured_result;
mod world;

use anyhow::Result;
use log::debug;
use simplelog::*;
use structopt::StructOpt;
use structured_result::StructuredResult;
use world::World;

#[derive(StructOpt, Debug)]
struct Opt {
    world_dir: String,
    #[structopt(short, long)]
    item_id_search: String,
    #[structopt(short, long)]
    dimension: String,
}

fn search_item_by_id(search_str: &str, world: &World, dimension: &str) -> Result<StructuredResult> {
    let mut results = vec![];

    for region in world.entity_regions(dimension)? {
        for chunk in region?.all_chunks() {
            let blob = chunk?.nbt;
            let maybe_ents = blob.get("Entities");
            if let Some(nbt::Value::List(entities)) = maybe_ents {
                for ent in entities {
                    if let nbt::Value::Compound(compound) = ent {
                        if let Some(nbt::Value::Compound(item)) = compound.get("Item") {
                            if let Some(nbt::Value::String(item_id)) = item.get("id") {
                                if item_id == search_str {
                                    results.push(ent.clone());
                                } else {
                                    debug!("Mismatch: {} != {}", search_str, item_id);
                                }
                            } else {
                                panic!("No item ID found in item: {:?}", ent);
                            }
                        } else {
                            let ent_id = compound.get("id").unwrap();
                            debug!("Entity was not an item: {:?}", ent_id);
                        }
                    } else {
                        panic!("NBT was not a Compound: {:?}", ent);
                    }
                }
            } else {
                panic!("Expected an entities NBT, but got: {:?}", maybe_ents);
            }
        }
    }

    Ok(StructuredResult::found_items(results)?)
}

fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let opt = Opt::from_args();
    let world = World::new(opt.world_dir);

    let item_id_search = if opt.item_id_search.starts_with("minecraft:") {
        opt.item_id_search
    } else {
        format!("minecraft:{}", opt.item_id_search)
    };

    let dimension = opt.dimension.to_lowercase();

    let result = match search_item_by_id(&item_id_search, &world, &dimension) {
        Ok(i) => i,
        Err(e) => StructuredResult::err(e),
    };
    println!("{}", result.to_interface_string()?);
    Ok(())
}
