// @generated automatically by Diesel CLI.

diesel::table! {
    admins (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        phone -> Nullable<Text>,
        email -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    announcements (id) {
        id -> Uuid,
        #[max_length = 150]
        title -> Varchar,
        community_id -> Nullable<Uuid>,
        message -> Text,
        sent_at -> Timestamp,
    }
}

diesel::table! {
    auth_tokens (id) {
        user_id -> Uuid,
        id -> Uuid,
        time_added -> Timestamp,
        active -> Bool,
        time_last_used -> Timestamp,
        device -> Nullable<Text>,
        browser -> Nullable<Text>,
        version -> Nullable<Text>,
        cpu_arch -> Nullable<Text>,
    }
}

diesel::table! {
    common_areas (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
        community_id -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    communities (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 25]
        short_name -> Nullable<Varchar>,
        address -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    elections (id) {
        id -> Uuid,
        community_id -> Nullable<Uuid>,
        #[max_length = 150]
        title -> Varchar,
        description -> Nullable<Text>,
        start_date -> Timestamp,
        end_date -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    incidents (id) {
        id -> Uuid,
        resident_id -> Nullable<Uuid>,
        community_id -> Nullable<Uuid>,
        description -> Text,
        #[max_length = 20]
        status -> Varchar,
        report_date -> Timestamp,
        resolution_date -> Nullable<Timestamp>,
        notes -> Nullable<Text>,
    }
}

diesel::table! {
    invoices (id) {
        id -> Uuid,
        resident_id -> Uuid,
        community_id -> Nullable<Uuid>,
        issue_date -> Date,
        due_date -> Date,
        amount -> Numeric,
        #[max_length = 20]
        status -> Varchar,
        paid_date -> Nullable<Date>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    maintenance_schedules (id) {
        id -> Uuid,
        community_id -> Nullable<Uuid>,
        description -> Text,
        scheduled_date -> Timestamp,
        #[max_length = 20]
        status -> Varchar,
        details -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    parcels (id) {
        id -> Uuid,
        resident_id -> Uuid,
        #[max_length = 50]
        parcel_type -> Varchar,
        description -> Nullable<Text>,
        arrival_date -> Timestamp,
        received -> Bool,
        received_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    reservations (id) {
        id -> Uuid,
        resident_id -> Uuid,
        common_area_id -> Uuid,
        reservation_date -> Timestamp,
        start_time -> Timestamp,
        end_time -> Timestamp,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    resident_invites (id) {
        id -> Uuid,
        email -> Text,
        community_id -> Uuid,
        key -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    residents (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        #[max_length = 20]
        unit_number -> Nullable<Varchar>,
        address -> Nullable<Text>,
        phone -> Nullable<Text>,
        email -> Text,
        date_of_birth -> Nullable<Date>,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_roles (id) {
        id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        community_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        entity_id -> Uuid,
        #[max_length = 10]
        entity_type -> Varchar,
        admin_id -> Nullable<Uuid>,
        resident_id -> Nullable<Uuid>,
        password -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    vehicles (id) {
        id -> Uuid,
        resident_id -> Uuid,
        #[max_length = 20]
        license_plate -> Varchar,
        #[max_length = 100]
        model -> Nullable<Varchar>,
        #[max_length = 50]
        color -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    votes (id) {
        id -> Uuid,
        election_id -> Uuid,
        resident_id -> Uuid,
        #[max_length = 50]
        vote_option -> Varchar,
        voted_at -> Timestamp,
    }
}

diesel::joinable!(announcements -> communities (community_id));
diesel::joinable!(auth_tokens -> users (user_id));
diesel::joinable!(common_areas -> communities (community_id));
diesel::joinable!(elections -> communities (community_id));
diesel::joinable!(incidents -> communities (community_id));
diesel::joinable!(incidents -> residents (resident_id));
diesel::joinable!(invoices -> communities (community_id));
diesel::joinable!(invoices -> residents (resident_id));
diesel::joinable!(maintenance_schedules -> communities (community_id));
diesel::joinable!(parcels -> residents (resident_id));
diesel::joinable!(reservations -> common_areas (common_area_id));
diesel::joinable!(reservations -> residents (resident_id));
diesel::joinable!(resident_invites -> communities (community_id));
diesel::joinable!(user_roles -> communities (community_id));
diesel::joinable!(user_roles -> users (user_id));
diesel::joinable!(users -> admins (admin_id));
diesel::joinable!(users -> residents (resident_id));
diesel::joinable!(vehicles -> residents (resident_id));
diesel::joinable!(votes -> elections (election_id));
diesel::joinable!(votes -> residents (resident_id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    announcements,
    auth_tokens,
    common_areas,
    communities,
    elections,
    incidents,
    invoices,
    maintenance_schedules,
    parcels,
    reservations,
    resident_invites,
    residents,
    user_roles,
    users,
    vehicles,
    votes,
);
