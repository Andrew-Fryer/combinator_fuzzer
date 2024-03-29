use std::{collections::{HashMap, HashSet}, rc::Rc, borrow::Borrow, fmt::format};

use crate::{core::{DataModel, RcDataModel, context::Context, Parser, Vectorizer, Serializer, Ast, Fuzzer, Cloneable, Breed, bit_array::BitArray, feature_vector::FeatureVector, DataModelBase, Named, Contextual, context::Children, ParseError}, impl_into_RcDataModel};


#[derive(Debug)]
pub struct Union {
    base: Rc<DataModelBase>, // todo: I should have a static DataModelBase for each thing in library. Then, we store a Rc<DataModelBase> in each DataModel...
    // bnt: BranchingNonTerminal,
    potential_children: Rc<Vec<Rc<dyn DataModel>>>,
    child: Rc<dyn DataModel>,
}

impl Union {
    pub fn new_no_name(potential_children: Rc<Vec<Rc<dyn DataModel>>>, child: Rc<dyn DataModel>) -> Self {
        Self {
            base: Rc::new(DataModelBase::new("Union".to_string())),
            potential_children,
            child,
        }
    }
    pub fn new(name: &str, potential_children: Vec<RcDataModel>) -> RcDataModel {
        let child = potential_children[0].clone();
        let mut result = Self::new_no_name(Rc::new(potential_children), child);
        result.set_name(name);
        Rc::new(result)
    }
    // todo: this should probably be an interface or something...
    // I think this is meant for making this better, but it still sucks IMHO: https://docs.rs/delegate/latest/delegate/#
    pub fn name(&self) -> &String {
        self.base.name()
    }
}

// pub fn union(name: &str, potential_children: Vec<RcDataModel>, child: Rc<dyn DataModel>) -> RcDataModel {
//     Rc::new(Union::new(name, Rc::new(potential_children), child))
// }

impl DataModel for Union {}

impl Contextual for Union {
    fn child(&self) -> Rc<dyn DataModel> {
        self.child.clone()
    }
}

impl Cloneable for Union {
    fn clone(&self) -> Box<dyn DataModel> {
        Box::new(Self{
            base: self.base.clone(),
            potential_children: self.potential_children.clone(),
            child: self.child.clone(),
        })
    }
}

impl Breed for Union {
    fn breed(&self, other: Box<dyn DataModel>) -> Box<dyn DataModel> {
        todo!();
    }
}

impl Parser for Union {
    fn parse(&self, input: &mut BitArray, ctx: &Rc<Context>) -> Result<Box<dyn DataModel>, ParseError> {
        let mut successful_children = Vec::new();
        let mut failures = Vec::new();
        for c in &*self.potential_children {
            let mut input_for_child = input.clone();
            let child_ctx = Context::new(Rc::downgrade(ctx), Children::Zilch);
            match c.parse(&mut input_for_child, &Rc::new(child_ctx)) {
                Ok(new_child) => {
                    successful_children.push((new_child, input_for_child));
                }
                Err(e) => {
                    failures.push(e);
                }
            }
        }
        if successful_children.len() > 1 {
            // println!("Warning: found ambiguity! {:?}", successful_children.iter().map(|c| c.debug()));
            // println!("Warning: found ambiguity!");
        }
        while successful_children.len() > 1 {
            successful_children.pop();
        }
        if let Some((child, input_from_child)) = successful_children.pop() {
            // this is so that we mutate `input` correctly
            input.advance_to_match(input_from_child);
            Ok(Box::new(Self {
                base: self.base.clone(),
                potential_children: self.potential_children.clone(),
                child: Rc::from(child),
            }))
        } else {
            // println!("failed to parse {:} at {:?}", self.name(), input);
            // TODO: should ParseError also record the base name?
            //     I think maybe Context should instead...
            Err(ParseError::Children(failures))
        }
    }
}

impl Ast for Union {
    fn debug(&self) -> String {
        "".to_string()
    }
}

impl Fuzzer for Union {
    fn fuzz(&self) -> Vec<Rc<dyn DataModel>> {
        self.child.fuzz().iter().map(|mutated_child| {
            let mutated_self: Rc<dyn DataModel> = Rc::new(Self {
                base: self.base.clone(),
                potential_children: self.potential_children.clone(),
                child: mutated_child.clone(),
            });
            mutated_self
        }).collect()
    }
}

impl Named for Union {
    fn name(&self) -> &String {
        self.base.name()
    }
    fn set_name(&mut self, name: &str) {
        self.base = Rc::new(DataModelBase::new(name.to_string()));
    }
}

// TODO: don't have duplicate code between here and Constraint
impl Vectorizer for Union {
    fn do_features(&self, features: &mut HashSet<String>) {
        features.insert(self.name().to_string());
        for c in self.potential_children.iter() {
            c.do_features(features);
        }
    }
    fn do_vectorization(&self, fv: &mut FeatureVector, depth: i32) {
        fv.tally(self.name(), depth);
        self.child.do_vectorization(fv, depth);
    }
}

impl Serializer for Union {
    fn do_serialization(&self, ba: &mut BitArray) {
        self.child.do_serialization(ba);
    }
}

// impl From<Union> for Rc<dyn DataModel> {
//     fn from(dm: Union) -> Rc<dyn DataModel> {
//         Rc::new(dm)
//     }
// }

impl_into_RcDataModel!(Union);
