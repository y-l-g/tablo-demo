//! Status tabs: the list tabs a list over an option field offers, one per option.

use tablo::{FormScalar, ListTab, Options, Table};
use toasty::stmt::Expr;

/// The tabs a list over an option field offers above its rows: `all`, which admits every row, then
/// one tab per option in declaration order, admitting the rows that hold it and showing their
/// count.
pub trait OptionTabs<M> {
    /// Offer those tabs, with `field` admitting the rows that hold the option a tab names.
    fn option_tabs<T>(self, field: impl Fn(T) -> Expr<bool>) -> Self
    where
        T: Options + FormScalar;
}

impl<M> OptionTabs<M> for Table<M> {
    fn option_tabs<T>(self, field: impl Fn(T) -> Expr<bool>) -> Self
    where
        T: Options + FormScalar,
    {
        T::options().into_iter().fold(
            self.tab(ListTab::new("all", tablo::text!("list-tab-all"))),
            |table, (value, label)| {
                let Ok(option) = T::parse_form(&value) else {
                    return table;
                };
                table.tab(ListTab::new(value, label).query(field(option)).badge())
            },
        )
    }
}
