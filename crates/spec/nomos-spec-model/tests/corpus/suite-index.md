---
id: DOC-DOMAIN-000
kind: artifact
type: artifact
title: "Nomos Domain-Owned Specification Suite"
status: accepted
version: 1
authority: canonical-explanatory-narrative
publication: authored-markdown
source_role: domain-owned-narrative
source_docx: "Nomos_Domain_Owned_Suite_Index.docx"
relations: []
---
# Nomos Domain-Owned Specification Suite

Nomos Focused Product Design & User Story Specification

Navigation, ownership rules, reading paths, and source-preservation notes for the domain-owned v14.19 suite.

Domain-owned edition v14.19 • 4 August 2026

## Document purpose

Navigation, ownership rules, reading paths, and source-preservation notes for the domain-owned v14.19 suite.

This volume owns the domain material collected here: concepts, canonical models, requirements, user stories, workflows, quality constraints, roadmap placement, and reference projections. Original Nomos identifiers and normative wording are preserved.

## Contents

Use Word’s Navigation pane to browse headings. Numbering inherited from the governed source is retained for traceability; gaps indicate material owned by another volume.

## Governing organization

Each domain volume owns its architecture, models, requirements, user stories, workflows, quality obligations, roadmap references, and relevant examples. Cross-domain IDs remain unchanged. The reference volume contains only material that is genuinely lookup-oriented or too extended to interrupt a domain specification.

## Volume map

### 01. Nomos Product Definition and Principles

Product thesis, outcomes, scope, actors, operating principles, and defining product loops.

### 02. Core Architecture, Identity, and Configuration

Layered architecture, canonical identity, snapshots, build variants, configuration, evidence authority, and shared operating models.

### 03. Packages, Providers, Rules, and Applicability

Language packages, rule packages, capability contracts, provider selection, applicability, package lifecycle, and authoring.

### 04. Checks, Gates, Corrections, and Governance

Findings, suppressions, gates, workflows, corrections, convergence, rule interaction, calibration, and approval governance.

### 05. Atlas, Architecture, Features, Tests, and Runtime Evidence

Software Atlas, topology, metrics, feature lineage, test intelligence, debugging, captures, runtime rules, and instrumentation.

### 06. Agents, Models, Knowledge, Evidence, and Telemetry

Agent safety, KnowledgeWorkbench integration, model routing, executor packages, engineering evidence, telemetry, trust, and capsules.

### 07. Clients, Interfaces, Deployment, and Administration

CLI/API/MCP/LSP surfaces, desktop/web/mobile/IDE clients, deployment, integrations, security, operations, stewardship, budgets, and adoption.

### 08. Roadmap, Quality, Decisions, and Traceability

Non-functional requirements, delivery roadmap, decision register, unresolved choices, and suite-wide traceability.

### 09. Reference: Glossary, Schemas, and Scenarios

Glossary, illustrative manifests, generated projections, reference profiles, and extended worked scenarios.

## First usable product boundary

The first coherent Nomos product is a **headless, deterministic, incremental check-and-gate platform**. It includes independently versioned packages, capability-derived applicability, truthful findings and coverage, baselines and suppressions, safe correction preview, reproducible snapshots, CLI/API/MCP access, and one rich local workflow.

The following are later extensions of that product spine rather than prerequisites for it: full Atlas visualization, inferred architecture, feature and test intelligence, runtime evidence, model routing, autonomous agent workflows, hosted collaboration, mobile approval, and advanced rule-ecology or topology research. These capabilities may deepen analysis and workflow support, but they shall not make deterministic local conformance depend on models, hosted services, broad runtime instrumentation, or the complete long-term platform.

## Recommended reading paths

- New contributor: 01 → 02 → the relevant domain volume → 08.

- Rule or language-package implementer: 03 → 04 → 08 → 09.

- Atlas, architecture, testing, or runtime implementer: 05 → 02 → 03 → 08.

- Agent/model platform implementer: 06 → 04 → 07 → 08.

- Client or operations implementer: 07 → 02 → 04 → 08.

- Product or release reviewer: 01 → 08, then inspect affected domain volumes.

## Preservation and normalization rules

- Original requirement, story, workflow, decision, and appendix identifiers are preserved.

- Normative wording is not silently rewritten or reconciled.

- Former appendices C, E, F, and H are promoted into their owning domain volumes.

- Traceability is retained in Volume 08; glossary, manifests, profiles, and long worked scenarios remain in Volume 09.

- Numbering gaps are intentional and indicate cross-volume ownership rather than missing content.
