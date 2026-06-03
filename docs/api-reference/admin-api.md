# Ghost Admin API — Deep Reference

> Source: [Ghost Developer Docs — Admin API](https://ghost.org/docs/admin-api/) — last fetched 2026-06-03

The Admin API provides full **read/write** access to your Ghost publication. It powers Ghost Admin itself — everything Ghost Admin can do, the API can do too.

> ⚠️ **Server-side only.** Admin API keys must never be exposed in browser code or public repositories.

---

## Table of Contents

1. [Base URL & Headers](#base-url--headers)
2. [Authentication](#authentication)
   - [Integration Token (JWT)](#integration-token-jwt--recommended)
   - [Token Generation Examples](#token-generation-examples)
   - [Staff Access Token](#staff-access-token)
   - [User Session Authentication](#user-session-authentication)
3. [Posts](#posts)
   - [Post Object Schema](#post-object-schema)
   - [Browse & Read](#browse--read-posts)
   - [Create a Post](#create-a-post)
   - [Update a Post](#update-a-post)
   - [Publish a Post](#publish-a-post)
   - [Schedule a Post](#schedule-a-post)
   - [Send via Email](#send-a-post-via-email)
   - [Delete a Post](#delete-a-post)
4. [Pages](#pages)
5. [Tags](#tags)
6. [Members](#members)
   - [Member Object Schema](#member-object-schema)
   - [Subscription Object](#subscription-object)
7. [Newsletters](#newsletters)
8. [Tiers](#tiers)
9. [Offers](#offers)
10. [Images](#images)
11. [Users](#users)
12. [Webhooks](#webhooks)
13. [Site](#site)
14. [Themes](#themes)
15. [Error Handling & Collision Detection](#error-handling--collision-detection)

---

## Base URL & Headers

```
https://{admin_domain}/ghost/api/admin/
```

Every request requires:

| Header | Value | Notes |
|--------|-------|-------|
| `Authorization` | `Ghost {jwt_token}` | Required for token auth |
| `Accept-Version` | `v6.0` | Always pin to a version |
| `Content-Type` | `application/json` | Required on POST / PUT |

---

## Authentication

Three methods are supported:

| Method | Use case | Security |
|--------|----------|----------|
| Integration Token (JWT) | Integrations & automations | Server-side only |
| Staff Access Token | Acting as a specific user | Server-side only |
| User Session | Interactive user-facing clients | Browser-safe (with CSRF protection) |

### Integration Token (JWT) — Recommended

Admin API keys are in the format `{key_id}:{secret}` and are used to sign short-lived JWTs (5-minute expiry).

**Get your key:** Ghost Admin → Settings → Integrations → Add Custom Integration.

**JWT Header:**
```json
{
  "alg": "HS256",
  "kid": "{key_id}",
  "typ": "JWT"
}
```

**JWT Payload:**
```json
{
  "iat": 1686000000,
  "exp": 1686000300,
  "aud": "/admin/"
}
```

> Timestamps are Unix epoch **seconds** (not milliseconds). `exp` must be ≤ 5 minutes after `iat`.

### Token Generation Examples

**Bash:**
```bash
KEY="YOUR_ADMIN_API_KEY"
IFS=':' read ID SECRET <<< "$KEY"

NOW=$(date +'%s')
FIVE_MINS=$(($NOW + 300))
HEADER="{\"alg\": \"HS256\",\"typ\": \"JWT\", \"kid\": \"$ID\"}"
PAYLOAD="{\"iat\":$NOW,\"exp\":$FIVE_MINS,\"aud\": \"/admin/\"}"

base64_url_encode() {
    printf '%s' "${1:-$(</dev/stdin)}" | base64 | tr -d '=' | tr '+' '-' | tr '/' '_'
}

header_b64=$(base64_url_encode "$HEADER")
payload_b64=$(base64_url_encode "$PAYLOAD")
header_payload="${header_b64}.${payload_b64}"
signature=$(printf '%s' "${header_payload}" | openssl dgst -binary -sha256 -mac HMAC -macopt hexkey:$SECRET | base64_url_encode)
TOKEN="${header_payload}.${signature}"

curl -H "Authorization: Ghost $TOKEN" \
     -H "Content-Type: application/json" \
     -H "Accept-Version: v6.0" \
     "https://{admin_domain}/ghost/api/admin/posts/"
```

**JavaScript (Node.js):**
```js
const jwt = require('jsonwebtoken');

const key = 'YOUR_ADMIN_API_KEY';
const [id, secret] = key.split(':');

const token = jwt.sign({}, Buffer.from(secret, 'hex'), {
    keyid: id,
    algorithm: 'HS256',
    expiresIn: '5m',
    audience: '/admin/'
});
// Use: Authorization: Ghost ${token}
```

**Python:**
```python
import jwt
from datetime import datetime as date

key = 'YOUR_ADMIN_API_KEY'
id, secret = key.split(':')

iat = int(date.now().timestamp())
header = {'alg': 'HS256', 'typ': 'JWT', 'kid': id}
payload = {'iat': iat, 'exp': iat + 5 * 60, 'aud': '/admin/'}

token = jwt.encode(payload, bytes.fromhex(secret), algorithm='HS256', headers=header)
# Use: Authorization: Ghost {token}
```

**Official JS Client (handles JWT automatically):**
```js
const GhostAdminAPI = require('@tryghost/admin-api');

const api = new GhostAdminAPI({
    url: 'https://{admin_domain}',
    key: 'YOUR_ADMIN_API_KEY',
    version: 'v6'
});
```

### Staff Access Token

Each user has a personal staff access token on their profile page. Use it exactly like an integration token JWT — server-side only.

### User Session Authentication

For interactive clients where users authenticate with email + password.

**Create a session:**
```bash
POST /admin/session/
Content-Type: application/x-www-form-urlencoded
Origin: https://yourapp.com

username=me@example.com&password=secret
```

**Responses:**
- `201 Created` → `set-cookie: ghost-admin-api-session=...` — use this cookie on all subsequent requests
- `403 Needs2FAError` → "User must verify session to login" — 2FA required

**Complete 2FA verification:**
```json
PUT /admin/session/verify/
{ "token": "{6-digit-auth-code}" }
```

**Resend auth code:**
```json
POST /admin/session/verify/
{}
```

**CSRF Protection:** All session requests must include an `Origin` header matching the original session creation request. Browsers handle this automatically; server/native apps must send it manually.

---

## Posts

Posts are the primary content resource. Each post has a **Lexical** field (Ghost's JSON-based rich text format) at its core.

```
GET    /admin/posts/
GET    /admin/posts/{id}/
GET    /admin/posts/slug/{slug}/
POST   /admin/posts/
PUT    /admin/posts/{id}/
DELETE /admin/posts/{id}/
```

### Post Object Schema

```json
{
  "posts": [
    {
      "id": "5ddc9141c35e7700383b2937",
      "uuid": "a5aa9bd8-ea31-415c-b452-3040dae1e730",
      "slug": "welcome-short",
      "title": "Welcome",
      "lexical": "{...}",
      "html": "<p>Hello, beautiful world! 👋</p>",
      "comment_id": "5ddc9141c35e7700383b2937",
      "feature_image": "https://example.com/image.png",
      "feature_image_alt": null,
      "feature_image_caption": null,
      "featured": false,
      "status": "published",
      "visibility": "public",
      "created_at": "2019-11-26T02:43:13.000Z",
      "updated_at": "2019-11-26T02:44:17.000Z",
      "published_at": "2019-11-26T02:44:17.000Z",
      "custom_excerpt": null,
      "codeinjection_head": null,
      "codeinjection_foot": null,
      "custom_template": null,
      "canonical_url": null,
      "email_only": false,
      "tags": [ { "id": "...", "name": "Getting Started", "slug": "getting-started", ... } ],
      "authors": [ { "id": "...", "name": "Ghost", "email": "info@ghost.org", ... } ],
      "primary_author": { ... },
      "primary_tag": { ... },
      "url": "https://demo.ghost.io/welcome-short/",
      "excerpt": "👋 Welcome, it's great to have you here.",
      "og_image": null,
      "og_title": null,
      "og_description": null,
      "twitter_image": null,
      "twitter_title": null,
      "twitter_description": null,
      "meta_title": null,
      "meta_description": null,
      "newsletter": { "id": "...", "name": "Weekly newsletter", "slug": "default-newsletter", ... },
      "email": {
        "id": "...",
        "status": "submitted",
        "recipient_filter": "status:-free",
        "email_count": 256,
        "delivered_count": 256,
        "opened_count": 59,
        "failed_count": 0,
        "track_opens": true,
        "submitted_at": "2022-05-26T08:33:10.000Z"
      }
    }
  ]
}
```

**Key post fields:**

| Field | Type | Description |
|-------|------|-------------|
| `title` | string | Post title (**required on create**) |
| `status` | string | `draft`, `published`, `scheduled`, `sent` |
| `visibility` | string | `public`, `members`, `paid`, `tiers` |
| `lexical` | string (JSON) | Rich text content in Lexical JSON format |
| `html` | string | Rendered HTML (read with `?formats=html`) |
| `feature_image` | string | URL to the feature image |
| `featured` | boolean | Whether post is featured |
| `published_at` | datetime | Publication datetime (ISO 8601) |
| `custom_excerpt` | string | Custom excerpt/summary |
| `email_only` | boolean | Send as email newsletter only (no web post) |
| `tags` | array | Tag objects or name strings |
| `authors` | array | Author objects or email strings |
| `newsletter` | object | Newsletter to send post through |
| `canonical_url` | string | Override canonical URL |
| `codeinjection_head` | string | Custom HTML injected in `<head>` |
| `codeinjection_foot` | string | Custom HTML injected before `</body>` |

> By default, the Admin API returns and expects **Lexical** format. To get HTML in responses use `?formats=html,lexical`.

### Browse & Read Posts

```bash
# Browse (paginated, 15 per page by default)
GET /admin/posts/?limit=10&page=1&include=tags,authors&formats=html,lexical

# Read by ID
GET /admin/posts/5ddc9141c35e7700383b2937/

# Read by slug
GET /admin/posts/slug/welcome-short/
```

Supports: `include`, `fields`, `formats`, `filter`, `limit`, `page`, `order`.

### Create a Post

```
POST /admin/posts/
```

**Required:** `title`

**Minimal draft:**
```json
{
  "posts": [
    {
      "title": "My new post"
    }
  ]
}
```

**Published post with Lexical content:**
```json
{
  "posts": [
    {
      "title": "My test post",
      "lexical": "{\"root\":{\"children\":[{\"children\":[{\"detail\":0,\"format\":0,\"mode\":\"normal\",\"style\":\"\",\"text\":\"Hello world!\",\"type\":\"extended-text\",\"version\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"paragraph\",\"version\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}}",
      "status": "published"
    }
  ]
}
```

**Create from HTML source** (Ghost converts HTML → Lexical):
```json
// POST /admin/posts/?source=html
{
  "posts": [
    {
      "title": "My HTML post",
      "html": "<p>My post content.</p>",
      "status": "published"
    }
  ]
}
```

> The HTML→Lexical conversion is **lossy**. For lossless HTML, wrap in a card:
> ```html
> <!--kg-card-begin: html-->
> <p>Any HTML here</p>
> <!--kg-card-end: html-->
> ```

**With tags and authors:**
```json
// Short form: tags by name, authors by email
{
  "posts": [{
    "title": "My post",
    "tags": ["Getting Started", "News"],
    "authors": ["editor@example.com"]
  }]
}

// Long form: by ID or with extra metadata
{
  "posts": [{
    "title": "My post",
    "tags": [
      { "name": "my tag", "description": "A useful tag" },
      { "name": "#internal-tag" }
    ],
    "authors": [
      { "id": "5c739b7c8a59a6c8ddc164a1" }
    ]
  }]
}
```

> Tags prefixed with `#` are **internal tags** (invisible to readers). Tags not found are **automatically created**. If no author is matched, Ghost falls back to the owner-role user.

### Update a Post

```
PUT /admin/posts/{id}/
```

**Required:** `updated_at` (collision detection — must match the current `updated_at` value from a fresh GET)

```json
// PUT /admin/posts/5b7ada404f87d200b5b1f9c8/
{
  "posts": [
    {
      "title": "My updated title",
      "updated_at": "2022-06-05T20:52:37.000Z"
    }
  ]
}
```

**Save a revision:**
```
PUT /admin/posts/{id}/?save_revision=true
```

> ⚠️ Tag and author relations are **replaced, not merged**. Always GET first, modify the array, then PUT the full modified array back.

### Publish a Post

```json
// PUT /admin/posts/5b7ada404f87d200b5b1f9c8/
{
  "posts": [
    {
      "status": "published",
      "updated_at": "2022-06-05T20:52:37.000Z"
    }
  ]
}
```

### Schedule a Post

Set `status` to `scheduled` with a future `published_at`:

```json
// PUT /admin/posts/5b7ada404f87d200b5b1f9c8/
{
  "posts": [
    {
      "status": "scheduled",
      "published_at": "2026-12-25T09:00:00.000Z",
      "updated_at": "2022-06-05T20:52:37.000Z"
    }
  ]
}
```

At the `published_at` time, the post will be published and email newsletters sent (if configured). Email-only posts change status to `sent`.

### Send a Post via Email

```json
// PUT /admin/posts/5b7ada404f87d200b5b1f9c8/
{
  "posts": [
    {
      "status": "published",
      "updated_at": "2022-06-05T20:52:37.000Z",
      "newsletter": { "id": "62750bff2b868a34f814af08" },
      "email_segment": "status:free"
    }
  ]
}
```

`email_segment` can be a [NQL filter](https://ghost.org/docs/content-api/filtering/) targeting members (e.g. `status:free`, `status:-free`, `all`).

### Delete a Post

```
DELETE /admin/posts/{id}/
```

Returns `204 No Content` on success.

---

## Pages

Pages are static resources not included in post feeds. They share the same object schema and CRUD operations as posts.

```
GET    /admin/pages/
GET    /admin/pages/{id}/
GET    /admin/pages/slug/{slug}/
POST   /admin/pages/
PUT    /admin/pages/{id}/
DELETE /admin/pages/{id}/
```

All post create/update patterns apply identically to pages.

---

## Tags

```
GET    /admin/tags/
GET    /admin/tags/{id}/
GET    /admin/tags/slug/{slug}/
POST   /admin/tags/
PUT    /admin/tags/{id}/
DELETE /admin/tags/{id}/
```

**Tag object fields:**

| Field | Description |
|-------|-------------|
| `name` | Tag display name (**required on create**) |
| `slug` | URL-friendly identifier |
| `description` | Tag description |
| `feature_image` | Feature image URL |
| `visibility` | `public` or `internal` (prefix name with `#` to make internal) |
| `meta_title` | SEO meta title |
| `meta_description` | SEO meta description |
| `og_image`, `og_title`, `og_description` | Open Graph overrides |
| `twitter_image`, `twitter_title`, `twitter_description` | Twitter Card overrides |
| `accent_color` | Hex color for tag accent |
| `codeinjection_head` | Code injection for tag pages |
| `codeinjection_foot` | Code injection for tag pages |

---

## Members

The members API manages your publication's audience (free, paid, comped, and gift members).

```
GET  /admin/members/
GET  /admin/members/{id}/
POST /admin/members/
PUT  /admin/members/{id}/
```

### Member Object Schema

```json
// GET /admin/members/?include=newsletters,labels
{
  "members": [
    {
      "id": "623199bfe8bc4d3097caefe0",
      "uuid": "4fa3e4df-85d5-44bd-b0bf-d504bbe22060",
      "email": "jamie@example.com",
      "name": "Jamie",
      "note": null,
      "geolocation": null,
      "subscribed": true,
      "created_at": "2022-03-16T08:03:11.000Z",
      "updated_at": "2022-03-16T08:03:40.000Z",
      "status": "free",
      "comped": false,
      "last_seen_at": "2022-05-20T16:29:29.000Z",
      "avatar_image": "https://gravatar.com/avatar/...",
      "email_count": 0,
      "email_opened_count": 0,
      "email_open_rate": null,
      "can_comment": true,
      "unsubscribe_url": "https://example.com/unsubscribe/?uuid=...",
      "labels": [
        { "id": "...", "name": "VIP", "slug": "vip", "created_at": "...", "updated_at": "..." }
      ],
      "newsletters": [
        { "id": "...", "name": "Weekly newsletter", "description": null, "status": "active" }
      ],
      "subscriptions": [],
      "commenting": {
        "disabled": false,
        "disabled_reason": null,
        "disabled_until": null
      },
      "email_suppression": {
        "suppressed": false,
        "info": null
      }
    }
  ]
}
```

**Member field reference:**

| Field | Description |
|-------|-------------|
| `email` | Member's email address |
| `name` | Member's display name |
| `note` | Internal note (max 2000 chars) |
| `geolocation` | JSON geolocation from member's IP (`country`, `city`, `region`) |
| `subscribed` | `true` if subscribed to ≥ 1 active newsletter |
| `status` | `free`, `paid`, `comped`, or `gift` |
| `comped` | `true` if on a complimentary subscription |
| `last_seen_at` | Timestamp of most recent site activity |
| `avatar_image` | Gravatar URL |
| `email_count` | Total emails sent to this member |
| `email_opened_count` | Total emails opened |
| `email_open_rate` | Open rate as integer 0–100, or `null` (too few emails) |
| `labels` | Array of label objects |
| `newsletters` | Subscribed newsletters (requires `?include=newsletters`) |
| `tiers` | Accessible tiers (requires `?include=tiers`, always on single read) |
| `subscriptions` | Active paid/comped/gift subscriptions |
| `unsubscribe_url` | One-click unsubscribe URL |
| `can_comment` | Whether the member can comment |
| `commenting` | `{disabled, disabled_reason, disabled_until}` — moderation state |
| `email_suppression` | `{suppressed, info}` — bounce/complaint suppression status |
| `attribution` | Signup source attribution (single-read endpoint only) |
| `email_recipients` | Per-email delivery records (requires `?include=email_recipients`) |

**Create a member:**
```json
// POST /admin/members/
{
  "members": [
    {
      "email": "jamie@example.com",
      "name": "Jamie",
      "note": "Signed up via promo",
      "labels": ["vip"],
      "newsletters": [{ "id": "62750bff2b868a34f814af08" }]
    }
  ]
}
```

**Update a member:**
```json
// PUT /admin/members/623199bfe8bc4d3097caefe0/
{
  "members": [
    {
      "name": "Jamie Smith",
      "note": "Updated note"
    }
  ]
}
```

### Subscription Object

Paid members include subscription details:

```json
{
  "id": "sub_1KlTkYSHlkrEJE2dGbzcgc61",
  "customer": {
    "id": "cus_LSOXHFwQB7ql18",
    "name": "Jamie",
    "email": "jamie@ghost.org"
  },
  "status": "active",
  "start_date": "2022-04-06T07:57:58.000Z",
  "default_payment_card_last4": "4242",
  "cancel_at_period_end": false,
  "cancellation_reason": null,
  "current_period_end": "2023-04-06T07:57:58.000Z",
  "trial_start_at": null,
  "trial_end_at": null,
  "discount_start": null,
  "discount_end": null,
  "price": {
    "id": "price_1Kg0ymSHlkrEJE2dflUN66EW",
    "price_id": "6239692c664a9e6f5e5e840a",
    "nickname": "Yearly",
    "amount": 100000,
    "interval": "year",
    "type": "recurring",
    "currency": "USD",
    "tier": { "id": "prod_LX9o7VWBLU3QLh", "name": "Platinum", "tier_id": "62307cc71b4376a976734038" }
  },
  "tier": { "id": "62307cc71b4376a976734038", "name": "Platinum", "monthly_price": 1000, "yearly_price": 10000, ... },
  "next_payment": {
    "original_amount": 100000,
    "amount": 100000,
    "interval": "year",
    "currency": "USD",
    "discount": null
  }
}
```

**Subscription status values:** `active`, `trialing`, `past_due`, `canceled`, `unpaid`

**Price `type` values:** `recurring`, `one_time`, `donation`

> `plan` field is **deprecated** — use `price` instead.

---

## Newsletters

Newsletters allow members to opt in/out of different content categories. Each site has one newsletter by default.

```
GET  /admin/newsletters/
GET  /admin/newsletters/{id}/
POST /admin/newsletters/
PUT  /admin/newsletters/{id}/
```

**Newsletter object fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Public newsletter name (**required on create**) |
| `description` | string\|null | Public description |
| `slug` | string | Used to reference newsletter when sending posts |
| `status` | string | `active` or `archived` |
| `sender_name` | string\|null | From name in email header |
| `sender_email` | string\|null | From email (requires validation) |
| `sender_reply_to` | string | `newsletter` (use sender_email) or `support` |
| `subscribe_on_signup` | boolean | Auto-subscribe new members |
| `visibility` | string | `members` or `paid` |
| `header_image` | string\|null | Email header image URL (recommended 1200×600) |
| `show_header_icon` | boolean | Show site icon in emails |
| `show_header_title` | boolean | Show site name in emails |
| `show_header_name` | boolean | Show newsletter name in emails |
| `title_font_category` | string | `serif` or `sans_serif` |
| `body_font_category` | string | `serif` or `sans_serif` |
| `show_feature_image` | boolean | Show post feature image in emails |
| `footer_content` | string\|null | HTML footer content |
| `show_badge` | boolean | Show Ghost badge in footer |
| `sort_order` | integer | Display order |

**Create a newsletter:**
```json
// POST /admin/newsletters/
{
  "newsletters": [
    {
      "name": "Premium Updates",
      "description": "Weekly in-depth analysis for paid subscribers",
      "status": "active",
      "subscribe_on_signup": false,
      "visibility": "paid"
    }
  ]
}
```

---

## Tiers

Tiers define membership levels with price points, benefits, and access permissions. Connected directly to Stripe.

```
GET  /admin/tiers/
GET  /admin/tiers/{id}/
POST /admin/tiers/
PUT  /admin/tiers/{id}/
```

**Tier object example:**
```json
// GET /admin/tiers/?include=monthly_price,yearly_price,benefits
{
  "tiers": [
    {
      "id": "622727ad96a190e914ab6664",
      "name": "Free",
      "slug": "free",
      "type": "free",
      "active": true,
      "visibility": "public",
      "welcome_page_url": null,
      "monthly_price": null,
      "yearly_price": null,
      "benefits": []
    },
    {
      "id": "622727ad96a190e914ab6665",
      "name": "Bronze",
      "slug": "default-product",
      "type": "paid",
      "active": true,
      "visibility": "public",
      "welcome_page_url": null,
      "monthly_price": 500,
      "yearly_price": 5000,
      "currency": "usd",
      "benefits": ["Free daily newsletter", "3 posts a week"]
    }
  ]
}
```

**Tier `include` options:** `monthly_price`, `yearly_price`, `benefits`

**Tier `filter` options:**
- `type:free` / `type:paid`
- `visibility:public` / `visibility:none`
- `active:true` / `active:false`

> Prices are in the **smallest currency unit** (e.g. 500 = $5.00 USD).

---

## Offers

Offers create discounts or special pricing for members signing up on a tier.

```
GET  /admin/offers/
GET  /admin/offers/{id}/
POST /admin/offers/
PUT  /admin/offers/{id}/
```

**Offer object:**
```json
{
  "offers": [
    {
      "id": "6230dd69e8bc4d3097caefd3",
      "name": "Black Friday",
      "code": "black-friday",
      "display_title": "Black Friday Sale!",
      "display_description": "10% off our yearly price",
      "type": "percent",
      "cadence": "year",
      "amount": 10,
      "duration": "once",
      "duration_in_months": null,
      "currency_restriction": false,
      "currency": null,
      "status": "active",
      "redemption_count": 0,
      "tier": { "id": "62307cc71b4376a976734038", "name": "Platinum" }
    }
  ]
}
```

**Offer field reference:**

| Field | Description |
|-------|-------------|
| `name` | Internal name (must be unique) |
| `code` | URL shortcode (e.g. `yoursite.com/black-friday`) |
| `display_title` | Title shown to visitors |
| `display_description` | Description shown to visitors |
| `type` | `percent` or `fixed` |
| `amount` | Discount amount (% for `percent`; smallest currency unit for `fixed`) |
| `cadence` | `month` or `year` — which price point the offer applies to |
| `duration` | `once`, `forever`, or `repeating` (monthly only) |
| `duration_in_months` | Number of months (when `duration: repeating`) |
| `currency` | ISO currency code (required for `fixed` type) |
| `currency_restriction` | If `true`, changing currency invalidates the offer |
| `status` | `active` or `archived` |
| `redemption_count` | Times the offer has been redeemed |
| `tier` | The tier the offer applies to |

---

## Images

Upload images to Ghost's storage adapter (default: local `/content/images/`).

```
POST /admin/images/upload/
```

The request must use `multipart/form-data`:

```bash
curl -X POST \
     -H "Authorization: Ghost $TOKEN" \
     -H "Accept-Version: v6.0" \
     -F "file=@/path/to/image.png;type=image/png" \
     -F "ref=my-image.png" \
     "https://{admin_domain}/ghost/api/admin/images/upload/"
```

**Response:**
```json
{
  "images": [
    {
      "url": "https://demo.ghost.io/content/images/2019/02/ghost-logo.png",
      "ref": "ghost-logo.png"
    }
  ]
}
```

| Field | Description |
|-------|-------------|
| `url` | The uploaded image's storage URL |
| `ref` | The reference string provided at upload (optional) |

> Filenames are sanitized. No modifications to the image itself are made by the default adapter.

---

## Users

Read-only access to staff users (write operations require user authentication).

```
GET /admin/users/
GET /admin/users/{id}/
GET /admin/users/slug/{slug}/
GET /admin/users/me/
```

Supports `include=roles` to include the user's role assignment.

**Available roles:** Owner, Administrator, Editor, Author, Contributor

---

## Webhooks

Programmatically manage webhooks via the Admin API.

```
POST   /admin/webhooks/
PUT    /admin/webhooks/{id}/
DELETE /admin/webhooks/{id}/
```

**Create a webhook:**
```json
// POST /admin/webhooks/
{
  "webhooks": [
    {
      "event": "post.published",
      "target_url": "https://yourapp.com/hooks/ghost-publish",
      "name": "Post published handler",
      "secret": "optional-secret-for-hmac-verification",
      "api_version": "v6"
    }
  ]
}
```

**Available events:** See the full list in [ghost-api-overview.md](./ghost-api-overview.md#webhooks).

Ghost signs webhook payloads with an HMAC signature (using the `secret`) in the `X-Ghost-Signature` header when a secret is set.

---

## Site

Read global site metadata.

```
GET /admin/site/
```

Returns site `title`, `description`, `logo`, `icon`, `cover_image`, `lang`, `url`, and `version`.

---

## Themes

```
POST /admin/themes/upload/
PUT  /admin/themes/{name}/activate/
```

**Upload a theme** (zip file):
```bash
curl -X POST \
     -H "Authorization: Ghost $TOKEN" \
     -F "file=@/path/to/theme.zip;type=application/zip" \
     "https://{admin_domain}/ghost/api/admin/themes/upload/"
```

**Activate a theme:**
```bash
PUT /admin/themes/casper/activate/
```

---

## Error Handling & Collision Detection

### Errors

All errors follow this shape:

```json
{
  "errors": [
    {
      "message": "Validation error",
      "context": "Title cannot be blank",
      "type": "ValidationError",
      "details": null,
      "property": "title",
      "help": null,
      "code": null,
      "id": "abc123"
    }
  ]
}
```

**Common status codes:**

| Code | Meaning |
|------|---------|
| `200` | OK |
| `201` | Created |
| `204` | No Content (DELETE success) |
| `400` | Bad Request |
| `401` | Unauthorized (bad/missing token) |
| `403` | Forbidden (insufficient permissions, or 2FA required) |
| `404` | Not Found |
| `409` | Conflict |
| `422` | Unprocessable Entity (validation failed) |
| `500` | Internal Server Error |

### Collision Detection on Updates

All `PUT` requests require `updated_at` to match the current value stored in Ghost. This prevents overwriting concurrent changes.

**Best practice for updates:**
1. `GET /admin/posts/{id}/` → get the current `updated_at`
2. Modify only the fields you need to change
3. `PUT /admin/posts/{id}/` with the original `updated_at` + your changes

---

## JavaScript Client Quick Reference

```js
const GhostAdminAPI = require('@tryghost/admin-api');

const api = new GhostAdminAPI({
    url: 'https://your-site.ghost.io',
    key: '{id}:{secret}',
    version: 'v6'
});

// Posts
api.posts.browse({ limit: 10, include: 'tags,authors' });
api.posts.read({ id: '...' }, { formats: ['html', 'lexical'] });
api.posts.add({ title: 'New Post', status: 'draft' });
api.posts.edit({ id: '...', title: 'Updated', updated_at: '...' });
api.posts.delete({ id: '...' });

// Members
api.members.browse({ limit: 'all', include: 'newsletters,labels' });
api.members.add({ email: 'user@example.com', name: 'User' });

// Images
api.images.upload({ file: '/path/to/image.png', ref: 'my-image' });
```

---

*Official docs: https://ghost.org/docs/admin-api/ | Full API index: https://docs.ghost.org/llms.txt*
