<div align="center">
  <h1>Enviame</h1>
  <h3>Priority Messenger</h3>
  <p>Demo: <a href="https://msg-beta.brianc.tech">https://msg-beta.brianc.tech</a></p>
  <p>
    <picture>
      <a href="https://github.com/differental/enviame/actions/workflows/ci_prod.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/ci_prod.yml?label=Production&style=for-the-badge" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://github.com/differental/enviame/actions/workflows/ci_beta.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/ci_beta.yml?label=Beta&style=for-the-badge">
      </a>
    </picture>
  </p>
  <p>
    <picture>
      <a href="https://github.com/differental/enviame/tree/main">
        <img src="https://img.shields.io/github/last-commit/differental/enviame?style=for-the-badge" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://github.com/differental/enviame/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/differental/enviame?style=for-the-badge&color=499dd0" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://www.rust-lang.org/">
        <img src="https://img.shields.io/badge/Made%20with-rust-red?style=for-the-badge" />
      </a>
    </picture>
  </p>
</div>

## Introduction

***TL;DR** Enviame is a priority messenger, designed as a simple solution to limited message prioritisation with instant messaging applications and protocols.*

For frequent users of "Focus Mode" or equivalent features on various devices, there is often the issue of being unable to selectively allow important messages to go through while not being disturbed by every single notification.

A popular solution is to mute all text messages and allow calls as the "emergency option", but mobile signals are not a guarantee whilst WhatsApp has limited call filtering features.

That's where Enviame comes in. Meaning "(to) send me" in Spanish, the tool is designed to allow prioritised communication from trusted friends and family to go through while blocking all other messages.

Features that integrate Enviame with smartwatches and other delivery options are planned.

## Tech Stack

- **Backend**
  - Rust
  - Axum (framework)
  - Tokio (async runtime)

- **Database**
  - PostgreSQL

- **Frontend**
  - Static HTML
  - "Vanilla" JavaScript

## Database Schema

`messages`:

```text
  column_name   |        data_type         | is_nullable |            column_default            
----------------+--------------------------+-------------+--------------------------------------
 id             | integer                  | NO          | nextval('messages_id_seq'::regclass)
 submitted_time | timestamp with time zone | NO          | CURRENT_TIMESTAMP
 user_uid       | integer                  | YES         | 
 name           | text                     | NO          | 
 email          | text                     | NO          | 
 message        | text                     | NO          | 
 priority       | text                     | NO          | 
 status         | text                     | NO          | 'pending'::text
 sender         | text                     | NO          | 
```

`users`:

```text
 column_name |        data_type         | is_nullable |           column_default           
-------------+--------------------------+-------------+------------------------------------
 uid         | integer                  | NO          | nextval('users_uid_seq'::regclass)
 added_time  | timestamp with time zone | NO          | CURRENT_TIMESTAMP
 name        | text                     | NO          | 
 email       | text                     | NO          | 
 token       | text                     | NO          | 
 verified    | boolean                  | NO          | 
```
