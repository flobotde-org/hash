use std::fmt;

use crate::store::postgres::query::{Column, ColumnAccess, Table, TableName, Transpile};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    pub current_column_access: ColumnAccess<'static>,
    pub join_table_name: TableName,
    pub join_column_access: ColumnAccess<'static>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct JoinExpression<'q> {
    pub join: Column<'q>,
    pub on: Column<'q>,
}

impl<'q> JoinExpression<'q> {
    #[must_use]
    pub const fn new(join: Column<'q>, on: Column<'q>) -> Self {
        Self { join, on }
    }
}

impl Transpile for JoinExpression<'_> {
    fn transpile(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // TODO: https://app.asana.com/0/1202805690238892/1203324626226299/f
        fmt.write_str("LEFT JOIN ")?;
        if self.join.table.alias.is_some() {
            let unaliased_table = Table {
                name: self.join.table.name,
                alias: None,
            };
            unaliased_table.transpile(fmt)?;
            fmt.write_str(" AS ")?;
        }
        self.join.table.transpile(fmt)?;

        fmt.write_str(" ON ")?;
        self.join.transpile(fmt)?;
        fmt.write_str(" = ")?;
        self.on.transpile(fmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::postgres::query::{ColumnAccess, TableAlias, TableName};

    #[test]
    fn transpile_join_expression() {
        assert_eq!(
            JoinExpression::new(
                Column {
                    table: Table {
                        name: TableName::TypeIds,
                        alias: None,
                    },
                    access: ColumnAccess::Table {
                        column: "version_id"
                    },
                },
                Column {
                    table: Table {
                        name: TableName::DataTypes,
                        alias: None,
                    },
                    access: ColumnAccess::Table {
                        column: "version_id"
                    },
                },
            )
            .transpile_to_string(),
            r#"LEFT JOIN "type_ids" ON "type_ids"."version_id" = "data_types"."version_id""#
        );

        assert_eq!(
            JoinExpression::new(
                Column {
                    table: Table {
                        name: TableName::TypeIds,
                        alias: Some(TableAlias {
                            condition_index: 0,
                            chain_depth: 1,
                            number: 2,
                        }),
                    },
                    access: ColumnAccess::Table {
                        column: "version_id"
                    },
                },
                Column {
                    table: Table {
                        name: TableName::DataTypes,
                        alias: None,
                    },
                    access: ColumnAccess::Table {
                        column: "version_id"
                    },
                },
            )
            .transpile_to_string(),
            r#"LEFT JOIN "type_ids" AS "type_ids_0_1_2" ON "type_ids_0_1_2"."version_id" = "data_types"."version_id""#
        );
    }
}
