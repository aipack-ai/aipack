## aip.pdf

The `aip.pdf` module exposes functions to work with PDF files.

### Functions Summary

```lua
aip.pdf.page_count(path: string): number

aip.pdf.split_pages(path: string, dest_dir?: string): FileInfo[]
```

### aip.pdf.page_count

Returns the number of pages in a PDF file.

```lua
-- API Signature
aip.pdf.page_count(path: string): number
```

#### Arguments

- `path: string` - The path to the PDF file.

#### Returns

- `number` - The number of pages in the PDF.

#### Example

```lua
local count = aip.pdf.page_count("documents/report.pdf")
print("Page count:", count)
```

#### Error

Returns an error if:
- The file does not exist or cannot be read.
- The file is not a valid PDF.

### aip.pdf.split_pages

Splits a PDF into individual page files.

```lua
-- API Signature
aip.pdf.split_pages(path: string, dest_dir?: string): FileInfo[]
```

Splits the PDF at `path` into individual single-page PDF files.

If `dest_dir` is not provided, the destination directory defaults to a folder
in the same location as the source PDF, named after the PDF's stem (filename without extension).

For example, if `path` is `"docs/report.pdf"`, the default destination would be `"docs/report/"`.

Each page file is named `{stem}-page-{NNNN}.pdf` where `{stem}` is the original filename
without extension and `{NNNN}` is a zero-padded 4-digit page number.

#### Arguments

- `path: string` - The path to the PDF file to split.
- `dest_dir?: string` (optional) - The destination directory for the split page files.
  If not provided, defaults to a folder named after the PDF stem in the same directory.

#### Returns

- `FileInfo[]` - A list of [FileInfo](#fileinfo) objects for each created page file.

#### Example

```lua
-- Split with default destination (creates "docs/report/" folder)
local pages = aip.pdf.split_pages("docs/report.pdf")
for _, page in ipairs(pages) do
  print(page.path) -- e.g., "docs/report/report-page-0001.pdf"
  print(page.name) -- e.g., "report-page-0001.pdf"
end

-- Split to a specific destination
local pages = aip.pdf.split_pages("docs/report.pdf", "output/pages")
for _, page in ipairs(pages) do
  print(page.path, page.size)
end
```

#### Error

Returns an error if:
- The source file does not exist or cannot be read.
- The file is not a valid PDF.
- The destination directory cannot be created.
- Any page cannot be saved.
