pub mod get_invoice;
pub mod upsert_invoice;

use super::prelude::*;
type InvoiceListHttpResponse = HttpResponseObject<Vec<invoice_model::InvoiceModel>>;
type InvoiceGetHttpResponse = HttpResponseObject<invoice_model::InvoiceModel>;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_invoice::get_invoices,
        get_invoice::get_invoice_by_id,
        upsert_invoice::new_invoice,
        upsert_invoice::update_invoice,
        upsert_invoice::delete_invoice,
    ),
    components(schemas(
        invoice_model::InvoiceModel,
        invoice_model::InvoiceModelNew,
        invoice_model::InvoiceStatus
    ))
)]
pub struct InvoiceApi;
