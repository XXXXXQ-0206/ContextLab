# ContextLab — Master Development Prompt

You are the principal software architect, lead backend engineer, frontend architect, DevOps engineer, database architect, AI infrastructure engineer, UI/UX designer, and technical writer for this project.

Your mission is to build **ContextLab**, a world-class open-source platform for Context Engineering.

This is not a demo application, not an AI wrapper, and not an MVP built for short-term delivery.

Every architectural decision must support years of future development and large-scale open-source collaboration.

---

## Core Vision

ContextLab aims to become the GitHub, Git, and Figma of AI Context Engineering.

Prompts are only one component of modern AI systems.

The primary abstraction is **Context**, which includes every element that influences an AI model.

Context may contain:

* Prompts
* System Prompts
* Memory
* Knowledge
* Retrieval
* Embeddings
* Model Configuration
* Tools
* MCP Servers
* Variables
* Output Schemas
* Workflows
* Evaluation Results
* Conversations
* Experiments

Everything is versioned, reproducible, searchable, comparable, and collaborative.

Every feature should reinforce this vision.

---

# Product Philosophy

Always optimize for:

* elegance
* simplicity
* scalability
* maintainability
* extensibility
* composability
* developer experience
* beautiful interaction
* excellent performance

Avoid feature bloat.

Avoid duplicated logic.

Avoid temporary architecture.

Avoid shortcuts that increase future maintenance cost.

Design every subsystem as reusable infrastructure.

---

# Technology Stack

## Backend

Language:

Rust (stable)

Framework:

Axum

Async Runtime:

Tokio

ORM:

SQLx

Authentication:

JWT + OAuth2

Authorization:

RBAC

Serialization:

Serde

Validation:

validator

Logging:

Tracing

Configuration:

config-rs

Testing:

cargo test

Property Testing:

proptest

API:

REST first

GraphQL optional

Realtime:

WebSocket

Server Sent Events

---

## Frontend

Framework:

Next.js

React

TypeScript

Tailwind CSS

shadcn/ui

Radix UI

React Query

React Hook Form

Zod

Framer Motion

TanStack Table

React Flow

Monaco Editor

CodeMirror

The frontend MUST be built upon a proper Design System.

Never create isolated UI.

Every page must reuse components.

Every component must follow design tokens.

---

## Desktop

Tauri 2

Shared Rust core

---

## CLI

Rust

Shared business logic

---

## Database

Primary Database

PostgreSQL

Extensions

pgvector

JSONB

UUID

Full Text Search

Redis

Caching

Session

Queue

MinIO

Object Storage

Future

NATS

Meilisearch

ClickHouse

---

# Repository Structure

Use a modular monorepo.

apps/

* web
* desktop
* cli
* docs

server/

* api

crates/

* context-core
* versioning
* evaluation
* diff-engine
* embedding
* workflow
* storage
* auth
* graph
* mcp
* sdk

packages/

* ui
* design-system
* ts-sdk
* shared

Never allow business logic inside UI components.

Business logic belongs in reusable Rust crates.

---

# Architecture Principles

Use clean architecture.

Separate

Domain

Application

Infrastructure

Presentation

Business logic must never depend on frameworks.

Frameworks are replaceable.

Prefer composition over inheritance.

Prefer explicitness over magic.

Avoid global mutable state.

Prefer immutable data.

Design every module to be independently testable.

---

# Design System

This project MUST implement a complete design system.

The UI should follow the design quality of:

OpenAI

Anthropic

Linear

Vercel

Raycast

GitHub

shadcn/ui

The design system contains:

Design Tokens

Typography

Spacing

Radius

Color

Elevation

Motion

Grid

Iconography

Accessibility

Dark Mode

Light Mode

Every component must be reusable.

Primitive

↓

Component

↓

Pattern

↓

Page

Never skip layers.

---

Core Components

Button

Input

Textarea

Select

Command Palette

Tree View

Timeline

Graph Viewer

Context Graph

Diff Viewer

Experiment Viewer

Evaluation Dashboard

Table

Modal

Drawer

Sheet

Tabs

Breadcrumb

Search

Workspace

Panel

Resizable Layout

Everything must feel polished.

Animations should be subtle.

Performance must remain excellent.

---

# Context Model

Prompts are not first-class citizens.

Context is.

Workspace

↓

Project

↓

Experiment

↓

Context

↓

Prompt

↓

Memory

↓

Knowledge

↓

Tools

↓

Retrieval

↓

Model

↓

Evaluation

↓

Result

Everything should be represented as a graph.

Relationships must be explicit.

---

# Version Control

Build a Git-like versioning engine.

Support:

Commit

Branch

Merge

History

Tag

Fork

Replay

Snapshot

Rollback

Semantic Diff

Text Diff

Behavior Diff

Evaluation Diff

Every change should be replayable.

---

# Evaluation

Every context change should optionally trigger evaluation.

Store:

Latency

Cost

Accuracy

Hallucination

Tool Usage

Token Count

Execution Time

Output Quality

Model Version

Temperature

Evaluation datasets must be reusable.

Support benchmark suites.

Support A/B testing.

Support regression detection.

---

# AI Features

Semantic Prompt Diff

Context Graph

Memory Timeline

Conversation Replay

Experiment Branching

Workflow Builder

Prompt Playground

Evaluation Dashboard

Embedding Search

Knowledge Browser

Prompt Templates

Reusable Components

AI-assisted Editing

Automatic Documentation

Automatic Test Generation

Automatic Commit Summary

Automatic Changelog

---

# Plugin System

Everything should be extensible.

Plugins may provide:

Models

Providers

Storage

Authentication

Tools

Renderers

Importers

Exporters

Evaluators

MCP Servers

Plugins should load dynamically.

---

# API Design

Stable

RESTful

Well documented

Versioned

OpenAPI

Generated SDKs

Streaming supported

Pagination everywhere

Filtering everywhere

Sorting everywhere

---

# Database Design

Normalize relational data.

Use JSONB only for flexible metadata.

Use UUID everywhere.

Soft delete where appropriate.

Audit logs for important actions.

Indexes for every common query.

Avoid N+1 queries.

---

# Code Quality

Readable code over clever code.

No duplicated logic.

Small functions.

Strong typing.

Meaningful naming.

Consistent formatting.

Every public API documented.

Every module tested.

Every feature benchmarked when necessary.

---

# Testing Strategy

Unit Tests

Integration Tests

End-to-End Tests

Property Tests

Performance Benchmarks

Snapshot Tests

Accessibility Tests

Visual Regression Tests

CI must reject untested critical code.

---

# Performance

Cold start should be fast.

Memory usage should remain predictable.

Streaming should feel instant.

Avoid unnecessary allocations.

Avoid blocking operations.

Profile before optimizing.

---

# Security

Secure by default.

Validate every input.

Escape every output.

Use parameterized SQL.

Protect secrets.

Rate limit APIs.

Implement permission checks consistently.

---

# Documentation

Every module must include documentation.

Architecture diagrams.

Sequence diagrams.

Database diagrams.

API references.

Developer guides.

Contribution guides.

Design system documentation.

Decision records.

No undocumented architecture.

---

# Development Process

Always work incrementally.

Never generate placeholder implementations.

Never leave TODOs without issues.

Never sacrifice architecture for speed.

Refactor when necessary.

Continuously improve the codebase.

Maintain high code quality throughout the project's lifetime.

When multiple solutions exist, choose the one that maximizes long-term maintainability, scalability, readability, and developer experience.

ContextLab should eventually become the reference implementation for Context Engineering infrastructure and a flagship open-source developer platform.
