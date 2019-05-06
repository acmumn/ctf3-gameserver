table! {
    check_ups (id, team_id, service_name) {
        id -> Int4,
        team_id -> Int4,
        service_name -> Varchar,
        in_progress -> Bool,
        up -> Bool,
        timestamp -> Timestamp,
    }
}

table! {
    flags (tick, team_id, service_name) {
        tick -> Int4,
        team_id -> Int4,
        service_name -> Varchar,
        flag -> Varchar,
        flag_id -> Nullable<Text>,
        in_progress -> Bool,
        claimed_by -> Nullable<Int4>,
        defended -> Bool,
        created -> Timestamp,
    }
}

table! {
    services (name) {
        name -> Varchar,
        port -> Int4,
        atk_score -> Int4,
        def_score -> Int4,
        up_score -> Int4,
    }
}

table! {
    teams (id) {
        id -> Int4,
        arbitrary_bonus_points -> Int4,
        ip -> Int4,
    }
}

table! {
    tick (id) {
        id -> Int4,
        start_time -> Timestamp,
        current_tick -> Int4,
        current_check -> Int4,
    }
}

joinable!(check_ups -> services (service_name));
joinable!(check_ups -> teams (team_id));
joinable!(flags -> services (service_name));
joinable!(flags -> teams (team_id));

allow_tables_to_appear_in_same_query!(check_ups, flags, services, teams, tick,);
