use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Meta, Path, parse_macro_input};

#[proc_macro_derive(DbOps, attributes(diesel))]
pub fn derive_db_ops(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // Extract the Diesel table name from the struct attributes
    let diesel_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("diesel"));

    let table_path = diesel_attr
        .and_then(|attr| {
            attr.parse_args::<Meta>().ok().and_then(|meta| {
                if let Meta::NameValue(mnv) = meta {
                    if mnv.path.is_ident("table_name") {
                        if let syn::Expr::Path(expr_path) = &mnv.value {
                            return Some(expr_path.path.clone());
                        }
                    }
                }
                None
            })
        })
        .expect("Couldn't find #[diesel(table_name = crate::schema::table)] attribute.");

    // Extract table identifier (last segment of path)
    let table_ident = &table_path
        .segments
        .last()
        .expect("Valid table path required")
        .ident;

    // Prepare expanded code block
    let expanded = quote! {
        impl DatabaseTrait for #struct_name {
            type Id = uuid::Uuid;
            type Table = #table_path::table;

            fn table() -> Self::Table {
                #table_path::table
            }

            fn new_id(conn: &mut PgConnection) -> uuid::Uuid {
                let mut uuid_new = uuid::Uuid::new_v4();
                let mut exists = true;
                let mut tries = 0;

                while exists && tries < 10 {
                    match Self::table().filter(#table_path::id.eq(uuid_new)).count().get_result::<i64>(conn) {
                        Ok(count) => {
                            if count == 0 {
                                exists = false;
                            } else {
                                uuid_new = uuid::Uuid::new_v4();
                            }
                        },
                        Err(e) => {
                            tries += 1;
                            uuid_new = uuid::Uuid::new_v4();
                        }
                    };
                };

                uuid_new
            }

            fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
                diesel::insert_into(Self::table()).values(self).execute(conn)
            }

            fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
                diesel::update(Self::table().filter(#table_path::id.eq(&self.id)))
                    .set(self)
                    .execute(conn)
            }

            fn db_upsert(&self, conn: &mut PgConnection) -> diesel::QueryResult<usize> {
                diesel::insert_into(Self::table())
                    .values(self)
                    .on_conflict(#table_path::id)
                    .do_update()
                    .set(self)
                    .execute(conn)
            }

            fn db_read_by_id(conn: &mut PgConnection, record_id: Self::Id) -> diesel::QueryResult<Self> {
                Self::table()
                   .filter(#table_path::id.eq(record_id))
                   .first(conn)
            }

            fn db_delete_by_id(conn: &mut PgConnection, record_id: Self::Id) -> diesel::QueryResult<usize> {
                diesel::delete(Self::table().filter(#table_path::id.eq(record_id))).execute(conn)
            }

            fn db_delete_all(conn: &mut PgConnection) -> diesel::QueryResult<usize> {
                diesel::delete(Self::table()).execute(conn)
            }

            fn db_count_all(conn: &mut PgConnection) -> diesel::QueryResult<i64> {
                Self::table().count().get_result(conn)
            }
        }

        impl DatabaseTraitVec for Vec<#struct_name> {
            type Id = uuid::Uuid;
            type Table = #table_path::table;

            fn table() -> Self::Table {
                #table_path::table
            }

            fn db_insert(&self, conn: &mut PgConnection) -> diesel::QueryResult<usize> {
                diesel::insert_into(Self::table())
                    .values(self)
                    .execute(conn)
            }

            fn db_update(&self, conn: &mut PgConnection) -> diesel::QueryResult<usize> {
                let mut total = 0;
                for item in self {
                    total += diesel::update(Self::table().filter(#table_path::id.eq(item.id)))
                        .set(item)
                        .execute(conn)?;
                }
                Ok(total)
            }

            fn db_upsert(&self, conn: &mut PgConnection) -> diesel::QueryResult<usize> {
                let mut total = 0;
                for item in self {
                    total += item.db_upsert(conn)?;
                }
                Ok(total)
            }

            fn db_read_by_id(conn: &mut PgConnection, ids: Vec<Self::Id>) -> diesel::QueryResult<Self> {
                Self::table()
                    .filter(#table_path::id.eq_any(ids))
                    .load(conn)
            }

            fn db_read_all(conn: &mut PgConnection) -> diesel::QueryResult<Self> {
                Self::table().load(conn)
            }

            fn db_read_by_range(conn: &mut PgConnection, per_page: i64, offset: i64) -> diesel::QueryResult<Self> {
                Self::table()
                    .limit(per_page)
                    .offset(offset)
                    .load(conn)
            }

            fn db_delete_by_id(conn: &mut PgConnection, ids: Vec<Self::Id>) -> diesel::QueryResult<usize> {
                diesel::delete(Self::table().filter(#table_path::id.eq_any(ids)))
                    .execute(conn)
            }
        }
    };

    TokenStream::from(expanded)
}
