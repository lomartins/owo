use crate::state::AppState;
use axum::{middleware, routing::*, Router};
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

pub mod accounts;
pub mod audit;
pub mod auth;
pub mod backup;
pub mod bills;
pub mod budgets;
pub mod cards;
pub mod categories;
pub mod goals;
pub mod health;
pub mod investments;
pub mod loans;
pub mod profile;
pub mod reports;
pub mod sync;
pub mod tags;
pub mod transactions;

pub struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let comp = openapi.components.get_or_insert_with(Default::default);
        comp.add_security_scheme(
            "bearer",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "owo API", version = "0.1.0", description = "Personal finance backend"),
    modifiers(&SecurityAddon),
    paths(
        health::check,
        auth::register, auth::login, auth::logout, auth::me, auth::update_profile,
        auth::change_password, auth::list_sessions, auth::revoke_session, auth::revoke_other_sessions,
        profile::upload_photo,
        accounts::list, accounts::create, accounts::show, accounts::update, accounts::delete, accounts::balance,
        cards::list, cards::create,
        categories::list, categories::create, categories::update, categories::delete,
        tags::list, tags::create,
        transactions::list, transactions::create, transactions::update, transactions::delete, transactions::transfer,
        bills::list, bills::create, bills::update, bills::delete, bills::pay, bills::reset,
        investments::list, investments::create,
        loans::list, loans::create,
        budgets::list, budgets::create, budgets::update, budgets::delete,
        goals::list, goals::create,
        reports::monthly, reports::cash_flow, reports::by_category, reports::net_worth,
        sync::push, sync::pull,
        audit::list,
        backup::create, backup::status, backup::download, backup::restore,
    ),
    components(schemas(
        crate::domain::common::Page,
        crate::domain::common::Money,
        crate::domain::user::User, crate::domain::user::RegisterRequest, crate::domain::user::LoginRequest,
        crate::domain::user::PasswordChangeRequest, crate::domain::user::ProfileUpdate,
        crate::domain::user::AuthResponse, crate::domain::user::SessionToken, crate::domain::user::UserResponse,
        crate::domain::user::PhotoUploadRequest, crate::domain::user::PhotoUploadResponse,
        crate::domain::session::Session,
        crate::domain::account::Account, crate::domain::account::CreateAccount,
        crate::domain::account::UpdateAccount, crate::domain::account::AccountBalance,
        crate::domain::card::Card, crate::domain::card::CreateCard, crate::domain::card::CardInvoice,
        crate::domain::category::Category, crate::domain::category::CreateCategory,
        crate::domain::category::UpdateCategory,
        crate::domain::tag::Tag, crate::domain::tag::CreateTag,
        crate::domain::transaction::Transaction, crate::domain::transaction::CreateTransaction,
        crate::domain::transaction::UpdateTransaction, crate::domain::transaction::CreateTransfer,
        crate::domain::bill::Bill, crate::domain::bill::CreateBill, crate::domain::bill::PayBill,
        crate::api::bills::UpdateBill, crate::api::bills::PayBillRequest,
        crate::domain::investment::Investment, crate::domain::investment::CreateInvestment,
        crate::domain::investment::UpdateValue,
        crate::domain::loan::Loan, crate::domain::loan::CreateLoan, crate::domain::loan::PayInstallment,
        crate::domain::budget::Budget, crate::domain::budget::CreateBudget,
        crate::domain::budget::UpdateBudget, crate::domain::budget::BudgetRow,
        crate::domain::budget::BudgetMonth,
        crate::domain::goal::Goal, crate::domain::goal::CreateGoal,
        crate::api::reports::MonthlyReport,
        crate::api::reports::NetWorthReport, crate::api::reports::NetWorthPoint,
    )),
    tags(
        (name = "auth"), (name = "accounts"), (name = "cards"), (name = "categories"),
        (name = "tags"), (name = "transactions"), (name = "bills"), (name = "investments"),
        (name = "loans"), (name = "budgets"), (name = "goals"), (name = "reports"),
        (name = "sync"), (name = "audit"), (name = "backup"), (name = "health"),
    )
)]
pub struct ApiDoc;

pub fn router(state: AppState) -> Router {
    let auth_required = Router::new()
        .route("/auth/me", get(auth::me).patch(auth::update_profile))
        .route("/profile/photo", post(profile::upload_photo))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/password", post(auth::change_password))
        .route("/auth/sessions", get(auth::list_sessions).delete(auth::revoke_other_sessions))
        .route("/auth/sessions/:id", delete(auth::revoke_session))
        .route("/accounts", get(accounts::list).post(accounts::create))
        .route(
            "/accounts/:id",
            get(accounts::show).patch(accounts::update).delete(accounts::delete),
        )
        .route("/accounts/:id/balance", get(accounts::balance))
        .route("/cards", get(cards::list).post(cards::create))
        .route("/categories", get(categories::list).post(categories::create))
        .route("/categories/:id", patch(categories::update).delete(categories::delete))
        .route("/tags", get(tags::list).post(tags::create))
        .route("/transactions", get(transactions::list).post(transactions::create))
        .route("/transactions/:id", patch(transactions::update).delete(transactions::delete))
        .route("/transactions/transfer", post(transactions::transfer))
        .route("/bills", get(bills::list).post(bills::create))
        .route("/bills/:id", patch(bills::update).delete(bills::delete))
        .route("/bills/:id/pay", post(bills::pay))
        .route("/bills/:id/reset", post(bills::reset))
        .route("/investments", get(investments::list).post(investments::create))
        .route("/loans", get(loans::list).post(loans::create))
        .route("/budgets", get(budgets::list).post(budgets::create))
        .route("/budgets/:id", patch(budgets::update).delete(budgets::delete))
        .route("/goals", get(goals::list).post(goals::create))
        .route("/reports/monthly", get(reports::monthly))
        .route("/reports/cash-flow", get(reports::cash_flow))
        .route("/reports/by-category", get(reports::by_category))
        .route("/reports/net-worth", get(reports::net_worth))
        .route("/sync/push", post(sync::push))
        .route("/sync/pull", get(sync::pull))
        .route("/audit", get(audit::list))
        .route("/backup/create", post(backup::create))
        .route("/backup/jobs/:id", get(backup::status))
        .route("/backup/download/:id", get(backup::download))
        .route("/backup/restore", post(backup::restore))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::auth_middleware::require_auth,
        ));

    let public = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login));

    Router::new()
        .route("/health", get(health::check))
        .nest("/api/v1", public.merge(auth_required))
        .with_state(state)
}
