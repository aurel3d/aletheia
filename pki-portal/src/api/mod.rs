pub mod audit;
pub mod certificates;
pub mod health;
pub mod intermediates;
pub mod policy;
pub mod revocations;
pub mod roots;
pub mod trust_bundles;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health)
        .service(
            web::scope("/roots")
                .service(roots::list_roots)
                .service(roots::create_root)
                .service(roots::get_root)
                .service(roots::rotate_root),
        )
        .service(
            web::scope("/intermediates")
                .service(intermediates::list_intermediates)
                .service(intermediates::create_intermediate)
                .service(intermediates::get_intermediate),
        )
        .service(
            web::scope("/certificates")
                .service(certificates::issue_certificate_handler)
                .service(certificates::get_certificate_handler),
        )
        .service(
            web::scope("/revocations")
                .service(revocations::get_revocations_handler)
                .service(revocations::revoke_certificate_handler),
        )
        .service(
            web::scope("/trust-bundles")
                .service(trust_bundles::get_latest_bundle_handler)
                .service(trust_bundles::get_bundle_by_version_handler)
                .service(trust_bundles::publish_bundle_handler),
        )
        .service(
            web::scope("/policy")
                .service(policy::get_policy_handler)
                .service(policy::update_policy_handler),
        )
        .service(
            web::scope("/audit")
                .service(audit::list_events_handler),
        );
}
