use axum::Json;
use warp::Filter;

#[derive(Deserialize)]
struct Form {
  team_id: i32,
  flag: String,
}

pub async fn submit_flag2(Json(payload): Json<Form>) {
  // Authentication is probably not needed for now, because submitting flags as
  // someone else just sabotages yourself in the game
  //
  // If we're deploying into the world wide web tho, we would want some higher
  // security to prevent abuse, etc.
}

/*
#[deprecated]
pub fn submit_flag() -> Resp!() {
  warp::ext::get::<Db>()
    .and(warp::body::form())
    .and_then(|db: Db, form: Form| {
      db.transaction(|| {
        // look for this flag
        let flag = db.lookup_flag(&form.flag)?;

        // check if this flag has already been claimed by another team
        // also don't claim your own flag
        if flag.claimed_by.is_some() || flag.team_id == form.team_id {
        } else {
          db.claim_flag(&flag, form.team_id)?;
        }

        Ok(())
      })
      .map_err(warp::reject::custom)
    })
    .map(|_| {
      let thanks = "thanks".to_owned();
      warp::reply::json(&thanks)
    })
    .boxed()
}
*/
