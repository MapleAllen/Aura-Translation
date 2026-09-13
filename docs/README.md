# Documentation Index

This directory contains current module descriptions, implementation plans, and release checklists for Aura Translation.

## Current Module Docs

- [Aura Guard](Aura-Guard/Aura-Guard-Description.md): clipboard credential screening for automatic Aura-mode translation.
- [Config And Secrets](Config-And-Secrets/Config-And-Secrets-Description.md): preferences, migration, and API key storage.
- [Daemon Core](Daemon-Core/Daemon-Core-Description.md): tray daemon, hotkey, windows, config, and backend commands.
- [Interaction Contract](Interaction-Contract/Interaction-Contract-Description.md): pure rules for hotkey, recall, and clipboard dispatch decisions.
- [macOS Adaptation](MacOS-Adaptation/MacOS-Adaptation-Description.md): macOS-specific runtime behavior and capability boundaries.
- [Runtime Readiness](Runtime-Readiness/Runtime-Readiness-Description.md): setup checklist and provider probe behavior.
- [Translation Engine](Translation-Engine/Translation-Engine-Description.md): OpenAI-compatible streaming translation client.
- [Translation History](Translation-History/Translation-History-Description.md): local result history and retry metadata.
- [Translation Profiles](Translation-Profiles/Translation-Profiles-Description.md): saved provider/language setups.
- [UI Shell](UI-Shell/UI-Shell-Description.md): Svelte windows, settings, bubble, notifications, and interactions.

## Release Checklists

- [macOS Adaptation Checklist](macOS-Adaptation-Checklist.md)
- [Windows Trial Checklist](Windows-Trial-Checklist.md)

## Planning Notes

- [macOS M1 Proposal](macOS-M1-Proposal.md): 2026-09-13 repository assessment and proposed milestone for a lightweight, easy-to-use macOS translator. Its P0/P1 items are implemented; the document remains the assessment of record.
- [M1 execution plan](../plan/active/macos-m1-daily-driver/00-overview.md): the active execution contract for that milestone, including the verification matrix and the target-host rows still open.

Each module folder may include a `*-Plan.md` file. These files preserve planning history and may mention earlier implementation phases. Prefer the matching `*-Description.md` file and the root README for current product behavior.
