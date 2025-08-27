use sqlx::Error;

const DUPLICATE_KEY_VALUE_VIOLATES_UNIQUE_CONSTRAINT: &str =
    "duplicate key value violates unique constraint";

pub fn is_unique_constraint_error(err: &Error, table: &str, column: &str) -> bool {
    if let Error::Database(db_err) = err
        && let Some(pg_err) = db_err.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
    {
        let message = pg_err.message();
        return message.contains(DUPLICATE_KEY_VALUE_VIOLATES_UNIQUE_CONSTRAINT)
            && message.contains(&format!("{table}_{column}_key"));
    }
    false
}
