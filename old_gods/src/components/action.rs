use specs::prelude::{Entity, Component, FlaggedStorage, HashMapStorage, ReadStorage, };

use super::Name;
use super::super::parser::*;


/// Encodes the strategies by which we evaluate an entity's elligibility to take
/// an action.
#[derive(Debug, Clone, PartialEq)]
pub enum FitnessStrategy {
  /// The target must have an item with the matching name in their inventory.
  HasItem(String),

  /// the target must have an inventory.
  HasInventory,

  /// The target may pass any fitness test
  Any(Vec<FitnessStrategy>),

  /// The target must pass all fitness tests
  All(Vec<FitnessStrategy>),
}


impl FitnessStrategy {
  pub fn tiled_key() -> String {
    "fitness".to_string()
  }

  pub fn from_str(
    input: &str,
  ) -> Result<FitnessStrategy, Err<(&str, ErrorKind)>> {
    let result = FitnessStrategyParser::parse(input);
    result.map(|(_, f)| f)
  }
}


pub struct FitnessStrategyParser;


impl FitnessStrategyParser {
  /// Parse a HasItem
  /// ```
  /// extern crate engine;
  /// use engine::systems::action::*;
  ///
  /// let my_str = "has_item \"white key\"";
  /// assert_eq!(
  ///   FitnessStrategyParser::has_item(my_str),
  ///   Ok(("", FitnessStrategy::HasItem("white key".to_string())))
  /// );
  /// ```
  pub fn has_item(i: &str) -> IResult<&str, FitnessStrategy> {
    let (i, _) = tag("has_item")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, n) = string(i)?;
    Ok((i, FitnessStrategy::HasItem(n.to_string())))
  }

  /// Parse a HasInventory
  /// ```
  /// extern crate engine;
  /// use engine::systems::action::*;
  ///
  /// let my_str = "has_inventory";
  /// assert_eq!(
  ///   FitnessStrategyParser::has_inventory(my_str),
  ///   Ok(("", FitnessStrategy::HasInventory))
  /// );
  /// ```
  pub fn has_inventory(i: &str) -> IResult<&str, FitnessStrategy> {
    let (i, _) = tag("has_inventory")(i)?;
    Ok((i, FitnessStrategy::HasInventory))
  }

  /// Parse an Any.
  pub fn any(i: &str) -> IResult<&str, FitnessStrategy> {
    let (i, _) = tag("any")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, v) = vec(&FitnessStrategyParser::parse)(i)?;

    Ok((i, FitnessStrategy::Any(v)))
  }

  /// Parse an All.
  pub fn all(i: &str) -> IResult<&str, FitnessStrategy> {
    let (i, _) = tag("all")(i)?;
    let (i, _) = multispace1(i)?;
    let (i, v) = vec(&FitnessStrategyParser::parse)(i)?;

    Ok((i, FitnessStrategy::All(v)))
  }

  /// Parse a FitnessStrategy
  pub fn parse(i: &str) -> IResult<&str, FitnessStrategy> {
    alt((
      FitnessStrategyParser::has_item,
      FitnessStrategyParser::has_inventory,
      FitnessStrategyParser::any,
      FitnessStrategyParser::all,
    ))(i)
  }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Lifespan {
  /// This thing has `n` uses.
  Many(u32),

  /// This thing never dies.
  Forever,
}


impl Lifespan {
  pub fn succ(&self) -> Lifespan {
    match self {
      Lifespan::Many(n) => Lifespan::Many(n + 1),
      Lifespan::Forever => Lifespan::Forever,
    }
  }

  pub fn pred(&self) -> Lifespan {
    match self {
      Lifespan::Many(0) => Lifespan::Many(0),
      Lifespan::Many(n) => Lifespan::Many(n - 1),
      Lifespan::Forever => Lifespan::Forever,
    }
  }

  pub fn is_dead(&self) -> bool {
    match self {
      Lifespan::Many(0) => true,
      _ => false,
    }
  }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Action {
  /// Any entities that are elligible to take this action.
  pub elligibles: Vec<Entity>,

  /// All the entities that have taken this action.
  pub taken_by: Vec<Entity>,

  /// Some text about the action to display to the user.
  pub text: String,

  /// The method to use for determining whether an entity is elligible to
  /// take this action.
  pub strategy: FitnessStrategy,

  /// The lifespan of this action.
  pub lifespan: Lifespan,
}


impl Component for Action {
  type Storage = FlaggedStorage<Self, HashMapStorage<Self>>;
}


/// Component used to request that an action be taken on behalf of an entity.
pub struct TakeAction;


impl Component for TakeAction {
  type Storage = HashMapStorage<TakeAction>;
}


#[derive(Debug, PartialEq)]
enum FitnessResult {
  Fit,
  UnfitDoesntHaveItem,
  UnfitDoesntHaveInventory,
  Unfit,
}


impl FitnessStrategy {
  ///// Determine whether or not the target entity is fit to take this action.
  //fn target_is_fit<'a>(
  //  &self,
  //  target_entity: &Entity,
  //  inventories: &ReadStorage<'a, Inventory>,
  //  names: &ReadStorage<'a, Name>,
  //) -> FitnessResult {
  //  match self {
  //    FitnessStrategy::HasItem(name) => {
  //      println!("  looking for item {:?}", name);
  //      let has_item = inventories
  //        .get(*target_entity)
  //        .map(|inv| {
  //          for item_ent in &inv.items {
  //            let Name(item_name) =
  //              names.get(*item_ent).expect("An item is missing a name.");
  //            println!("  checking item {:?}", item_name);
  //            if name == item_name {
  //              return true;
  //            }
  //          }
  //          false
  //        })
  //        .unwrap_or(false);
  //      if has_item {
  //        FitnessResult::Fit
  //      } else {
  //        FitnessResult::UnfitDoesntHaveItem
  //      }
  //    }

  //    FitnessStrategy::HasInventory => {
  //      let has_inventory = inventories.contains(*target_entity);
  //      if has_inventory {
  //        FitnessResult::Fit
  //      } else {
  //        FitnessResult::UnfitDoesntHaveInventory
  //      }
  //    }

  //    FitnessStrategy::All(strategies) => {
  //      for strategy in strategies {
  //        let fitness =
  //          strategy.target_is_fit(target_entity, inventories, names);
  //        if fitness != FitnessResult::Fit {
  //          return fitness;
  //        }
  //      }
  //      FitnessResult::Fit
  //    }

  //    FitnessStrategy::Any(strategies) => {
  //      for strategy in strategies {
  //        let fitness =
  //          strategy.target_is_fit(target_entity, inventories, names);
  //        if fitness == FitnessResult::Fit {
  //          return fitness;
  //        }
  //      }
  //      FitnessResult::Unfit
  //    }
  //  }
  //}
}
