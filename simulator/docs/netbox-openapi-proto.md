# NetBox OpenAPI to protobuf pipeline

The simulator contains a Rust generator at `src/bin/netbox_openapi_proto.rs` which translates NetBox's committed machine-readable OpenAPI schema into protobuf contracts.

## Upstream contract

NetBox commits its generated OpenAPI document at:

`netbox-community/netbox/contrib/openapi.json`

NetBox itself verifies that file against the schema produced by `manage.py spectacular --format openapi-json`. The generator consumes that committed JSON rather than scraping Swagger UI.

## Local generation

From the repository root:

```sh
curl --fail --location \
  https://raw.githubusercontent.com/netbox-community/netbox/main/contrib/openapi.json \
  -o /tmp/netbox-openapi.json

cargo run --locked --manifest-path simulator/Cargo.toml --bin netbox_openapi_proto -- \
  --input /tmp/netbox-openapi.json \
  --output-dir simulator/proto/netbox \
  --package netbox.v1
```

The generator can also read JSON from standard input by passing `--input -`.

To verify an already-generated tree without modifying it:

```sh
cargo run --locked --manifest-path simulator/Cargo.toml --bin netbox_openapi_proto -- \
  --input /tmp/netbox-openapi.json \
  --output-dir simulator/proto/netbox \
  --package netbox.v1 \
  --check
```

## Mapping rules

The generator intentionally treats OpenAPI as an API contract, not as a database schema.

| OpenAPI construct | Protobuf output |
|---|---|
| `components.schemas` object | `message` |
| schema-level string/integer enum | `enum` |
| inline property enum | nested `enum` |
| `$ref` | reference to generated component type |
| array | `repeated` field |
| dictionary/`additionalProperties` | protobuf map or `Struct` |
| RFC 3339 `date-time` | `google.protobuf.Timestamp` |
| binary/byte string | `bytes` |
| unsupported `oneOf`/`anyOf` or anonymous complex shape | `google.protobuf.Value` with an explicit generated comment |
| OpenAPI operation | gRPC `rpc` |
| path/query parameter | request-message field |
| request body | request-message `body` field |
| component response | direct protobuf response type |
| array/primitive/anonymous response | generated response wrapper |

Services are grouped by the first path segment after `/api/`. For example:

```text
/api/dcim/...            -> DcimService
/api/ipam/...            -> IpamService
/api/virtualization/...  -> VirtualizationService
/api/extras/...          -> ExtrasService
```

The generated RPC comment retains the original HTTP method and path. The generator does not currently emit grpc-gateway annotations.

## Stable field numbering

Sequential field numbering is simple but causes wire-incompatible renumbering when a newly-added OpenAPI property sorts before an existing property.

This generator instead derives a protobuf field number from a stable FNV-1a hash of:

```text
<message-name>.<OpenAPI-field-name>
```

It excludes protobuf's reserved field-number interval 19000-19999 and resolves the extremely unlikely collision deterministically within each message.

This means adding unrelated fields normally leaves every existing protobuf field number unchanged.

## Generated-file ownership

The output directory contains a manifest named:

`simulator/proto/netbox/.netbox-openapi-proto-manifest`

Only files listed by that manifest are eligible for stale-file deletion by the generator. Unrelated files in the directory are left alone.

## GitHub Actions

`.github/workflows/netbox-openapi-proto.yml` does two jobs:

1. On relevant pull requests it runs the Rust generator's unit tests.
2. Weekly and on manual dispatch it downloads NetBox's current `contrib/openapi.json`, regenerates `simulator/proto/netbox`, and, when the generated contracts changed, creates or refreshes the branch `automation/netbox-openapi-proto` and opens a pull request.

The manual workflow accepts a NetBox branch, tag, or commit via `netbox_ref`, so a release can be pinned deliberately instead of following `main`.

## Deliberate boundaries

This pipeline does **not** infer the simulator database schema from OpenAPI. NetBox models/migrations remain the authority for persistence semantics.

It also does not pretend OpenAPI unions map cleanly to protobuf. Where NetBox exposes a shape that cannot be represented faithfully without schema-specific policy, the generator uses `google.protobuf.Value` and marks the generated field as such. That is preferable to silently inventing an incorrect protobuf type.
