-- Phase C10: enforce invoice-payment = category-less transfer (asset -> credit_card).
-- The service-layer guard in `api::transactions::validate_leg_pair` already returns
-- 400 on bad payloads (category attached to a transfer); these triggers are the
-- DB-level last line of defense.
-- See specs/08-cards-preserve.md.

CREATE TRIGGER trg_tx_invoice_payment_no_category_insert
BEFORE INSERT ON transactions
FOR EACH ROW
WHEN NEW.category_id IS NOT NULL
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.destination_account_id AND type = 'credit_card')
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.source_account_id      AND type = 'asset')
BEGIN
    SELECT RAISE(ABORT, 'INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY');
END;

CREATE TRIGGER trg_tx_invoice_payment_no_category_update
BEFORE UPDATE OF category_id, source_account_id, destination_account_id ON transactions
FOR EACH ROW
WHEN NEW.category_id IS NOT NULL
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.destination_account_id AND type = 'credit_card')
     AND EXISTS (SELECT 1 FROM accounts WHERE id = NEW.source_account_id      AND type = 'asset')
BEGIN
    SELECT RAISE(ABORT, 'INVOICE_PAYMENT_REQUIRES_NULL_CATEGORY');
END;
