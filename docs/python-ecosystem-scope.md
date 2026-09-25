# Python Ecosystem Model Scope

## Languages

Primary support:

- Python (3.9+ syntax; type hints are in scope, not required)

## Frameworks

### Primary

- Django

### Secondary

- FastAPI
- Flask

Django is prioritized first because its ORM, migrations, admin, forms/serializers, middleware, and app/settings structure create the same kind of repository-level review signal NestJS does for the JavaScript/TypeScript specialization (see `docs/node-ecosystem-scope.md`): a large surface of framework-specific conventions worth deep, dedicated rules rather than generic Python advice.

## ORM and data-access coverage

Prioritize:

1. Django ORM (querysets, migrations, `select_related`/`prefetch_related`, signals)
2. SQLAlchemy (Core and ORM, sessions, migrations via Alembic)
3. Peewee
4. Tortoise ORM

Django ORM and SQLAlchemy get the first deep rules: they represent the two dominant styles in the ecosystem (framework-integrated active-record-like ORM vs. a standalone, more SQL-oriented toolkit usable outside any one framework).

## Database coverage

Relational:

- PostgreSQL
- MySQL / MariaDB
- SQLite
- SQL Server

Document/key-value:

- MongoDB
- Redis

As with the Node specialization, the model should learn database-specific review context (raw SQL safety, transaction semantics, indexing, pagination, connection lifecycle) rather than treating every database as interchangeable.

## Tooling and testing

Initial ecosystem signals should include:

- pip, Poetry, uv, pipenv, conda
- pytest, unittest, tox, nox
- mypy, Ruff, Black, Flake8
- Celery (background tasks, common alongside Django/FastAPI)
- common ASGI/WSGI servers: uvicorn, gunicorn

## Model responsibilities

Same dimensions as `docs/node-ecosystem-scope.md`: security, correctness, likely bugs, readability, maintainability, architecture, ORM usage, database access, error handling, API design, testing quality, performance, dependency usage, Python idioms, Django conventions, repository-specific standards. Static analyzers (`languages/python`) provide machine-verifiable evidence; the model reasons on top of it and does not replace it.

## Initial training priority

Same ordering as the Node specialization: security fixes, bug fixes, accepted PR review comments, reasoned refactors, regression fixes, ORM/database fixes, Django architecture corrections, meaningful test additions, maintainability/readability improvements, then style-only changes last.

## Deterministic analyzers implemented so far

`languages/python` (Rust, tree-sitter-python) implements five analyzers, chosen because each catches a distinct, well-known Python-specific failure mode rather than duplicating a JavaScript/TypeScript check verbatim:

- `python.mutable-default-argument` -- a list/dict/set literal as a default argument value, evaluated once at definition time and shared across calls.
- `python.bare-except` -- `except:` with no exception type, which also catches `SystemExit`/`KeyboardInterrupt`/`GeneratorExit`.
- `python.sql-injection` -- an f-string or `%`-formatted string passed to `.execute()`/`.executemany()`.
- `python.dangerous-eval` -- `eval()`/`exec()` calls.
- `python.debug-artifact` -- a leftover `breakpoint()` or `pdb.set_trace()` call.
