# Form Fields (AcroForms)

## Purpose

Enables creating fillable PDF forms using PDF AcroForms (ISO 32000-1 §12.7).
Use cases: onboarding forms, applications, contracts, data-collection sheets.

## How It Works

### API

```rust
doc.begin_page(612.0, 792.0);
doc.add_text_field("full_name", Rect { x: 180.0, y: 706.0, width: 300.0, height: 18.0 })?;
doc.add_text_field("email",     Rect { x: 180.0, y: 676.0, width: 300.0, height: 18.0 })?;
doc.end_page().unwrap();
```

`add_text_field(name, rect)` must be called while a page is active (between `begin_page`
and `end_page`). Returns `Err(FormFieldError)` on failure.

### PHP

```php
$doc->addTextField("full_name", new Rect(180, 706, 300, 18));
$doc->addTextField("email",     new Rect(180, 676, 300, 18));
```

### Object Structure

Each text field produces one PDF Widget annotation object:

```
/Type /Annot
/Subtype /Widget
/FT /Tx          — text field type
/T (field_name)  — unique field name
/Rect [x y x+w y+h]
/P <page-ref>    — back-reference to the page
/F 4             — Print flag (field is printed with the page)
```

The document catalog gains an `/AcroForm` entry:

```
/AcroForm <<
  /Fields [<widget-ref> ...]   — all fields across all pages
  /NeedAppearances true        — viewer generates visual appearance
  /DA (/Helv 12 Tf 0 g)        — default appearance (Helvetica 12pt black)
>>
```

Pages with fields gain an `/Annots [<widget-ref> ...]` entry in their page dict.

## Design Decisions

### Invisible by Default

Fields have no border, background, or default value. The PDF spec allows viewers
to render their own chrome (border, cursor highlight, etc.) without the document
specifying it. `NeedAppearances true` delegates appearance generation to the viewer,
which means the PDF is viewer-portable without any `/AP` (appearance stream) work.

### Pre-allocated ObjIds at `end_page()`

Widget annotation ObjIds are pre-allocated in `end_page()` (same pattern as page
dict ObjIds). This keeps the object numbering forward-sequential in the file, which
simplifies the xref table and matches how other deferred objects (page dicts, fonts)
are handled.

### Document-level Uniqueness via `BTreeSet<String>`

AcroForm field names must be unique within a document (PDF spec §12.7.3.2).
A `BTreeSet<String>` on `PdfDocument` enforces this. Duplicate names produce
`FormFieldError::DuplicateName` immediately.

### `FormFieldDef` → `FormFieldRecord` Transition

While a page is open, fields are stored as `FormFieldDef` (name + rect) in
`PageBuilder`. At `end_page()` they are converted to `FormFieldRecord` (name + rect
+ pre-allocated ObjId) and stored in `PageRecord`. This separation keeps the
open-page state lightweight and the committed-page state complete.

## Error Cases

| Error | Condition |
|-------|-----------|
| `FormFieldError::NoActivePage` | `add_text_field` called with no open page |
| `FormFieldError::DuplicateName(name)` | Same field name used more than once |

## Limitations

- Text fields only (no checkboxes, radio buttons, dropdowns — future issues)
- No pre-filled values (`/V` key not set)
- No multi-line support
- No visible styling (border, background) — viewer provides its own chrome
- Overlays (opened via `open_page`) cannot add new form fields

## Examples

- Rust: `examples/rust/generate_form_fields.rs`
- PHP: `examples/php/generate_form_fields.php`

## History

- **Issue 34**: Initial implementation — AcroForm text fields with uniqueness
  enforcement, Widget annotations, page `/Annots` array, catalog `/AcroForm` entry.
