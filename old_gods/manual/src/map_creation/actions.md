# Actions

Actions are special components that allow characters to interact with objects in
the world.

![action display](../img/action_display.png "a player's available action")

<small> Action "Unlock with white key" </small>

An action has a description, a fitness strategy.

A good example of an action is the "Pick up" action automatically created for every
item, which enables a character to pick up an item from the map and place it in
their inventory. Another example is the action on a door that opens or closes the
door.

### To place an action

* Create an object layer if one doesn't already exist
* Use the `Insert Point` tool to add a point object to the layer
* Set the `Type` of the object to `action`
* Add a custom property `text`
* Set the `text` property to the string to display to the player

### Properties

#### Required

| property | value                                             | description                                                  |
|----------|---------------------------------------------------|--------------------------------------------------------------|
| text     | any unique string                                 | text displayed to the user when the action appears in the UI |
| fitness  | [fitness strategy value](#fitness_strategy_value) | defines when an action can be taken                          |
| lifespan | [lifespan value](#lifespane_value)                | defines how long an action lives                             |

#### <a name="fitness_strategy_value">Fitness Strategy Values</a>
| value                                | description                                                               |
|--------------------------------------|---------------------------------------------------------------------------|
| has_inventory                        | is fit if a taker has an inventory                                        |
| has_item {string}                    | is fit if a taker has an inventory containing an item with the given name |
| any [_strategy1_, __strategy2_, ...] | is fit if any of the enclosed fitness strategies are fit                  |
| all [_strategy1_, __strategy2_, ...] | is fit if all of the enclosed fitness strategies are fit                  |

#### <a name="lifespan_value">Lifespan Values</a>
| value   | description                                  |
|---------|----------------------------------------------|
| forever | the action lives forever                     |
| {int}   | the number of times this action may be taken |
|         |                                              |

## Action Effects
Once an action is taken it is up to a game system to carry out its effects.
TODO: Write more about action effects.
