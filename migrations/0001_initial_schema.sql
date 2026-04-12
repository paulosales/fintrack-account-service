-- Account types (e.g. Checking, Savings, Credit)
CREATE TABLE IF NOT EXISTS account_types (
    id   BIGINT       NOT NULL AUTO_INCREMENT,
    code VARCHAR(50)  NOT NULL,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_account_types_code (code)
);

-- Bank / credit-card accounts
CREATE TABLE IF NOT EXISTS accounts (
    id              BIGINT       NOT NULL AUTO_INCREMENT,
    code            VARCHAR(50)  NOT NULL,
    name            VARCHAR(255) NOT NULL,
    account_type_id BIGINT       NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_accounts_code (code),
    CONSTRAINT fk_accounts_account_type
        FOREIGN KEY (account_type_id) REFERENCES account_types (id)
);

-- Transaction types (e.g. Expense, Income, Transfer)
CREATE TABLE IF NOT EXISTS transaction_types (
    id   BIGINT       NOT NULL AUTO_INCREMENT,
    code VARCHAR(50)  NOT NULL,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_transaction_types_code (code)
);

-- Spending / income categories
CREATE TABLE IF NOT EXISTS categories (
    id   BIGINT       NOT NULL AUTO_INCREMENT,
    name VARCHAR(255) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_categories_name (name)
);

-- Financial transactions
CREATE TABLE IF NOT EXISTS transactions (
    id                  BIGINT        NOT NULL AUTO_INCREMENT,
    account_id          BIGINT        NOT NULL,
    transaction_type_id BIGINT        NOT NULL,
    datetime            DATETIME      NOT NULL,
    amount              DOUBLE        NOT NULL,
    description         VARCHAR(500)  NOT NULL,
    note                VARCHAR(1000),
    fingerprint         VARCHAR(36)   NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_transactions_fingerprint (fingerprint),
    CONSTRAINT fk_transactions_account
        FOREIGN KEY (account_id) REFERENCES accounts (id),
    CONSTRAINT fk_transactions_type
        FOREIGN KEY (transaction_type_id) REFERENCES transaction_types (id)
);

-- Categories assigned to a transaction (many-to-many)
CREATE TABLE IF NOT EXISTS transactions_categories (
    transaction_id BIGINT NOT NULL,
    category_id    BIGINT NOT NULL,
    PRIMARY KEY (transaction_id, category_id),
    CONSTRAINT fk_tc_transaction
        FOREIGN KEY (transaction_id) REFERENCES transactions (id) ON DELETE CASCADE,
    CONSTRAINT fk_tc_category
        FOREIGN KEY (category_id) REFERENCES categories (id)
);

-- Line-item breakdown within a transaction
CREATE TABLE IF NOT EXISTS sub_transactions (
    id             BIGINT        NOT NULL AUTO_INCREMENT,
    transaction_id BIGINT        NOT NULL,
    product_code   VARCHAR(100),
    amount         DOUBLE        NOT NULL,
    description    VARCHAR(500)  NOT NULL,
    note           VARCHAR(1000),
    PRIMARY KEY (id),
    CONSTRAINT fk_sub_transactions_transaction
        FOREIGN KEY (transaction_id) REFERENCES transactions (id) ON DELETE CASCADE
);

-- Categories assigned to a sub-transaction (many-to-many)
CREATE TABLE IF NOT EXISTS sub_transactions_categories (
    sub_transaction_id BIGINT NOT NULL,
    category_id        BIGINT NOT NULL,
    PRIMARY KEY (sub_transaction_id, category_id),
    CONSTRAINT fk_stc_sub_transaction
        FOREIGN KEY (sub_transaction_id) REFERENCES sub_transactions (id) ON DELETE CASCADE,
    CONSTRAINT fk_stc_category
        FOREIGN KEY (category_id) REFERENCES categories (id)
);

-- Recurring / one-off budget templates
CREATE TABLE IF NOT EXISTS budget_setup (
    id               BIGINT        NOT NULL AUTO_INCREMENT,
    account_id       BIGINT        NOT NULL,
    date             DATE          NOT NULL,
    is_repeatle      TINYINT(1)    NOT NULL DEFAULT 0,
    repeat_frequency VARCHAR(20),
    end_date         DATE,
    description      VARCHAR(500)  NOT NULL,
    amount           DOUBLE        NOT NULL,
    note             VARCHAR(1000),
    PRIMARY KEY (id),
    CONSTRAINT fk_budget_setup_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
);

-- Concrete budget entries (generated from budget_setup)
CREATE TABLE IF NOT EXISTS budget (
    id              BIGINT        NOT NULL AUTO_INCREMENT,
    budget_setup_id BIGINT        NOT NULL,
    date            DATE          NOT NULL,
    amount          DOUBLE        NOT NULL,
    description     VARCHAR(500)  NOT NULL,
    processed       TINYINT(1)    NOT NULL DEFAULT 0,
    note            VARCHAR(1000),
    PRIMARY KEY (id),
    CONSTRAINT fk_budget_budget_setup
        FOREIGN KEY (budget_setup_id) REFERENCES budget_setup (id) ON DELETE CASCADE
);
