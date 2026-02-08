use std::ops::Deref;
use std::fmt::Debug;
#[derive(PartialEq, Eq, Debug)]
pub struct Vector {
    data: Vec<i32>,
}

impl Vector {
    pub fn new(data: impl Into<Vec<i32>>) -> Self {
        Vector { data: data.into() }
    }
}

impl Deref for Vector {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


pub fn dot_vector(v1: &Vector, v2: &Vector) -> i32 {
    v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
}

