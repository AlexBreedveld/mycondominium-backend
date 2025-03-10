// @generated automatically by Diesel CLI.

diesel::table! {
    announcements (id) {
        id -> Uuid,
        #[max_length = 150]
        title -> Varchar,
        message -> Text,
        sent_at -> Timestamp,
    }
}

diesel::table! {
    common_areas (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    documents (id) {
        id -> Uuid,
        #[max_length = 150]
        title -> Varchar,
        description -> Nullable<Text>,
        file_url -> Text,
        #[max_length = 50]
        document_type -> Nullable<Varchar>,
        shared_at -> Timestamp,
    }
}

diesel::table! {
    elections (id) {
        id -> Uuid,
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
        start_time -> Time,
        end_time -> Time,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        email -> Nullable<Text>,
        date_of_birth -> Nullable<Date>,
        resident_since -> Timestamp,
        is_active -> Bool,
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

diesel::joinable!(incidents -> residents (resident_id));
diesel::joinable!(invoices -> residents (resident_id));
diesel::joinable!(parcels -> residents (resident_id));
diesel::joinable!(reservations -> common_areas (common_area_id));
diesel::joinable!(reservations -> residents (resident_id));
diesel::joinable!(vehicles -> residents (resident_id));
diesel::joinable!(votes -> elections (election_id));
diesel::joinable!(votes -> residents (resident_id));

diesel::allow_tables_to_appear_in_same_query!(
    announcements,
    common_areas,
    documents,
    elections,
    incidents,
    invoices,
    maintenance_schedules,
    parcels,
    reservations,
    residents,
    vehicles,
    votes,
);
