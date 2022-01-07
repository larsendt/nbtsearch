use anyhow::{anyhow, Context, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum StructuredResult {
    Error(String),
    FoundItems(Vec<FoundItem>),
    NoResult,
}

#[derive(Debug, Serialize)]
pub struct FoundItem {
    pub item_id: String,
    // TODO
    pub name: Option<String>,
    pub location: (i64, i64, i64),
    pub location_status: Option<String>,
}

impl StructuredResult {
    pub fn err(e: anyhow::Error) -> Self {
        Self::Error(format!("{:#}", e))
    }

    pub fn found_items(item_nbts: Vec<nbt::Value>) -> Result<Self> {
        if item_nbts.is_empty() {
            return Ok(Self::NoResult);
        }

        let mut items = vec![];
        for item_nbt in item_nbts {
            if let nbt::Value::Compound(compound) = item_nbt {
                items.push(FoundItem {
                    item_id: Self::get_item_id(&compound)?,
                    name: None,
                    location: Self::get_item_location(&compound)?,
                    location_status: None,
                });
            } else {
                panic!("Item NBT wasn't a compound");
            }
        }

        Ok(Self::FoundItems(items))
    }

    pub fn to_interface_string(&self) -> Result<String> {
        serde_json::to_string(&self).context("Failed to convert StructuredResult to JSON")
    }

    fn get_item_id(compound: &nbt::Map<String, nbt::Value>) -> Result<String> {
        let item_val = compound
            .get("Item")
            .context("Compound didn't have an Item field")?;
        let item = match item_val {
            nbt::Value::Compound(map) => map,
            _ => return Err(anyhow!("Compound's item field wasn't a Compound")),
        };

        let str_val = item.get("id").context("Item didn't have ID")?;
        if let nbt::Value::String(s) = str_val {
            Ok(s.clone())
        } else {
            Err(anyhow!("Item's 'id' wasn't a String field"))
        }
    }

    fn get_item_location(compound: &nbt::Map<String, nbt::Value>) -> Result<(i64, i64, i64)> {
        let list_val = compound
            .get("Pos")
            .context("Compound didn't have a Pos field")?;

        let l = match list_val {
            nbt::Value::List(l) => l,
            _ => return Err(anyhow!("Compound's Pos field wasn't a List")),
        };

        fn get_pos_elem(elem: &nbt::Value) -> i64 {
            match elem {
                nbt::Value::Float(f) => *f as i64,
                _ => panic!("Pos elem was not a Float"),
            }
        }

        Ok((
            get_pos_elem(&l[0]),
            get_pos_elem(&l[1]),
            get_pos_elem(&l[2]),
        ))
    }
}
