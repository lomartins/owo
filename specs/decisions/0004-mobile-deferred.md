# ADR 0004 — Mobile KMP untouched this pass

Status: Accepted (2026-05-27)

## Context

The repo has a partially scaffolded KMP mobile app (`mobile/feature/home/`, dashboard / onboarding / nav components per recent commits). The two-legged transaction refactor + account-type changes invalidate the current API surface the mobile client would consume. The MVP spec requires a responsive web SPA; mobile is not in MVP scope.

## Decision

- Do not touch mobile this pass.
- Backend refactor proceeds. Web SPA built fresh against the new API.
- Mobile gets a follow-up pass after the backend stabilises.

## Why

- Spec is explicit: web SPA satisfies the "non-CLI client" requirement for MVP.
- Touching mobile now forces a third client to track every breaking change of the foundational refactor.
- The KMP shared layer is small enough that re-wiring it in one focused pass later is cheaper than ratcheting it every commit during the refactor.

## Trade-offs

- Mobile is broken-by-omission until the follow-up pass. Acceptable; nothing depends on it shipping for MVP.

## Follow-up trigger

When `cargo test` passes on the Phase B endpoints and the web SPA is live, open `phases/E-mobile-resync.md` and wire the KMP shared client to the new endpoints.
