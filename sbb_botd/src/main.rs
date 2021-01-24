#[macro_use]
extern crate diesel;

use diesel::prelude::*;

use sbb_data::*;

mod function {
    use diesel::sql_types::*;
    no_arg_sql_function!(random, Integer, "SQL RANDOM() function");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_conn = sbb_data::connect_env()?;
    println!("{:?}", select_robot(&db_conn));
    Ok(())
}

fn select_robot(db_conn: &PgConnection) -> QueryResult<Robot> {
    if let Some(robot) = scheduled_robot(db_conn)? {
        return Ok(robot);
    }
    random_robot(db_conn)
}

fn scheduled_robot(db_conn: &PgConnection) -> QueryResult<Option<Robot>> {
    use diesel::dsl::{now, date, exists};
    use schema::*;
    let res = robots::table.filter(
            exists(scheduled_dailies::table
                .filter(robots::id.eq(scheduled_dailies::robot_id)
                    .and(scheduled_dailies::post_on.eq(date(now))))))
        .first(db_conn);
    match res {
        Ok(robot) => Ok(Some(robot)),
        Err(diesel::NotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

fn random_robot(db_conn: &PgConnection) -> QueryResult<Robot> {
    use diesel::dsl::{now, date, exists, not, IntervalDsl};
    use schema::*;
    const NO_REPEAT_DAYS: i32 = 14;
    robots::table.filter(
            not(exists(past_dailies::table
                .filter(robots::id.eq(past_dailies::robot_id)
                    .and(past_dailies::posted_on.ge(date(now - NO_REPEAT_DAYS.days())))))))
        .order(function::random)
        .first(db_conn)
}
