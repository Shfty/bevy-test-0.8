use crate::prelude::{AddInputs, AddOutputs};

pub trait Edges {
    type Inputs: AddInputs;
    type Outputs: AddOutputs;
}

