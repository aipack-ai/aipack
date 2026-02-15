## aip.web

Functions for making HTTP GET and POST requests, and for URL manipulation.

### Functions Summary

```lua
aip.web.get(url: string, options?: WebOptions): WebResponse

aip.web.post(url: string, data: string | table, options?: WebOptions): WebResponse

aip.web.parse_url(url: string | nil): table | nil

aip.web.resolve_href(href: string | nil, base_url: string): string | nil
```

### Constants

- `aip.web.UA_AIPACK: string`: Default aipack User Agent string (`aipack`).
- `aip.web.UA_BROWSER: string`: Default browser User Agent string.

### aip.web.get

Makes an HTTP GET request.

```lua
-- API Signature
aip.web.get(url: string, options?: WebOptions): WebResponse
```

#### Arguments

- `url: string`: The URL to request.
- `options?`: [WebOptions](#weboptions): Optional web request options ([WebOptions](#weboptions)).

#### Returns

- [WebResponse](#webresponse): A [WebResponse](#webresponse) table containing the result.

#### Example

```lua
local response = aip.web.get("https://httpbin.org/get")
-- default `User-Agent` is `aipack`
if response.success then
  print("Status:", response.status)
  -- response.content might be a string or table (if JSON)
  print("Content Type:", type(response.content))
else
  print("Error:", response.error, "Status:", response.status)
end

-- With options (user_agent: true uses 'aipack' default UA)
local response_with_opts = aip.web.get("https://api.example.com/data", {
  user_agent = "my-user-agent",
  headers = { ["X-API-Key"] = "secret123" },
  redirect_limit = 10
})

-- Example of using the browser UA constant
local response_browser_ua = aip.web.get("https://api.example.com/data", {
  user_agent = aip.web.UA_BROWSER,
})
```

#### Error

Returns an error (Lua table `{ error: string }`) if the request cannot be initiated (e.g., network error, invalid URL). Check `response.success` for HTTP-level errors (non-2xx status).

### aip.web.post

Makes an HTTP POST request.

```lua
-- API Signature
aip.web.post(url: string, data: string | table, options?: WebOptions): WebResponse
```

Sends `data` in the request body. If `data` is a string, `Content-Type` is `text/plain`. If `data` is a table, it's serialized to JSON and `Content-Type` is `application/json`.

#### Arguments

- `url: string`: The URL to request.
- `data: string | table`: Data to send in the body.
- `options?`: [WebOptions](#weboptions): Optional web request options ([WebOptions](#weboptions)).

#### Returns

- [WebResponse](#webresponse): A [WebResponse](#webresponse) table containing the result.

#### Example

```lua
-- POST plain text
local r1 = aip.web.post("https://httpbin.org/post", "plain text data")

-- POST JSON
local r2 = aip.web.post("https://httpbin.org/post", { key = "value", num = 123 }, { parse = true })
if r2.success and type(r2.content) == "table" then
  print("Received JSON echo:", r2.content.json.key) -- Output: value
end

-- POST with options
local r3 = aip.web.post("https://api.example.com/submit", { data = "value" }, {
  user_agent = "MyApp/1.0",
  headers = { ["X-API-Key"] = "secret123" }
})
```

#### Error

Returns an error (Lua table `{ error: string }`) if the request cannot be initiated or data serialization fails. Check `response.success` for HTTP-level errors.

### aip.web.parse_url

Parses a URL string and returns its components as a table.

```lua
-- API Signature
aip.web.parse_url(url: string | nil): table | nil
```

Parses the given URL string and extracts its various components.

#### Arguments

- `url: string | nil`: The URL string to parse. If `nil` is provided, the function returns `nil`.

#### Returns
 (`table | nil`)

- If the `url` is a valid string, returns a table with the following fields:
  - `scheme: string` (e.g., "http", "https")
  - `host: string | nil` (e.g., "example.com")
  - `port: number | nil` (e.g., 80, 443)
  - `path: string` (e.g., "/path/to/resource")
  - `query: table | nil` (A Lua table where keys are query parameter names and values are their corresponding string values. E.g., `{ name = "value" }`)
  - `fragment: string | nil` (The part of the URL after '#')
  - `username: string` (The username for authentication, empty string if not present)
  - `password: string | nil` (The password for authentication)
  - `url: string` (The original or normalized URL string that was parsed)
  - `page_url: string` - (The url without the query and fragment
- If the input `url` is `nil`, the function returns `nil`.

#### Example

```lua
local parsed = aip.web.parse_url("https://user:pass@example.com:8080/path/to/page.html?param1=val#fragment")
if parsed then
  print(parsed.scheme)       -- "https"
  print(parsed.host)         -- "example.com"
  print(parsed.port)         -- 8080
  print(parsed.path)         -- "/path/to/page.html"
  print(parsed.query.param1) -- "val"
  print(parsed.fragment)     -- "fragment"
  print(parsed.username)     -- "user"
  print(parsed.password)     -- "pass"
  print(parsed.url)          -- "https://user:pass@example.com:8080/path/to/page.html?query=val#fragment"
  print(parsed.page_url)     -- "https://user:pass@example.com:8080/path/to/page.html"
end

local nil_result = aip.web.parse_url(nil)
-- nil_result will be nil
```

#### Error


Returns an error (Lua table `{ error: string }`) if the `url` string is provided but is invalid and cannot be parsed.


### aip.web.resolve_href

Resolves an `href` (like one from an HTML `<a>` tag) against a `base_url`.

```lua
-- API Signature
aip.web.resolve_href(href: string | nil, base_url: string): string | nil
```

#### Arguments

- `href: string | nil`: The href string to resolve. This can be an absolute URL, a scheme-relative URL, an absolute path, or a relative path. If `nil`, the function returns `nil`.
- `base_url: string`: The base URL string against which to resolve the `href`. Must be a valid absolute URL.

#### Returns
 (`string | nil`)

- If `href` is `nil`, returns `nil`.
- If `href` is already an absolute URL (e.g., "https://example.com/page"), it's returned as is.
- Otherwise, `href` is joined with `base_url` to form an absolute URL.
- Returns the resolved absolute URL string.

#### Example

```lua
local base = "https://example.com/docs/path/"

-- Absolute href
print(aip.web.resolve_href("https://another.com/page.html", base))
-- Output: "https://another.com/page.html"

-- Relative path href
print(aip.web.resolve_href("sub/page.html", base))
-- Output: "https://example.com/docs/path/sub/page.html"

-- Absolute path href
print(aip.web.resolve_href("/other/resource.txt", base))
-- Output: "https://example.com/other/resource.txt"

-- Scheme-relative href
print(aip.web.resolve_href("//cdn.com/asset.js", base))
-- Output: "https://cdn.com/asset.js" (uses base_url's scheme)

print(aip.web.resolve_href("//cdn.com/asset.js", "http://example.com/"))
-- Output: "http://cdn.com/asset.js"

-- href is nil
print(aip.web.resolve_href(nil, base))
-- Output: nil (Lua nil)
```

#### Error


Returns an error (Lua table `{ error: string }`) if:
- `base_url` is not a valid absolute URL.
- `href` and `base_url` cannot be successfully joined (e.g., due to malformed `href`).
