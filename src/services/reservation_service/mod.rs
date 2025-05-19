pub mod get_reservation;
pub mod upsert_reservation;

use super::prelude::*;
type ReservationListHttpResponse = HttpResponseObject<Vec<reservation_model::ReservationModel>>;
type ReservationGetHttpResponse = HttpResponseObject<reservation_model::ReservationModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_reservation::get_reservations,
        get_reservation::get_reservation_by_id,
        upsert_reservation::new_reservation,
        upsert_reservation::update_reservation,
        upsert_reservation::delete_reservation,
    ),
    components(schemas(
        reservation_model::ReservationModel,
        reservation_model::ReservationModelNew,
        reservation_model::ReservationStatus,
    ))
)]
pub struct ReservationApi;
