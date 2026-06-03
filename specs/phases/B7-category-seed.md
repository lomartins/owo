# Phase B7 — Category seed on register

Status: shipped (2026-05-27)

## What shipped

Folded into `services::user_provisioning::provision_user` alongside the bucket-account provisioning (Phase A3). On `auth::register`, the 10 MVP categories are inserted in the same atomic transaction as the user row and the revenue/expense buckets.

Seed list: Food, Leisure, Transport, Health, Education, Clothes, Home, Pet, Subscriptions, Other. All kind = `EXPENSE`.

## Verification

Covered by `phases/A3-account-types.md`.

## Follow-ups

- If we want INCOME-kind categories pre-seeded (e.g. Salary, VA), add them with kind `INCOME`. Out of MVP — users can create them via `POST /categories`.
