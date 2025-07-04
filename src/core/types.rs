use clap::ValueEnum;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum SortOrder {
    Asc,
    Desc,
}
