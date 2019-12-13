#[derive(Debug, Clone)]
pub enum PathComponent {
  /// The entity has a Name
  HasName(String),

  /// The entity is an Effect
  IsEffect
}

#[derive(Debug, Clone)]
pub enum PathSpace {
  /// The entity lives within an inventory. Get the inventory by its Name.
  Inventory(String, Vec<PathComponent>),
}


/// An EntityPathComponent is a type that allows us to locate an entity by some
/// number of components, nestings, etc. It is used to resolve pointers to
/// entities created by map loading.
#[derive(Debug, Clone)]
pub struct EntityPath {
  space: PathSpace
}


pub struct EntityPathParser;


impl EntityPathParser {
  pub fn name(i: &str) -> IResult<&str, PathComponent> {
    let (i, _) = tag("name")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, s) = string(i)?;
    Ok((i, PathComponent::HasName(s)))
  }

  pub fn is_effect(i: &str) -> IResult<&str, PathComponent> {
    let (i, _) = tag("is_effect")(i)?;
    Ok((i, PathComponent::IsEffect))
  }

  pub fn component(i: &str) -> IResult<&str, PathComponent> {
    alt((
      EntityPathParser::name,
      EntityPathParser::is_effect
    ))(i)
  }


  pub fn parse_path(i: &str) -> IResult<&str, EntityPath> {
    let (i, _) = tag("inventory")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, (inv_name, comps)) = params2(
      string,
      vec(&EntityPathParser::component)
    )(i)?;
    Ok(
      (i,
       EntityPath{
         space: PathSpace::Inventory(inv_name, comps)
       }
      ))
  }
}


impl EntityPath {
  /// Parses an EntityPath from a string.
  pub fn from_str(input: &str) -> Result<EntityPath, Err<(&str, ErrorKind)>> {
    let result = EntityPathParser::parse_path(input);
    result
      .map(|(_, e)| e)
  }

  /// Resolve an EntityPath, retreiving an Entity, if possible.
  pub fn resolve(&self, world: &World) -> Vec<Entity> {
    let get_inventory_items =
      |s: &String| -> Vec<Entity> {
        let entities = world.entities();
        let inventories = world.read_storage::<Inventory>();
        let names = world.read_storage::<Name>();
        (&entities, &inventories, &names)
          .join()
          .filter_map(|(_, inventory, name)| {
            if name.0 == *s {
              Some(inventory.items.clone())
            } else {
              None
            }
          })
          .flatten()
          .collect()
      };
    match &self.space {
      PathSpace::Inventory(s, comps) => {
        if comps.is_empty() {
          get_inventory_items(s)
        } else {
          get_inventory_items(s)
            .into_iter()
            .filter(|ent: &Entity| {
              // The ent must have every comp
              for comp in comps {
                match comp {
                  PathComponent::IsEffect => {
                    let effects = world.read_storage::<Effect>();
                    if effects.get(*ent).is_none() {
                      return false;
                    }
                  }

                  PathComponent::HasName(name) => {
                    let names = world.read_storage::<Name>();
                    if let Some(Name(item_name)) = names.get(*ent) {
                      if *item_name != *name {
                        return false;
                      }
                    }
                  }
                }
              }
              true
            })
            .collect::<Vec<_>>()
        }
      }
    }
  }
}
