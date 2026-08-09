//! Five items, three of them public.

pub const LIMIT: u8 = 8;

pub enum Choice
{
    First,
    Second,
}

pub trait Contract
{
    fn Run(&self);
}

mod inner {}
