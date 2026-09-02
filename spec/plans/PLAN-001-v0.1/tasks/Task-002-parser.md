---
id: Task-002
title: "Versioned lexer and direct parser"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-001
    type: part_of
---
# Task-002: Versioned lexer and direct parser

## Scope

Implement the independently authored ASCII dialect, precedence rules, stable
UTF-8 byte spans, and direct construction of exact-profile tl-syntax graphs.

## Completion Evidence

Tests cover every token and node kind, precedence, associativity, invalid
lexemes, exact profiles, topological node order, source spans, and fail-closed
recovery without a second public AST.
