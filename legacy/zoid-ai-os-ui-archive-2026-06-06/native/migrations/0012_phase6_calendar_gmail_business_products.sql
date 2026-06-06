-- Phase 6: Calendar, Gmail, Inbox, Business, Products

create table if not exists companies (
    id text primary key,
    name text not null,
    domain text,
    status text not null default 'active' check (status in ('active', 'archived')),
    notes text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp
);

create table if not exists contacts (
    id text primary key,
    company_id text,
    full_name text not null,
    email text,
    phone text,
    role text,
    status text not null default 'active' check (status in ('active', 'archived')),
    notes text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (company_id) references companies(id) on delete set null
);

create table if not exists follow_ups (
    id text primary key,
    subject text not null,
    due_at text,
    state text not null default 'open' check (state in ('open', 'blocked', 'done', 'dismissed')),
    priority text not null default 'normal' check (priority in ('low', 'normal', 'high', 'urgent')),
    contact_id text,
    company_id text,
    product_id text,
    task_id text,
    note_id text,
    email_ref_id text,
    calendar_event_id text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (contact_id) references contacts(id) on delete set null,
    foreign key (company_id) references companies(id) on delete set null,
    foreign key (task_id) references tasks(id) on delete set null,
    foreign key (note_id) references notes(id) on delete set null
);

create table if not exists products (
    id text primary key,
    name text not null,
    status text not null default 'active' check (status in ('idea', 'active', 'paused', 'archived')),
    summary text,
    owner_contact_id text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (owner_contact_id) references contacts(id) on delete set null
);

create table if not exists email_refs (
    id text primary key,
    external_id text,
    thread_id text,
    subject text not null,
    sender text,
    recipients_json text not null default '[]' check (json_valid(recipients_json)),
    snippet text,
    state text not null default 'draft' check (state in ('received', 'draft', 'queued_confirmation', 'sent', 'send_blocked')),
    confirmation_id text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (confirmation_id) references confirmation_decisions(id) on delete set null
);

create table if not exists calendar_refs (
    id text primary key,
    external_id text,
    title text not null,
    starts_at text not null,
    ends_at text not null,
    location text,
    notes text,
    state text not null default 'local_draft' check (state in ('local_draft', 'created', 'updated', 'deleted', 'blocked')),
    confirmation_id text,
    metadata_json text not null default '{}' check (json_valid(metadata_json)),
    created_at text not null default current_timestamp,
    updated_at text not null default current_timestamp,
    foreign key (confirmation_id) references confirmation_decisions(id) on delete set null
);

create index if not exists idx_contacts_company on contacts(company_id);
create index if not exists idx_follow_ups_state_due on follow_ups(state, due_at);
create index if not exists idx_follow_ups_contact_company_product on follow_ups(contact_id, company_id, product_id);
create index if not exists idx_products_status on products(status);
create index if not exists idx_email_refs_state_thread on email_refs(state, thread_id);
create index if not exists idx_calendar_refs_state_start on calendar_refs(state, starts_at);
