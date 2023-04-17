use perseus::{prelude::*, state::rx_collections::RxVecNested};
use sycamore::prelude::*;
use serde::{Serialize, Deserialize } ;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ReactiveState)]
pub struct Recipe {
    pub name: String,
    pub catagory_name: String,
    pub information: Option<String>,
    pub base_amount: f32,
    pub unit_name: String,
    pub preparation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, ReactiveState, Debug)]
pub struct IngredientAndAmount {
    pub recipe_name: String,
    pub ingredient_name: String,
    pub index: u32,
    pub amount: f32,
    pub unit_name: String,
}

impl IngredientAndAmountPerseusRxIntermediate {
    pub fn new(
        recipe_name: RcSignal<String>,
        ingredient_name: &str,
        index: u32,
        amount: f32,
        unit_name: &str,
    ) -> IngredientAndAmountPerseusRxIntermediate 
    {
        let ingredient_name = create_rc_signal(ingredient_name.to_owned());
        let amount = create_rc_signal(amount);
        let unit_name = create_rc_signal(unit_name.to_owned());
        let index = create_rc_signal(index);
        IngredientAndAmountPerseusRxIntermediate {
            recipe_name,
            ingredient_name,
            index,
            amount,
            unit_name,
        }
    }
}

impl PartialEq for IngredientAndAmountPerseusRxIntermediate {
    fn eq(&self, other: &Self) -> bool {
        self.recipe_name.get() == other.recipe_name.get()
            && self.ingredient_name.get() == other.ingredient_name.get()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash, Eq)]
pub struct Catagory {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ingredient
{
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Unit
{
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, ReactiveState, Debug)]
pub struct RecipeInformation {
    #[rx(nested)]
    pub recipe: Recipe,
    #[rx(nested)]
    pub ingredients: RxVecNested<IngredientAndAmount>,
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Image {
    name: String,
    location: String,
}