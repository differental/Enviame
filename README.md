<h1 align="center">Enviame</h1>

<p align="center">
  <a href="https://github.com/differental/enviame/actions/workflows/deploy_prod.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/deploy_prod.yml?label=Production&style=for-the-badge" />
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://github.com/differental/enviame/actions/workflows/deploy_main.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/deploy_main.yml?label=Beta&style=for-the-badge" />
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Made%20with-rust-red?style=for-the-badge" />
  </a>
</p>

<p align="center">Priority Messenger</p>

<p align="center">Demo: <a href="https://msg-beta.brianc.tech">https://msg-beta.brianc.tech</a></p>

## Introduction

***TL;DR** Enviame is a priority messenger, designed as a simple solution to limited message prioritisation with instant messaging applications and protocols.*

For frequent users of "Focus Mode" or equivalent features on various devices, there is often the issue of being unable to selectively allow important messages to go through while not being disturbed by every single notification.

A popular solution is to mute all text messages and allow calls as the "emergency option", but mobile signals are not a guarantee whilst WhatsApp has limited call filtering features.

That's where Enviame comes in. Meaning "(to) send me" in Spanish, the tool is designed to allow prioritised communication from trusted friends and family to go through while blocking all other messages.

Features that integrate Enviame with smartwatches and other delivery options are planned.

## Tech Stack

- Backend: Rust (w/ Axum), PostgreSQL
- Frontend: Static HTML (w/ Bootstrap UI)
