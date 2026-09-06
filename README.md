# Toasty Axum POC

CRUD API for the `Family` entity built with [Axum] and the [Toasty] ORM over
SQLite. `DELETE` is a soft delete: rows are preserved in the database with a
`deleted_at` timestamp and only hidden from the public API.

## Run

```bash
cargo run
```

The server listens on `http://0.0.0.0:3000` and creates `./families.db`
(ignored by git) on startup, pushing the schema defined by the models.

## Endpoints

| Method   | Path                | Description                      |
|----------|---------------------|----------------------------------|
| `GET`    | `/`                 | Health check                     |
| `GET`    | `/families`         | List active families             |
| `POST`   | `/families`         | Create a family                  |
| `GET`    | `/families/{id}`    | Get one family, 404 if deleted   |
| `PUT`    | `/families/{id}`    | Update a family                  |
| `DELETE` | `/families/{id}`    | Soft delete a family             |

## Examples

```bash
# Create
curl -X POST http://localhost:3000/families \
  -H 'content-type: application/json' \
  -d '{"name":"The Simpsons","summary":"A family from Springfield"}'

# List
curl http://localhost:3000/families

# Get one
curl http://localhost:3000/families/1

# Update (summary is optional; null clears it)
curl -X PUT http://localhost:3000/families/1 \
  -H 'content-type: application/json' \
  -d '{"name":"The Simpsons 2","summary":null}'

# Soft delete (204; the row stays in the database)
curl -X DELETE http://localhost:3000/families/1
```

## Soft delete behaviour

- `DELETE /families/{id}` stamps `deleted_at` with the current time; the row
  is never physically removed.
- Deleted families are excluded from `GET /families` and answer `404` on
  `GET /families/{id}` and `PUT /families/{id}`.
- Deleting a family that was already deleted is idempotent and returns
  `204`; deleting a nonexistent id returns `404`.

## Notes

- The schema is created with `Db::push_schema` for prototyping. Toasty also
  ships a migration system for when the model stabilizes and data must be
  preserved across schema changes.
- The single `Db` handle is shared through axum state behind `Arc<Mutex<_>>`
  because Toasty operations take the handle by mutable reference.

[Axum]: https://github.com/tokio-rs/axum
[Toasty]: https://github.com/tokio-rs/toasty