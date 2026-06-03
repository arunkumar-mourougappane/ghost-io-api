# Ghost.io API Reference

> Source: [Ghost Developer Docs](https://ghost.org/docs/) — last fetched 2026-06-03

Ghost exposes two distinct REST APIs for working with your publication:

| API | Base URL | Access | Auth method |
|-----|----------|--------|-------------|
| **Content API** | `https://{admin_domain}/ghost/api/content/` | Read-only (public) | API key query param |
| **Admin API** | `https://{admin_domain}/ghost/api/admin/` | Read + Write | JWT (server-side only) |

---

## Table of Contents

1. [Content API](#content-api)
   - [Authentication](#content-api-authentication)
   - [Endpoints](#content-api-endpoints)
   - [Query Parameters](#query-parameters)
   - [Response Format](#response-format)
2. [Admin API](#admin-api)
   - [Authentication](#admin-api-authentication)
   - [Token Generation](#token-generation)
   - [Endpoints](#admin-api-endpoints)
3. [Webhooks](#webhooks)
4. [Error Handling](#error-handling)
5. [Versioning](#versioning)

---

## Content API

Ghost's RESTful Content API delivers published content and is safe for use in browsers or public applications. It is fully cacheable with no rate limits.

### Content API Authentication

Content API keys are passed as a query parameter:

```
GET https://{admin_domain}/ghost/api/content/posts/?key={your_content_api_key}
```

Keys are obtained from **Ghost Admin → Settings → Integrations → Add Custom Integration**.

> ⚠️ Keys only ever give access to public data. For private-mode sites, be mindful of where keys are shared.

**Working example (real endpoint):**

```bash
curl -H "Accept-Version: v6.0" \
  "https://demo.ghost.io/ghost/api/content/posts/?key=22444f78447824223cefc48062"
```

### Content API Endpoints

All Content API endpoints use `GET` only (read-only):

| Method | Path | Description |
|--------|------|-------------|
| GET | `/posts/` | Browse posts |
| GET | `/posts/{id}/` | Read a post by ID |
| GET | `/posts/slug/{slug}/` | Read a post by slug |
| GET | `/pages/` | Browse pages |
| GET | `/pages/{id}/` | Read a page by ID |
| GET | `/pages/slug/{slug}/` | Read a page by slug |
| GET | `/tags/` | Browse tags |
| GET | `/tags/{id}/` | Read a tag by ID |
| GET | `/tags/slug/{slug}/` | Read a tag by slug |
| GET | `/authors/` | Browse authors |
| GET | `/authors/{id}/` | Read an author by ID |
| GET | `/authors/slug/{slug}/` | Read an author by slug |
| GET | `/tiers/` | Browse tiers (membership levels) |
| GET | `/settings/` | Read site-wide settings |

---

## Query Parameters

All endpoints accept `include` and `fields`. Browse endpoints additionally accept `filter`, `limit`, `page`, and `order`.

> All query parameter values **must be URL-encoded** when used directly. Client libraries handle this automatically.

### `include`

Fetch related data alongside the primary resource:

| Resource | Available includes |
|----------|--------------------|
| Posts & Pages | `authors`, `tags` |
| Authors | `count.posts` |
| Tags | `count.posts` |
| Tiers | `monthly_price`, `yearly_price`, `benefits` |

```
# Multiple includes separated by comma
&include=authors,tags
```

Effect on Posts/Pages:
- `&include=authors` → adds `"authors": [{...}]` and `"primary_author": {...}`
- `&include=tags` → adds `"tags": [{...}]` and `"primary_tag": {...}`

### `fields`

Limit fields returned (useful for performance optimization):

```
&fields=title,url
```

Response example:

```json
{
  "posts": [
    {
      "id": "5b7ada404f87d200b5b1f9c8",
      "title": "Welcome to Ghost",
      "url": "https://demo.ghost.io/welcome/"
    }
  ]
}
```

### `formats`

*(Posts and Pages only)*

By default, only `html` is returned. Available formats: `html`, `plaintext`.

```
&formats=html,plaintext
```

### `filter`

*(Browse requests only)*

Apply fine-grained NQL (Notion Query Language) filters:

```
&filter=featured:true              # Featured posts only
&filter=tag:getting-started        # Posts with specific tag slug
&filter=visibility:public          # Publicly visible tiers only
```

Supports operators: `:`, `-`, `<`, `>`, `[in]`, `+` (AND), `,` (OR), `()` (grouping).

### `limit`

*(Browse requests only)*

Default: `15`. Maximum: `100`. Use `all` to retrieve everything (use with caution).

```
&limit=5      # Return 5 records
&limit=100    # Return 100 records (max)
```

### `page`

*(Browse requests only)*

Navigate through paginated results:

```
&page=2    # Return the second page of results
```

### `order`

*(Browse requests only)*

Control sort order (follows SQL `ORDER BY` syntax):

| Resource | Default Order |
|----------|--------------|
| Posts | `published_at DESC` (newest first) |
| Pages | `title ASC` (alphabetical) |
| Tags | `name ASC` (alphabetical) |
| Authors | `name ASC` (alphabetical) |
| Tiers | `monthly_price ASC` (lowest first) |

```
# URL-encoded space required
&order=published_at%20asc
```

---

## Response Format

All API responses return JSON with a consistent structure:

```json
{
  "resource_type": [
    { ... }
  ],
  "meta": {
    "pagination": {
      "page": 1,
      "limit": 15,
      "pages": 5,
      "total": 73,
      "next": 2,
      "prev": null
    }
  }
}
```

- `resource_type` — always matches the URL resource name (e.g. `"posts"`, `"tags"`). All resources are returned wrapped in an array, **except** `/site/` and `/settings/`.
- `meta.pagination` — present on Browse requests only.

---

## Admin API

The Admin API provides full read/write access. It powers Ghost Admin itself, so anything Ghost Admin can do, the API can do too.

> ⚠️ **Never expose the Admin API key in browser-side code or public repositories.** It must only be used in secure server-side environments.

### Admin API Authentication

There are three authentication methods:

| Method | Use case |
|--------|----------|
| **Integration Token (JWT)** | Integrations & automations (recommended) |
| **Staff Access Token** | Acting as a specific user (server-side) |
| **User Authentication (Session)** | Full client apps where users log in interactively |

#### Integration Token (JWT) — Recommended

Admin API keys follow the format `{key_id}:{secret}` and are used to sign a short-lived JWT:

```bash
curl -H "Authorization: Ghost $TOKEN" \
     -H "Accept-Version: v6.0" \
     https://{admin_domain}/ghost/api/admin/{resource}/
```

### Token Generation

The JWT must be generated **server-side**. Steps:

1. Split the Admin API key on `:` → `id` and `secret`
2. Decode the hex `secret` into a binary byte array
3. Sign a JWT with `HS256` using the decoded secret

**JWT Header:**
```json
{
  "alg": "HS256",
  "kid": "{id}",
  "typ": "JWT"
}
```

**JWT Payload:**
```json
{
  "iat": 1234567890,
  "exp": 1234568190,
  "aud": "/admin/"
}
```

> Timestamps are Unix epoch in **seconds** (not milliseconds). `exp` must be at most 5 minutes after `iat`.

#### Examples

**Bash (cURL):**
```bash
#!/usr/bin/env bash
KEY="YOUR_ADMIN_API_KEY"
IFS=':' read ID SECRET <<< "$KEY"

NOW=$(date +'%s')
FIVE_MINS=$(($NOW + 300))
HEADER="{\"alg\": \"HS256\",\"typ\": \"JWT\", \"kid\": \"$ID\"}"
PAYLOAD="{\"iat\":$NOW,\"exp\":$FIVE_MINS,\"aud\": \"/admin/\"}"

base64_url_encode() {
    printf '%s' "${1:-$(</dev/stdin)}" | base64 | tr -d '=' | tr '+' '-' | tr '/' '_'
}

header_base64=$(base64_url_encode "$HEADER")
payload_base64=$(base64_url_encode "$PAYLOAD")
header_payload="${header_base64}.${payload_base64}"
signature=$(printf '%s' "${header_payload}" | openssl dgst -binary -sha256 -mac HMAC -macopt hexkey:$SECRET | base64_url_encode)
TOKEN="${header_payload}.${signature}"

curl -H "Authorization: Ghost $TOKEN" \
     -H "Content-Type: application/json" \
     -H "Accept-Version: v6.0" \
     -d '{"posts":[{"title":"Hello world"}]}' \
     "http://localhost:2368/ghost/api/admin/posts/"
```

**Python:**
```python
import requests
import jwt
from datetime import datetime as date

key = 'YOUR_ADMIN_API_KEY'
id, secret = key.split(':')

iat = int(date.now().timestamp())
header = {'alg': 'HS256', 'typ': 'JWT', 'kid': id}
payload = {
    'iat': iat,
    'exp': iat + 5 * 60,
    'aud': '/admin/'
}

token = jwt.encode(payload, bytes.fromhex(secret), algorithm='HS256', headers=header)

url = 'http://localhost:2368/ghost/api/admin/posts/'
headers = {'Authorization': 'Ghost {}'.format(token)}
body = {'posts': [{'title': 'Hello World'}]}
r = requests.post(url, json=body, headers=headers)
```

**JavaScript (Node.js):**
```js
const jwt = require('jsonwebtoken');
const axios = require('axios');

const key = 'YOUR_ADMIN_API_KEY';
const [id, secret] = key.split(':');

const token = jwt.sign({}, Buffer.from(secret, 'hex'), {
    keyid: id,
    algorithm: 'HS256',
    expiresIn: '5m',
    audience: '/admin/'
});

const url = 'http://localhost:2368/ghost/api/admin/posts/';
const headers = { Authorization: `Ghost ${token}` };
const payload = { posts: [{ title: 'Hello World' }] };
axios.post(url, payload, { headers });
```

**Using the official JS Admin API Client (simplest):**
```js
const GhostAdminAPI = require('@tryghost/admin-api');

const api = new GhostAdminAPI({
    url: 'http://localhost:2368/',
    key: 'YOUR_ADMIN_API_KEY',
    version: 'v6'
});

api.posts.add({ title: 'Hello world' })
    .then(response => console.log(response))
    .catch(error => console.error(error));
```

#### Staff Access Token

Found in individual user profile pages. Used exactly like Integration Token JWT — server-side only.

#### User Authentication (Session-based)

For interactive clients where users log in manually.

**Create a session:**
```bash
# POST /admin/session/
curl -c ghost-cookie.txt \
  -d username=me@site.com \
  -d password=secretpassword \
  -H "Origin: https://myappsite.com" \
  -H "Accept-Version: v6.0" \
  https://demo.ghost.io/ghost/api/admin/session/
```

On success → HTTP `201` with a `set-cookie` header containing the session token.

If 2FA is required → HTTP `403` with message `"User must verify session to login"`. Then submit the auth code:

```bash
# PUT /admin/session/verify/
{ "token": "{auth_code}" }
```

**CSRF Protection:** Session requests must include an `Origin` or `Referer` header matching the session creation request.

### Admin API Endpoints

Available to integrations (all stable):

| Resource | Available Methods |
|----------|------------------|
| `/posts/` | Browse, Read, Edit, Add, Copy, Delete |
| `/pages/` | Browse, Read, Edit, Add, Copy, Delete |
| `/tags/` | Browse, Read, Edit, Add, Delete |
| `/tiers/` | Browse, Read, Edit, Add |
| `/newsletters/` | Browse, Read, Edit, Add |
| `/offers/` | Browse, Read, Edit, Add |
| `/members/` | Browse, Read, Edit, Add |
| `/labels/` | Browse, Read, Edit, Add, Delete |
| `/users/` | Browse, Read |
| `/images/` | Upload |
| `/themes/` | Upload, Activate |
| `/site/` | Read |
| `/webhooks/` | Edit, Add, Delete |

**HTTP method mapping:**
- Browse → `GET /resource/`
- Read → `GET /resource/{id}/`
- Add → `POST /resource/`
- Edit → `PUT /resource/{id}/`
- Delete → `DELETE /resource/{id}/`

**All POST/PUT requests require:**
```
Content-Type: application/json
```

And payloads must follow the standard JSON structure:
```json
{
  "resource_type": [{ ... }]
}
```

---

## Webhooks

Ghost sends `POST` requests to user-configured URLs when events occur.

**Setup:** Ghost Admin → Settings → Advanced → Integrations → Add custom integration

If the server responds with a `2xx` HTTP status, the delivery is considered successful. Response body is discarded.

### Available Webhook Events

| Event | Description |
|-------|-------------|
| `site.changed` | Any content or settings change |
| `post.added` | Post created |
| `post.deleted` | Post deleted |
| `post.edited` | Post edited |
| `post.published` | Post published |
| `post.published.edited` | Published post edited |
| `post.unpublished` | Post unpublished |
| `post.scheduled` | Post scheduled |
| `post.unscheduled` | Post unscheduled |
| `post.rescheduled` | Post rescheduled |
| `page.added` | Page created |
| `page.deleted` | Page deleted |
| `page.edited` | Page edited |
| `page.published` | Page published |
| `page.published.edited` | Published page edited |
| `page.unpublished` | Page unpublished |
| `page.scheduled` | Page scheduled |
| `page.unscheduled` | Page unscheduled |
| `page.rescheduled` | Page rescheduled |
| `tag.added` | Tag created |
| `tag.edited` | Tag edited |
| `tag.deleted` | Tag deleted |
| `post.tag.attached` | Tag attached to post |
| `post.tag.detached` | Tag detached from post |
| `page.tag.attached` | Tag attached to page |
| `page.tag.detached` | Tag detached from page |
| `member.added` | Member created |
| `member.edited` | Member updated |
| `member.deleted` | Member deleted |

### Stripe Webhooks (Local Development)

To test Stripe payments locally:

```bash
# Terminal 1: Forward Stripe events to local Ghost
stripe listen --forward-to http://localhost:2368/members/webhooks/stripe/

# Terminal 2: Start Ghost with the webhook secret
WEBHOOK_SECRET=whsec_1234567890abcdefg ghost start
```

---

## Error Handling

Errors follow a consistent structure:

```json
{
  "errors": [
    {
      "message": "Resource not found",
      "context": null,
      "type": "NotFoundError",
      "details": null,
      "property": null,
      "help": null,
      "code": null,
      "id": "abc123"
    }
  ]
}
```

Common HTTP status codes used:
- `200 OK` — Successful GET
- `201 Created` — Successful POST
- `204 No Content` — Successful DELETE
- `400 Bad Request` — Validation error
- `401 Unauthorized` — Missing or invalid auth
- `403 Forbidden` — Insufficient permissions
- `404 Not Found` — Resource doesn't exist
- `422 Unprocessable Entity` — Data failed validation
- `500 Internal Server Error` — Server-side error

---

## Versioning

Use the `Accept-Version` header to pin to a specific API version:

```
Accept-Version: v6.0
```

- Versioning follows Ghost's release versioning (currently v6.x)
- All documented endpoints are considered **stable**
- Breaking changes are catalogued at https://docs.ghost.org/changes.md

---

## Official Client Libraries

| Language | Package | Use with |
|----------|---------|----------|
| JavaScript (browser/Node) | `@tryghost/content-api` | Content API |
| JavaScript (Node only) | `@tryghost/admin-api` | Admin API |

**Install:**
```bash
npm install @tryghost/content-api
npm install @tryghost/admin-api
```

**Content API client:**
```js
const GhostContentAPI = require('@tryghost/content-api');

const api = new GhostContentAPI({
    url: 'https://demo.ghost.io',
    key: '22444f78447824223cefc48062',
    version: 'v6'
});

// Browse posts
api.posts.browse({ limit: 5, include: 'tags,authors' })
    .then(posts => console.log(posts))
    .catch(err => console.error(err));
```

---

## Quick Reference

```
# Content API
GET https://{host}/ghost/api/content/posts/?key={key}&limit=10&include=tags,authors

# Admin API
POST https://{host}/ghost/api/admin/posts/
Headers:
  Authorization: Ghost {jwt}
  Content-Type: application/json
  Accept-Version: v6.0
Body: { "posts": [{ "title": "...", "status": "published" }] }
```

---

*Official docs: https://ghost.org/docs/ | API index: https://docs.ghost.org/llms.txt*
