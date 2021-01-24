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
    println!("{:?}", select_robot(&db_conn, Some(14)));
    Ok(())
}

fn select_robot(db_conn: &PgConnection, no_repeat_days: Option<i32>) -> QueryResult<Robot> {
    if let Some(robot) = scheduled_robot(db_conn)? {
        return Ok(robot);
    }
    random_robot(db_conn, no_repeat_days)
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

fn random_robot(db_conn: &PgConnection, no_repeat_days: Option<i32>) -> QueryResult<Robot> {
    use diesel::dsl::{now, date, not, IntervalDsl};
    use schema::*;

    let recent_groups: Vec<i32> = match no_repeat_days {
        Some(days) => past_dailies::table
            .inner_join(robots::table)
            .filter(past_dailies::posted_on.ge(date(now - days.days())))
            .select(robots::robot_group_id)
            .distinct()
            .load(db_conn)?,
        None => Vec::new(),
    };

    robots::table
        .filter(not(robots::robot_group_id.eq_any(&recent_groups)))
        .order(function::random)
        .first(db_conn)
}
